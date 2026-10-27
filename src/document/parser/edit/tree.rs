//! Позиционное дерево исходника: проход по строкам, зеркалящий
//! `PositionedParser` (fmt_parser), но вместо trivia записывающий
//! byte-диапазоны значений. Диапазоны снимаются на тех же решениях
//! line dispatch'а, что и у реального парсера: те же
//! `scan_unescaped_colon` / `classify_separator` / `classify_value_shape`
//! / `classify_root_shape`, та же вставка через `insert_value`
//! (dotted-key expansion, reopen, duplicate/conflict).

use std::borrow::Cow;

use indexmap::map::Entry;
use indexmap::IndexMap;
use memchr::{memchr, memchr2};

use crate::error::{CompoundKind, Error, ErrorKind, Span};
use crate::value::Scalar;
use crate::whitespace::is_ktav_whitespace;

use crate::parser::bracket::Bracket;
use crate::parser::classify::{classify_value_shape, ValueShape};
use crate::parser::collecting::MultilineMode;
use crate::parser::inline::{
    find_unescaped_colon_inline, has_quote_bytes, scan_inline_closer_with_bounds,
    scan_unescaped_colon, split_top_level, ColonScan, InlineBody, InlineBounds, MAX_INLINE_DEPTH,
};
use crate::parser::insert::{insert_value, InsertShape, InsertTable, OccupiedShape};
use crate::parser::parser::{
    classify_root_shape, classify_separator, require_sep_end, trimmed_span_in, RootShape, Separator,
};

// ---------------------------------------------------------------------------
// Узлы дерева
// ---------------------------------------------------------------------------

/// Что стоит в узле: скаляр, объект или массив. Inline-узлы отличаются
/// полем `inline: bool` у `Node`, а не отдельной разновидностью.
pub(crate) enum Kind {
    Scalar,
    Object(ObjTable),
    Array(Vec<Node>),
}

/// Как узел лежит в исходнике; определяет границы сплайса.
#[derive(Clone, Copy)]
pub(crate) enum Slot {
    /// Значение пары `key: value` / `key:: value`: `colon` — байт
    /// первого `:` (точка, откуда можно менять маркер), `body` —
    /// точные байты тела значения.
    Pair { colon: u32, body: (u32, u32) },
    /// Элемент массива в свою строку / сегмент inline-массива: `start` —
    /// первый байт содержимого (включая маркер `::`), `body` — байты тела.
    Item { start: u32, body: (u32, u32) },
    /// Многострочный блок: multiline-строка `(...)`/`((...))` или
    /// контейнер `{ ... }` / `[ ... ]`. `anchor` — `:` пары, открывшей
    /// блок (None для элемента массива); диапазон — от opener до конца
    /// закрывающей строки включительно.
    Block {
        anchor: Option<u32>,
        start: u32,
        end: u32,
    },
}

pub(crate) struct Node {
    pub(crate) kind: Kind,
    /// Узел построен из inline-compound (`{ ... }` / `[ ... ]`).
    pub(crate) inline: bool,
    /// None — синтетический (noncontiguous) узел: промежуточный объект
    /// dotted-key/reopen, либо корень, либо inline-узел до навешивания
    /// слота родителем.
    pub(crate) slot: Option<Slot>,
    /// Диапазон inline-корня из диспетчеризации формы парсером.
    pub(crate) root_range: Option<(u32, u32)>,
}

impl Node {
    fn scalar(slot: Slot) -> Node {
        Node {
            kind: Kind::Scalar,
            inline: false,
            slot: Some(slot),
            root_range: None,
        }
    }
    fn empty_object(slot: Slot) -> Node {
        Node {
            kind: Kind::Object(ObjTable::default()),
            inline: false,
            slot: Some(slot),
            root_range: None,
        }
    }
    fn empty_array(slot: Slot) -> Node {
        Node {
            kind: Kind::Array(Vec::new()),
            inline: false,
            slot: Some(slot),
            root_range: None,
        }
    }

    /// Слот узла (None — синтетический/корневой).
    pub(crate) fn slot(&self) -> Option<Slot> {
        self.slot
    }

    /// Ребёнок-объект по уже декодированному ключу.
    pub(crate) fn child(&self, key: &str) -> Option<&Node> {
        match &self.kind {
            Kind::Object(t) => t.entries.get(key),
            _ => None,
        }
    }

    /// Мутирующий вариант [`Node::child`] — для будущего спуска сплайса.
    #[allow(dead_code)]
    pub(crate) fn child_mut(&mut self, key: &str) -> Option<&mut Node> {
        match &mut self.kind {
            Kind::Object(t) => t.entries.get_mut(key),
            _ => None,
        }
    }

    /// i-й элемент массива.
    pub(crate) fn item(&self, i: usize) -> Option<&Node> {
        match &self.kind {
            Kind::Array(items) => items.get(i),
            _ => None,
        }
    }
}

/// Упорядоченная таблица пар объекта. Реализует `InsertTable`, чтобы
/// dotted-key / reopen / duplicate-семантика совпадала с `parse_str`
/// байт-в-байт. `IndexMap` — тот же контейнер, что `PObject`
/// (fmt_parser/doc.rs): O(1) вставка/поиск, сохранение порядка.
#[derive(Default)]
pub(crate) struct ObjTable {
    pub(crate) entries: IndexMap<Scalar, Node>,
}

impl InsertShape for Node {
    fn is_object(&self) -> bool {
        matches!(self.kind, Kind::Object(_))
    }
    fn kind_label(&self) -> &'static str {
        match &self.kind {
            Kind::Scalar => "string",
            Kind::Object(_) => "object",
            Kind::Array(_) => "array",
        }
    }
}

impl<'k> InsertTable<'k> for ObjTable {
    type Value = Node;

    fn insert_leaf(&mut self, key: Cow<'k, str>, value: Node) -> Result<(), (OccupiedShape, Node)> {
        // Ключ приходит УЖЕ декодированным от insert_value.
        match self.entries.entry(key.as_ref().into()) {
            Entry::Occupied(e) => {
                let existing = e.get();
                Err((
                    OccupiedShape {
                        is_object: existing.is_object(),
                        label: existing.kind_label(),
                    },
                    value,
                ))
            }
            Entry::Vacant(v) => {
                v.insert(value);
                Ok(())
            }
        }
    }

    fn descend(&mut self, key: Cow<'k, str>) -> Result<&mut Self, ()> {
        match self.entries.entry(key.as_ref().into()) {
            Entry::Occupied(e) => {
                let node = e.into_mut();
                match &mut node.kind {
                    Kind::Object(t) => {
                        // Reopen / дальний dotted-спуск делает узел
                        // noncontiguous: ложный contiguous-слот
                        // инвалидируется даже у positioned-объекта.
                        node.slot = None;
                        Ok(t)
                    }
                    _ => Err(()),
                }
            }
            Entry::Vacant(e) => {
                let inserted = e.insert(Node {
                    kind: Kind::Object(ObjTable::default()),
                    inline: false,
                    slot: None,
                    root_range: None,
                });
                match &mut inserted.kind {
                    Kind::Object(t) => Ok(t),
                    _ => unreachable!("just inserted an object"),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Построитель
// ---------------------------------------------------------------------------

enum Frame<'a> {
    Object {
        table: ObjTable,
        pending_key: Option<&'a str>,
        pending_span: Option<Span>,
        /// `:` пары, открывшей текущий pending-блок (для Slot::Block).
        pending_anchor: Option<u32>,
        /// Байт opener'а блока, открытого этим фреймом.
        range_start: u32,
    },
    Array {
        items: Vec<Node>,
        range_start: u32,
    },
}

struct Builder<'a> {
    stack: Vec<Frame<'a>>,
    collecting: Option<MultilineMode>,
    /// Байт opener'а `(` / `((` текущей multiline-строки.
    collecting_opener: u32,
    root_initialized: bool,
    root_consumed: bool,
    root_inline: Option<Node>,
    root_is_explicit: bool,
}

/// Byte-офсет `slice` внутри строки `line`, у которой абсолютный старт
/// `line_start` (обе — срезы одного буфера).
fn off_in_line(line: &str, line_start: u32, slice: &str) -> u32 {
    line_start + (slice.as_ptr() as usize - line.as_ptr() as usize) as u32
}

/// Внутренняя ошибка позиционирования: после успешного `parse(text)`
/// недостижима, отображается в MalformedInlineCompound.
fn internal_inline(line_num: usize, span: Span, detail: &str) -> Error {
    Error::Structured(ErrorKind::MalformedInlineCompound {
        line: line_num as u32,
        span,
        detail: detail.to_string(),
    })
}

/// Верхний уровень позиционирования inline-compound: ОДИН раз строит
/// bound-кеш (`scan_inline_closer_with_bounds`, offsets относительно
/// top-body) и консервативный флаг кавычек; рекурсия переиспользует их
/// через один и тот же `InlineBounds::over(TOP, ..)` — origin
/// сохраняется, повторных сканов суффиксов нет (сложность
/// O(N + nodes·log nodes) из-за бинарного поиска по memo).
/// Иммутабельные данные top-сканера inline-прохода: консервативный
/// флаг кавычек и memo-кеш границ (origin — top-body). Передаются в
/// рекурсию без изменений (Copy).
#[derive(Clone, Copy)]
struct InlineScanCtx<'a> {
    quotes: bool,
    bounds: InlineBounds<'a>,
}

fn position_inline_top(
    body: &str,
    base_abs: u32,
    line_num: usize,
    span: Span,
    is_object: bool,
) -> Result<Node, Error> {
    let quotes = has_quote_bytes(body.as_bytes());
    let mut pairs = Vec::new();
    let (open, close) = if is_object {
        (b'{', b'}')
    } else {
        (b'[', b']')
    };
    // После успешного parse ожидался бы Found; не-Found оставляет
    // pairs пустым — split уйдёт в live-dispatch (memo лишь оптимизация).
    let _ = scan_inline_closer_with_bounds(body, open, close, line_num, span, &mut pairs);
    let bounds = InlineBounds::over(body, &pairs);
    position_inline(
        body,
        base_abs,
        line_num,
        span,
        is_object,
        0,
        InlineScanCtx { quotes, bounds },
    )
}

/// Позиционировать inline-тело `body` (срез trimmed-строки, начинается
/// `{`/`[` и заканчивается парным closer'ом — гарантия classify-сканов),
/// чей первый байт лежит в исходнике по абсолютному офсету `base_abs`.
/// `quotes`/`bounds` — консервативный флаг кавычек и memo-кеш,
/// построенные на TOP-body (см. [`position_inline_top`]); они передаются
/// вниз без изменений — все child-слайсы суть подстроки top-body, так
/// что origin не сбрасывается. Ошибки здесь — internal (см. [`build`]).
fn position_inline(
    body: &str,
    base_abs: u32,
    line_num: usize,
    span: Span,
    is_object: bool,
    depth: usize,
    scan: InlineScanCtx<'_>,
) -> Result<Node, Error> {
    if depth > MAX_INLINE_DEPTH {
        return Err(internal_inline(
            line_num,
            span,
            "превышена максимальная глубина вложенности inline-compound",
        ));
    }
    let inner = &body[1..body.len() - 1];
    let segments = split_top_level(
        inner,
        line_num,
        span,
        if is_object {
            InlineBody::Object
        } else {
            InlineBody::Array
        },
        scan.bounds,
        scan.quotes,
    )
    .map_err(|_| internal_inline(line_num, span, "inline split failed после успешного parse"))?;

    if is_object {
        let mut table = ObjTable::default();
        for seg in segments {
            let trimmed = seg.trim_matches(is_ktav_whitespace);
            if trimmed.is_empty() {
                continue; // хвостовая запятая
            }
            let seg_base = off_in_line(body, base_abs, trimmed);
            let seg_end = seg_base + trimmed.len() as u32;
            let colon = match find_unescaped_colon_inline(trimmed) {
                Some(c) => c,
                None => {
                    return Err(internal_inline(
                        line_num,
                        span,
                        "inline-пара без разделителя",
                    ))
                }
            };
            let colon_abs = seg_base + colon as u32;
            let key = &trimmed[..colon];
            let after = &trimmed[colon + 1..];
            let node = if let Some(rest) = after.strip_prefix(':') {
                // raw-маркер `::`: тело — всё после маркера. Лист —
                // ФИЗИЧЕСКИ inline (контекст на самом узле).
                let v = rest.trim_start_matches(is_ktav_whitespace);
                Node {
                    kind: Kind::Scalar,
                    inline: true,
                    slot: Some(Slot::Pair {
                        colon: colon_abs,
                        body: (off_in_line(trimmed, seg_base, v), seg_end),
                    }),
                    root_range: None,
                }
            } else {
                let v = after.trim_start_matches(is_ktav_whitespace);
                let value_abs = off_in_line(trimmed, seg_base, v);
                let mut n = inline_child(v, value_abs, line_num, span, depth, scan)?;
                n.slot = Some(Slot::Pair {
                    colon: colon_abs,
                    body: (value_abs, seg_end),
                });
                n
            };
            insert_value(&mut table, key, node, line_num, span)?;
        }
        Ok(Node {
            kind: Kind::Object(table),
            inline: true,
            slot: None,
            root_range: None,
        })
    } else {
        // Inline-массив: `::` здесь — НЕ строковый raw-маркер, а
        // literal-данные элемента (parse_inline_value_raw получает
        // элемент ЦЕЛИКОМ): все non-compound сегменты — скаляры с телом
        // во весь trimmed-сегмент, без срезания префикса.
        let mut items = Vec::new();
        for seg in segments {
            let trimmed = seg.trim_matches(is_ktav_whitespace);
            if trimmed.is_empty() {
                continue;
            }
            let seg_base = off_in_line(body, base_abs, trimmed);
            let seg_end = seg_base + trimmed.len() as u32;
            let v = trimmed.trim_start_matches(is_ktav_whitespace);
            let value_abs = off_in_line(trimmed, seg_base, v);
            let mut n = inline_child(v, value_abs, line_num, span, depth, scan)?;
            n.slot = Some(Slot::Item {
                start: seg_base,
                body: (value_abs, seg_end),
            });
            items.push(n);
        }
        Ok(Node {
            kind: Kind::Array(items),
            inline: true,
            slot: None,
            root_range: None,
        })
    }
}

/// Значение inline-пары/элемента: вложенный compound — рекурсия,
/// скаляр — позиционируемый узел без слота (его навешивает родитель).
/// Скаляр помечается inline=true — ФИЗИЧЕСКИЙ контекст значения живёт
/// на самом листе (синтетический dotted-предок и reopen-потомки не
/// наследуют и не портят его).
fn inline_child(
    v: &str,
    v_abs: u32,
    line_num: usize,
    span: Span,
    depth: usize,
    scan: InlineScanCtx<'_>,
) -> Result<Node, Error> {
    match v.as_bytes().first() {
        Some(b'{') => position_inline(v, v_abs, line_num, span, true, depth + 1, scan),
        Some(b'[') => position_inline(v, v_abs, line_num, span, false, depth + 1, scan),
        _ => Ok(Node {
            kind: Kind::Scalar,
            inline: true,
            slot: None,
            root_range: None,
        }),
    }
}

/// Построить позиционное дерево документа. Ошибки здесь означают
/// расхождение с реальным парсером; вызывающая сторона отображает их
/// в типизированную ошибку (после успешного `parse(text)` они
/// недостижимы).
#[cfg(test)]
pub(crate) fn build(text: &str) -> Result<Node, Error> {
    build_with_meta(text).map(|(n, _)| n)
}

/// То же с флагом «корень явный» (открыт `{`/`[` строкой-Opener):
/// нужен сплайсу, чтобы отличать неявный root-array dispatch (§ 5.0.1
/// применяется только к неявному корню) от явного.
pub(crate) fn build_with_meta(text: &str) -> Result<(Node, bool), Error> {
    let mut b = Builder {
        stack: Vec::with_capacity(8),
        collecting: None,
        collecting_opener: 0,
        root_initialized: false,
        root_consumed: false,
        root_inline: None,
        root_is_explicit: false,
    };
    let bytes = text.as_bytes();
    // § 3.1: пропустить ровно один ведущий U+FEFF; офсеты — в исходных
    // координатах, BOM сохраняется в выходе сам собой (сплайс его не
    // трогает).
    let mut line_start = crate::parser::leading_bom_len(text);
    let mut line_num: usize = 0;
    if memchr(b'\r', bytes).is_none() {
        while line_start <= bytes.len() {
            let end = memchr(b'\n', &bytes[line_start..])
                .map(|p| line_start + p)
                .unwrap_or(bytes.len());
            line_num += 1;
            b.handle_line(&text[line_start..end], line_num, line_start as u32)?;
            if end == bytes.len() {
                break;
            }
            line_start = end + 1;
        }
    } else {
        while line_start < bytes.len() {
            let pos = memchr2(b'\n', b'\r', &bytes[line_start..])
                .map(|p| line_start + p)
                .unwrap_or(bytes.len());
            line_num += 1;
            b.handle_line(&text[line_start..pos], line_num, line_start as u32)?;
            line_start = if pos < bytes.len() {
                if bytes[pos] == b'\r' && bytes.get(pos + 1) == Some(&b'\n') {
                    pos + 2
                } else {
                    pos + 1
                }
            } else {
                break;
            };
        }
    }
    let root_explicit = b.root_is_explicit;
    b.finish(text.len() as u32).map(|n| (n, root_explicit))
}

impl<'a> Builder<'a> {
    fn handle_line(&mut self, raw: &'a str, line_num: usize, line_start: u32) -> Result<(), Error> {
        if let Some(mode) = self.collecting.take() {
            let trimmed = raw.trim_matches(is_ktav_whitespace);
            if trimmed == mode.terminator() {
                let opener = self.collecting_opener;
                // Терминатор `)`/`))` может нести ведущий отступ — его
                // конец считаем от фактического положения trimmed в raw.
                let end = off_in_line(raw, line_start, trimmed) + trimmed.len() as u32;
                let node = Node::scalar(Slot::Block {
                    anchor: None,
                    start: opener,
                    end,
                });
                return self.attach_scalar(node, line_num, line_start);
            }
            self.collecting = Some(mode);
            return Ok(());
        }

        let trimmed = raw.trim_matches(is_ktav_whitespace);
        if trimmed.is_empty() || trimmed.starts_with("##") {
            return Ok(());
        }
        let ts = trimmed_span_in(raw, trimmed, line_start);

        if self.root_consumed {
            return Err(Error::Structured(
                ErrorKind::OrphanLineAfterTopLevelInline {
                    line: line_num as u32,
                    span: ts,
                },
            ));
        }

        if !self.root_initialized {
            self.root_initialized = true;
            if trimmed != "}" && trimmed != "]" {
                match classify_root_shape(trimmed) {
                    RootShape::InlineObject | RootShape::InlineArray => {
                        self.root_consumed = true;
                        // Позиционируем inline-корень: тело — вся
                        // trimmed-строка, база — ts.start.
                        let is_obj = trimmed.starts_with('{');
                        let mut node =
                            position_inline_top(trimmed, ts.start, line_num, ts, is_obj)?;
                        node.root_range = Some((ts.start, ts.end));
                        self.root_inline = Some(node);
                        return Ok(());
                    }
                    RootShape::ExplicitObject => {
                        self.root_is_explicit = true;
                        self.push_frame(
                            Frame::Object {
                                table: ObjTable::default(),
                                pending_key: None,
                                pending_span: None,
                                pending_anchor: None,
                                range_start: ts.start,
                            },
                            ts.start,
                        );
                        return Ok(());
                    }
                    RootShape::ExplicitArray => {
                        self.root_is_explicit = true;
                        self.push_frame(
                            Frame::Array {
                                items: Vec::new(),
                                range_start: ts.start,
                            },
                            ts.start,
                        );
                        return Ok(());
                    }
                    RootShape::Object => {
                        self.push_frame(
                            Frame::Object {
                                table: ObjTable::default(),
                                pending_key: None,
                                pending_span: None,
                                pending_anchor: None,
                                range_start: 0,
                            },
                            0,
                        );
                    }
                    RootShape::Array => {
                        self.push_frame(
                            Frame::Array {
                                items: Vec::new(),
                                range_start: 0,
                            },
                            0,
                        );
                    }
                }
            }
        }

        if trimmed == "}" {
            return self.close_frame(Bracket::Object, line_num, ts);
        }
        if trimmed == "]" {
            return self.close_frame(Bracket::Array, line_num, ts);
        }

        if matches!(self.stack.last(), Some(Frame::Array { .. })) {
            self.handle_array_item(trimmed, line_num, ts)
        } else {
            self.handle_object_pair(trimmed, line_num, line_start, ts)
        }
    }

    fn push_frame(&mut self, frame: Frame<'a>, _opener: u32) {
        // Офсет opener'а хранится в самом фрейме (range_start).
        self.stack.push(frame);
    }

    /// `line` — УЖЕ trimmed-строка; все офсеты внутри неё считаются от
    /// `ts.start` (начало trimmed в исходнике), не от начала raw.
    fn handle_object_pair(
        &mut self,
        line: &'a str,
        line_num: usize,
        line_start: u32,
        ts: Span,
    ) -> Result<(), Error> {
        let colon = match scan_unescaped_colon(line) {
            ColonScan::Found(c) => c,
            ColonScan::UnterminatedQuote => {
                return Err(Error::Structured(ErrorKind::UnterminatedQuotedKey {
                    line: line_num as u32,
                    span: ts,
                }));
            }
            ColonScan::Absent => {
                return Err(Error::Structured(ErrorKind::MissingSeparator {
                    line: line_num as u32,
                    span: ts,
                }));
            }
        };
        let key = line[..colon].trim_end_matches(is_ktav_whitespace);
        let key_span = Span::new(ts.start, ts.start + key.len() as u32);
        let colon_off = ts.start + colon as u32;
        let after_colon = &line[colon + 1..];

        match classify_separator(after_colon) {
            Separator::Raw(after) => {
                // body_off — абсолютный офсет первого байта после `::`.
                require_sep_end(after, line_num, line_start, colon as u32, colon_off + 2, ts)?;
                let body = after.trim_matches(is_ktav_whitespace);
                let node = Node::scalar(Slot::Pair {
                    colon: colon_off,
                    body: (off_in_line(line, ts.start, body), ts.end),
                });
                self.insert_pair(key, node, line_num, key_span)
            }
            Separator::Plain(after) => {
                require_sep_end(after, line_num, line_start, colon as u32, colon_off + 1, ts)?;
                let after_trimmed = after.trim_start_matches(is_ktav_whitespace);
                let value_start = off_in_line(line, ts.start, after_trimmed);
                // Один classify на ветку (двойной вызов — дефект).
                let vs = classify_value_shape(after);
                match &vs {
                    ValueShape::Scalar | ValueShape::EmptyString => {
                        let node = Node::scalar(Slot::Pair {
                            colon: colon_off,
                            body: (value_start, ts.end),
                        });
                        self.insert_pair(key, node, line_num, key_span)
                    }
                    ValueShape::EmptyObject => self.insert_pair(
                        key,
                        Node::empty_object(Slot::Pair {
                            colon: colon_off,
                            body: (value_start, ts.end),
                        }),
                        line_num,
                        key_span,
                    ),
                    ValueShape::EmptyArray => self.insert_pair(
                        key,
                        Node::empty_array(Slot::Pair {
                            colon: colon_off,
                            body: (value_start, ts.end),
                        }),
                        line_num,
                        key_span,
                    ),
                    ValueShape::OpenObject | ValueShape::OpenArray => {
                        let is_obj = matches!(vs, ValueShape::OpenObject);
                        self.set_pending(key, key_span, Some(colon_off))?;
                        let frame = if is_obj {
                            Frame::Object {
                                table: ObjTable::default(),
                                pending_key: None,
                                pending_span: None,
                                pending_anchor: None,
                                range_start: value_start,
                            }
                        } else {
                            Frame::Array {
                                items: Vec::new(),
                                range_start: value_start,
                            }
                        };
                        self.stack.push(frame);
                        Ok(())
                    }
                    ValueShape::OpenMultilineStripped => {
                        self.set_pending(key, key_span, Some(colon_off))?;
                        self.collecting = Some(MultilineMode::Stripped);
                        self.collecting_opener = value_start;
                        Ok(())
                    }
                    ValueShape::OpenMultilineVerbatim => {
                        self.set_pending(key, key_span, Some(colon_off))?;
                        self.collecting = Some(MultilineMode::Verbatim);
                        self.collecting_opener = value_start;
                        Ok(())
                    }
                    ValueShape::InlineValue => {
                        // Тело inline — срез trimmed-строки от значения до
                        // её конца; начинается `{`/`[`.
                        let is_obj = after_trimmed.starts_with('{');
                        let mut node =
                            position_inline_top(after_trimmed, value_start, line_num, ts, is_obj)?;
                        // Сам контейнер физически лежит на СТРОКЕ пары
                        // (дети остаются inline=true).
                        node.inline = false;
                        node.slot = Some(Slot::Pair {
                            colon: colon_off,
                            body: (value_start, ts.end),
                        });
                        self.insert_pair(key, node, line_num, key_span)
                    }
                }
            }
        }
    }

    /// Для массивов `line_start = ts.start` (trimmed — вся строка).
    fn handle_array_item(&mut self, line: &'a str, line_num: usize, ts: Span) -> Result<(), Error> {
        let line_start = ts.start;

        if let Some(rest) = line.strip_prefix("::") {
            require_sep_end(rest, line_num, line_start, 1, line_start + 2, ts)?;
            let body = rest.trim_start_matches(is_ktav_whitespace);
            let node = Node::scalar(Slot::Item {
                start: ts.start,
                body: (off_in_line(line, line_start, body), ts.end),
            });
            return self.push_item(node);
        }

        let vs = classify_value_shape(line);
        let body_start = off_in_line(
            line,
            line_start,
            line.trim_start_matches(is_ktav_whitespace),
        );
        match vs {
            ValueShape::Scalar | ValueShape::EmptyString => {
                self.push_item(Node::scalar(Slot::Item {
                    start: ts.start,
                    body: (body_start, ts.end),
                }))
            }
            ValueShape::EmptyObject => self.push_item(Node::empty_object(Slot::Item {
                start: ts.start,
                body: (body_start, ts.end),
            })),
            ValueShape::EmptyArray => self.push_item(Node::empty_array(Slot::Item {
                start: ts.start,
                body: (body_start, ts.end),
            })),
            ValueShape::OpenObject => {
                self.stack.push(Frame::Object {
                    table: ObjTable::default(),
                    pending_key: None,
                    pending_span: None,
                    pending_anchor: None,
                    range_start: body_start,
                });
                Ok(())
            }
            ValueShape::OpenArray => {
                self.stack.push(Frame::Array {
                    items: Vec::new(),
                    range_start: body_start,
                });
                Ok(())
            }
            ValueShape::OpenMultilineStripped => {
                self.collecting = Some(MultilineMode::Stripped);
                self.collecting_opener = body_start;
                Ok(())
            }
            ValueShape::OpenMultilineVerbatim => {
                self.collecting = Some(MultilineMode::Verbatim);
                self.collecting_opener = body_start;
                Ok(())
            }
            ValueShape::InlineValue => {
                let is_obj = line.trim_start_matches(is_ktav_whitespace).starts_with('{');
                let mut node = position_inline_top(
                    line.trim_start_matches(is_ktav_whitespace),
                    body_start,
                    line_num,
                    ts,
                    is_obj,
                )?;
                node.inline = false;
                node.slot = Some(Slot::Item {
                    start: ts.start,
                    body: (body_start, ts.end),
                });
                self.push_item(node)
            }
        }
    }

    fn set_pending(
        &mut self,
        key: &'a str,
        key_span: Span,
        anchor: Option<u32>,
    ) -> Result<(), Error> {
        match self.stack.last_mut() {
            Some(Frame::Object {
                pending_key,
                pending_span,
                pending_anchor,
                ..
            }) => {
                *pending_key = Some(key);
                *pending_span = Some(key_span);
                *pending_anchor = anchor;
                Ok(())
            }
            _ => Err(Error::Structured(ErrorKind::Other {
                line: None,
                message: "internal: pending key outside object frame".into(),
                span: key_span,
            })),
        }
    }

    fn insert_pair(
        &mut self,
        key: &'a str,
        value: Node,
        line_num: usize,
        key_span: Span,
    ) -> Result<(), Error> {
        match self.stack.last_mut() {
            Some(Frame::Object { table, .. }) => {
                insert_value(table, key, value, line_num, key_span)
            }
            _ => Err(Error::Structured(ErrorKind::Other {
                line: None,
                message: "internal: pair outside object frame".into(),
                span: key_span,
            })),
        }
    }

    fn push_item(&mut self, value: Node) -> Result<(), Error> {
        match self.stack.last_mut() {
            Some(Frame::Array { items, .. }) => {
                items.push(value);
                Ok(())
            }
            _ => Err(Error::Structured(ErrorKind::Other {
                line: None,
                message: "internal: item outside array frame".into(),
                span: Span::EMPTY,
            })),
        }
    }

    fn attach_scalar(&mut self, node: Node, line_num: usize, line_start: u32) -> Result<(), Error> {
        match self.stack.last_mut() {
            Some(Frame::Object {
                table,
                pending_key,
                pending_span,
                pending_anchor,
                ..
            }) => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: "internal: multiline closed without pending key".into(),
                        span: Span::new(line_start, line_start),
                    })
                })?;
                let key_span = pending_span.take().unwrap_or(Span::EMPTY);
                let anchor = pending_anchor.take();
                // Перепривязать anchor к узлу блока.
                let node = match node.slot {
                    Some(Slot::Block { start, end, .. }) => {
                        Node::scalar(Slot::Block { anchor, start, end })
                    }
                    _ => node,
                };
                insert_value(table, key, node, line_num, key_span)
            }
            Some(Frame::Array { items, .. }) => {
                items.push(node);
                Ok(())
            }
            None => Err(Error::Structured(ErrorKind::Other {
                line: Some(line_num as u32),
                message: "internal: attach without frame".into(),
                span: Span::EMPTY,
            })),
        }
    }

    fn close_frame(&mut self, expected: Bracket, line_num: usize, ts: Span) -> Result<(), Error> {
        if self.stack.len() == 1 && self.root_is_explicit {
            self.root_consumed = true;
            return Ok(());
        }
        if self.stack.len() <= 1 {
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: ts,
                expected: match expected {
                    Bracket::Object => CompoundKind::Object,
                    Bracket::Array => CompoundKind::Array,
                },
                found: match expected {
                    Bracket::Object => '}',
                    Bracket::Array => ']',
                },
            }));
        }
        let frame = self.stack.pop().unwrap();
        let (kind, range_start) = match frame {
            Frame::Object {
                table, range_start, ..
            } => (Kind::Object(table), range_start),
            Frame::Array { items, range_start } => (Kind::Array(items), range_start),
        };
        // anchor берём у родителя-объекта (открывшая пара).
        let anchor = match self.stack.last_mut() {
            Some(Frame::Object { pending_anchor, .. }) => pending_anchor.take(),
            _ => None,
        };
        let node = Node {
            kind,
            inline: false,
            slot: Some(Slot::Block {
                anchor,
                start: range_start,
                end: ts.end,
            }),
            root_range: None,
        };
        match self.stack.last_mut() {
            Some(Frame::Object {
                table,
                pending_key,
                pending_span,
                ..
            }) => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: "internal: closed compound without pending key".into(),
                        span: ts,
                    })
                })?;
                let key_span = pending_span.take().unwrap_or(ts);
                insert_value(table, key, node, line_num, key_span)
            }
            Some(Frame::Array { items, .. }) => {
                items.push(node);
                Ok(())
            }
            None => Err(Error::Structured(ErrorKind::Other {
                line: None,
                message: "internal: close without parent frame".into(),
                span: ts,
            })),
        }
    }

    fn finish(mut self, eof_offset: u32) -> Result<Node, Error> {
        if self.collecting.is_some() {
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind: CompoundKind::MultilineVerbatim,
                line: 1,
                span: Span::new(0, eof_offset),
            }));
        }
        if self.stack.len() > 1 {
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind: CompoundKind::Object,
                line: 1,
                span: Span::new(0, eof_offset),
            }));
        }
        if self.root_is_explicit && !self.root_consumed {
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind: CompoundKind::Object,
                line: 1,
                span: Span::new(0, eof_offset),
            }));
        }
        if let Some(root) = self.root_inline.take() {
            return Ok(root);
        }
        match self.stack.pop() {
            Some(frame) => Ok(match frame {
                Frame::Object { table, .. } => Node {
                    kind: Kind::Object(table),
                    inline: false,
                    slot: None,
                    root_range: None,
                },
                Frame::Array { items, .. } => Node {
                    kind: Kind::Array(items),
                    inline: false,
                    slot: None,
                    root_range: None,
                },
            }),
            // Пустой / комментарийный документ — пустой объект-корень.
            None => Ok(Node {
                kind: Kind::Object(ObjTable::default()),
                inline: false,
                slot: None,
                root_range: None,
            }),
        }
    }
}
