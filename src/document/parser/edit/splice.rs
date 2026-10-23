//! Source-splice engine for the public `ktav::edit`: замена только
//! байтов целевого значения по диапазонам позиционного дерева
//! (`edit::tree`), без reparse→reserialize всего документа. Маркер
//! `: / ::` и разделяющие пробелы сохраняются/минимально корректируются
//! (контекст — по СТАРОМУ слоту, рендер — по НОВОМУ значению); потери
//! комментариев внутри структурно заменяемого контейнера — typed
//! `Unsupported` ДО сплайса; совпадающее значение — byte-identical
//! no-op. После патча — обязательная семантическая верификация.

use std::fmt::Write as _;

use crate::edit::EditError;
use crate::render::helpers::{
    bare_item_is_pair_candidate, item_needs_raw_marker, needs_raw_marker, push_escaped_key_segment,
};
use crate::value::Value;
use crate::whitespace::is_ktav_whitespace;

use super::tree::{self, Kind, Node, Slot};

/// Как значение лежит в исходной строке; определяет маркерную зону.
#[derive(Clone, Copy, PartialEq)]
enum SlotKind {
    /// Значение пары: первый `:` — разделитель ключа (неизменяемый),
    /// второй `:` (при raw) — переключаемый маркер.
    Pair,
    /// Элемент массива своей строкой: `::` (при raw) — маркер строки.
    Item,
    /// Multiline-блок: сплайс от opener до closer. Роль (pair/item)
    /// определяется независимо — по наличию открывшей пары (anchor).
    Block,
}

/// Контекст цели, влияющий на правила маркера/кодирования.
#[derive(Clone, Copy)]
struct TargetCtx {
    /// ФИЗИЧЕСКИ inline-значение (лист внутри `{ ... }` / `[ ... ]`;
    /// хранится на самом листе — dotted-предки и reopen не наследуют).
    inline: bool,
    /// Цель — элемент 0 НЕЯВНОГО корня-массива (подвержен § 5.0.1
    /// root-dispatch; явный `[ ... ]`-корень детерминирован скобками).
    root_first: bool,
}

/// Параметры плана замены (собраны в структуру, чтобы сигнатуры
/// планировщиков не разрастались).
#[derive(Clone, Copy)]
struct PlanCtx<'t> {
    slot: SlotKind,
    /// Block-цель — элемент массива (anchor отсутствует), а не значение
    /// пары; pair-роль маркер есть всегда (хотя бы `:`).
    role_item_block: bool,
    existing_raw: bool,
    root_first: bool,
    indent: &'t str,
    eol: &'t str,
}

/// Тело вставки: заимствованное (scalar literal, дословное String) или
/// владеющее (encoded inline, compound, verbatim) — без копии replacement.
enum Body<'r> {
    Borrowed(&'r str),
    Owned(String),
}

impl Body<'_> {
    fn as_str(&self) -> &str {
        match self {
            Body::Borrowed(s) => s,
            Body::Owned(s) => s,
        }
    }

    fn len(&self) -> usize {
        self.as_str().len()
    }
}

/// Компактный план сплайса: заимствованные куски маркерной зоны +
/// [`Body`]; строка собирается один раз, без промежуточных format!.
struct SplicePlan<'t, 'r> {
    from: usize,
    /// Куски перед телом (второй `:` / исходные пробелы маркерной зоны).
    prefix: [&'t str; 2],
    body: Body<'r>,
}

pub(crate) fn apply<S: AsRef<str>>(
    text: &str,
    segments: &[S],
    replacement: &Value,
) -> Result<String, EditError> {
    // Границы u32-офсетов позиционного дерева — до любых кастов.
    if text.len() > u32::MAX as usize {
        return Err(EditError::Unsupported {
            reason: "document exceeds 4 GiB; offsets do not fit u32".to_string(),
        });
    }
    // Оригинал обязан парситься ДО позиционного прохода: (а) дерево
    // строится только для валидных документов, (б) значение оригинала
    // переиспользуется верификацией без второго parse.
    let original_value = crate::parse(text).map_err(|e| EditError::Unsupported {
        reason: format!("source text does not parse: {e}"),
    })?;
    let target = lookup_value(&original_value, segments).ok_or(EditError::NotFound)?;
    if target == replacement {
        return Ok(text.to_string());
    }
    let (root, root_explicit) =
        tree::build_with_meta(text).map_err(|e| EditError::Unsupported {
            reason: format!("source text does not parse: {e}"),
        })?;
    let (node, ctx) = descend(&root, segments, root_explicit)?;

    let slot = node.slot().ok_or_else(|| EditError::Unsupported {
        reason: "target is a noncontiguous value (dotted-key/reopen intermediate); \
                 no lossless single-range splice exists"
            .to_string(),
    })?;

    let slot_kind = match slot {
        Slot::Pair { .. } => SlotKind::Pair,
        Slot::Item { .. } => SlotKind::Item,
        Slot::Block { .. } => SlotKind::Block,
    };

    // Тело значения и якорь маркерной зоны; для Block — роль
    // (anchor есть ⇒ значение пары, нет ⇒ элемент массива).
    let (body, anchor, block_item_role): ((u32, u32), u32, bool) = match slot {
        Slot::Pair { colon, body } => (body, colon, false),
        Slot::Item { start, body } => (body, start, false),
        Slot::Block { start, end, anchor } => {
            ((start, end), anchor.unwrap_or(start), anchor.is_none())
        }
    };
    let (b0, b1) = (body.0 as usize, body.1 as usize);
    let a = anchor as usize;
    // Все производные диапазоны валидируются .get() до срезов —
    // невалидная разметка даёт typed Err, а не bounds panic.
    if text.get(a..b1).is_none() || b0 > b1 {
        return Err(EditError::Unsupported {
            reason: "internal: invalid marker/body offsets".to_string(),
        });
    }
    // Маркерная зона: существующий raw, исходные пробелы между
    // маркером и телом (данные, сохраняются). Физически inline-элемент
    // массива маркера НЕ имеет (`::` — literal-данные) — не читаем.
    let inline_item = ctx.inline && slot_kind == SlotKind::Item;
    let (existing_raw, ws_raw, ws_plain) = match slot_kind {
        SlotKind::Block => (false, "", ""),
        _ if inline_item => (false, "", ""),
        SlotKind::Pair | SlotKind::Item => {
            let raw = text.as_bytes().get(a + 1) == Some(&b':');
            let ws_raw = if raw {
                text.get(a + 2..b0).ok_or_else(|| EditError::Unsupported {
                    reason: "internal: invalid raw marker range".to_string(),
                })?
            } else {
                ""
            };
            let ws_plain = match slot_kind {
                SlotKind::Pair => text.get(a + 1..b0).ok_or_else(|| EditError::Unsupported {
                    reason: "internal: invalid plain marker range".to_string(),
                })?,
                _ => "",
            };
            (raw, ws_raw, ws_plain)
        }
    };

    // Структурная перезапись контейнера: потеря комментариев внутри
    // цели недопустима — typed отказ до сплайса. Multiline-блоки
    // скаляров исключаются: их содержимое — данные.
    if matches!(node.kind, Kind::Object(_) | Kind::Array(_)) {
        let mut ranges = Vec::new();
        collect_multiline_scalar_ranges(node, &mut ranges);
        if has_comment_line(text, b0, b1, &mut ranges) {
            return Err(EditError::Unsupported {
                reason: "target container contains comment lines; structural \
                         replacement cannot preserve them losslessly"
                    .to_string(),
            });
        }
    }

    let indent = line_indent(text, a);
    let eol = detect_eol(text, b1);
    let pc = PlanCtx {
        slot: slot_kind,
        // Block-роль: multiline-блок без открывшей пары — элемент массива.
        role_item_block: slot_kind == SlotKind::Block && block_item_role,
        existing_raw,
        root_first: ctx.root_first,
        indent: &indent,
        eol,
    };
    // WS пары для Block-цели с pair-ролью; Item-блок якоря-разделителя
    // не имеет (anchor==b0) — диапазон был бы reversed.
    let ws_pair_block = if slot_kind != SlotKind::Block || block_item_role {
        "".to_string()
    } else {
        text.get(a + 1..b0)
            .ok_or_else(|| EditError::Unsupported {
                reason: "internal: invalid block pair marker range".to_string(),
            })?
            .to_string()
    };

    let plan: SplicePlan<'_, '_> = if ctx.inline {
        // Физически inline.
        match (slot_kind, replacement) {
            // Inline-элемент: `::` — literal-данные, маркера нет; тело
            // целиком с типосохраняющим кодированием; пустая строка —
            // `()` (голая пустая строка стала бы blank-линией).
            (SlotKind::Item, Value::String(s)) if s.is_empty() => SplicePlan {
                from: b0,
                prefix: ["", ""],
                body: Body::Borrowed("()"),
            },
            (SlotKind::Item, _) => SplicePlan {
                from: b0,
                prefix: ["", ""],
                body: inline_body(replacement, true)?,
            },
            // Inline-пара с не-String заменой при существующем raw:
            // raw-тело literal — контейнер не выражаем; снять только
            // ВТОРОЙ `:`, исходные пробелы сохранить.
            (SlotKind::Pair, value) if existing_raw && !matches!(value, Value::String(_)) => {
                SplicePlan {
                    from: a + 1,
                    prefix: [ws_raw, ""],
                    body: inline_body(replacement, false)?,
                }
            }
            // Остальное: маркер не меняется; encoder обеспечивает
            // String-провенанс при необходимости (см. encode_inline).
            _ => SplicePlan {
                from: b0,
                prefix: ["", ""],
                body: inline_body(replacement, !existing_raw)?,
            },
        }
    } else if slot_kind == SlotKind::Block {
        block_replacement(replacement, &pc, a, b0, &ws_pair_block)?
    } else {
        // Линейные слоты: план (тело + желаемый маркер), затем
        // минимальный маркерный переход.
        let (body, desired_raw) = plan_value(replacement, &pc)?;
        match (existing_raw, desired_raw) {
            // Маркер верен — меняется только тело (b0 уже после
            // исходных пробелов; пустое тело — один пробел-разделитель).
            (true, true) | (false, false) => {
                let prefix = if b0 == b1 && ws_plain.is_empty() && ws_raw.is_empty() {
                    " "
                } else {
                    ""
                };
                SplicePlan {
                    from: b0,
                    prefix: [prefix, ""],
                    body,
                }
            }
            // raw → plain:
            //   Pair: снять только ВТОРОЙ `:` ('k:: x' → 'k: x'),
            //   пробелы после `::` сохраняются.
            //   Item: убрать `::` целиком, СОХРАНИВ исходный WS между
            //   маркером и телом (он остаётся ведущим после отступа).
            (true, false) if slot_kind == SlotKind::Pair => SplicePlan {
                from: a + 1,
                prefix: [sep_ws(b0, b1, ws_raw), ""],
                body,
            },
            (true, false) => SplicePlan {
                from: a,
                prefix: [ws_raw, ""],
                body,
            },
            // plain → raw:
            //   Pair: добавить только ВТОРОЙ `:` за первым ('k: 5' →
            //   'k:: 42'), исходные пробелы — как были.
            //   Item: вставить `:: ` перед телом (anchor == b0).
            (false, true) if slot_kind == SlotKind::Pair => SplicePlan {
                from: a + 1,
                prefix: [":", sep_ws(b0, b1, ws_plain)],
                body,
            },
            (false, true) => SplicePlan {
                from: a,
                prefix: [":: ", ""],
                body,
            },
        }
    };
    let from = plan.from;
    if text.get(from..b1).is_none() {
        return Err(EditError::Unsupported {
            reason: "internal: splice range out of bounds".to_string(),
        });
    }

    let insertion_len = plan
        .prefix
        .iter()
        .map(|piece| piece.len())
        .try_fold(plan.body.len(), usize::checked_add)
        .ok_or_else(|| EditError::Unsupported {
            reason: "patched document length overflows usize".to_string(),
        })?;
    let capacity = (text.len() - (b1 - from))
        .checked_add(insertion_len)
        .ok_or_else(|| EditError::Unsupported {
            reason: "patched document length overflows usize".to_string(),
        })?;
    let mut result = String::with_capacity(capacity);
    result.push_str(&text[..from]);
    result.push_str(plan.prefix[0]);
    result.push_str(plan.prefix[1]);
    result.push_str(plan.body.as_str());
    result.push_str(&text[b1..]);

    // Цель сравнивается отдельно; остальное дерево — с одинаковыми Null-слотами.
    let mut new_doc = crate::parse(&result).map_err(|e| EditError::Unsupported {
        reason: format!("patch produced an invalid document: {e}"),
    })?;
    let target_matches = take_target(&mut new_doc, segments)
        .map(|actual| actual == *replacement)
        .unwrap_or(false);
    let mut expected = original_value;
    drop(take_target(&mut expected, segments)?);
    if !target_matches || new_doc != expected {
        return Err(EditError::Unsupported {
            reason: "semantic mismatch after splice; no lossless patch produced".into(),
        });
    }
    Ok(result)
}

/// Маркерная зона ветки с неизменным маркером: " " на пустом теле
/// источника, иначе исходный WS — без копий.
fn sep_ws(b0: usize, b1: usize, ws: &str) -> &str {
    if b0 == b1 && ws.is_empty() {
        " "
    } else {
        ws
    }
}

/// Спуск по декодированным сегментам позиционного дерева: в Object
/// сегмент — строковый ключ, в Array — zero-based индекс. Контекст
/// берётся с самого целевого листа (физический inline) и из факта
/// «последний шаг — элемент 0 неявного корня-массива».
fn descend<'n, S: AsRef<str>>(
    root: &'n Node,
    segments: &[S],
    root_explicit: bool,
) -> Result<(&'n Node, TargetCtx), EditError> {
    let mut container_is_root = true;
    let mut cur: &Node = root;
    let mut root_first = false;
    for seg in segments {
        let seg = seg.as_ref();
        cur = match &cur.kind {
            Kind::Object(_) => {
                root_first = false;
                cur.child(seg).ok_or(EditError::NotFound)?
            }
            Kind::Array(_) => {
                let idx = parse_index(seg)?;
                root_first = container_is_root && !root_explicit && idx == 0;
                cur.item(idx).ok_or(EditError::NotFound)?
            }
            Kind::Scalar => return Err(EditError::NotFound),
        };
        container_is_root = false;
    }
    Ok((
        cur,
        TargetCtx {
            inline: cur.inline,
            root_first,
        },
    ))
}

/// Значение цели в распарсенном оригинале (для no-op и верификации).
fn lookup_value<'a, S: AsRef<str>>(v: &'a Value, segments: &[S]) -> Option<&'a Value> {
    let mut cur = v;
    for seg in segments {
        let seg = seg.as_ref();
        cur = match cur {
            Value::Object(o) => o.get(seg)?,
            Value::Array(a) => a.get(parse_index(seg).ok()?)?,
            _ => return None,
        };
    }
    Some(cur)
}

/// Извлечь цель из временного дерева без клонирования.
fn take_target<S: AsRef<str>>(v: &mut Value, segments: &[S]) -> Result<Value, EditError> {
    let (last, parents) = segments.split_last().ok_or(EditError::NotFound)?;
    let last = last.as_ref();
    let mut cur = v;
    for seg in parents {
        let seg = seg.as_ref();
        cur = match cur {
            Value::Object(o) => o.get_mut(seg).ok_or(EditError::NotFound)?,
            Value::Array(a) => {
                let idx = parse_index(seg)?;
                a.get_mut(idx).ok_or(EditError::NotFound)?
            }
            _ => return Err(EditError::NotFound),
        };
    }
    let slot = match cur {
        Value::Object(o) => o.get_mut(last).ok_or(EditError::NotFound)?,
        Value::Array(a) => {
            let idx = parse_index(last)?;
            a.get_mut(idx).ok_or(EditError::NotFound)?
        }
        _ => return Err(EditError::NotFound),
    };
    Ok(std::mem::replace(slot, Value::Null))
}

fn parse_index(seg: &str) -> Result<usize, EditError> {
    // Только ASCII-цифры: usize-парсер принимал бы '+'/знаки — вне
    // контракта. Ведущие нули допустимы ('01' == индекс 1).
    if seg.is_empty() || !seg.bytes().all(|b| b.is_ascii_digit()) {
        return Err(EditError::NotFound);
    }
    seg.parse::<usize>().map_err(|_| EditError::NotFound)
}

/// Все диапазоны multiline-скалярных блоков (данных, а не разметки)
/// в поддереве — исключаются из comment-скана.
fn collect_multiline_scalar_ranges(node: &Node, out: &mut Vec<(u32, u32)>) {
    if let (Kind::Scalar, Some(Slot::Block { start, end, .. })) = (&node.kind, node.slot) {
        out.push((start, end));
        return;
    }
    match &node.kind {
        Kind::Object(t) => {
            for (_, child) in t.entries.iter() {
                collect_multiline_scalar_ranges(child, out);
            }
        }
        Kind::Array(items) => {
            for item in items {
                collect_multiline_scalar_ranges(item, out);
            }
        }
        Kind::Scalar => {}
    }
}

/// Есть ли в диапазоне [from, to) строка, начинающаяся (после
/// § 3.3-whitespace) с `##`, вне multiline-скалярных диапазонов.
/// Диапазоны сортируются один раз, курсор движется монотонно —
/// O(lines + ranges), без квадрата.
fn has_comment_line(text: &str, from: usize, to: usize, skip: &mut [(u32, u32)]) -> bool {
    skip.sort_by_key(|r| r.0);
    let mut skip_i = 0usize;
    let bytes = text.as_bytes();
    let mut i = from;
    while i < to {
        let line_start = i;
        let mut j = line_start;
        while j < to && bytes[j] != b'\n' && bytes[j] != b'\r' {
            j += 1;
        }
        let line_end = j;
        // Первый не-§3.3-whitespace code point строки (Unicode
        // whitespace классифицируется тем же предикатом, что и в
        // парсере — не только ASCII space/tab).
        let line = &text[line_start..line_end];
        let trimmed = line.trim_start_matches(is_ktav_whitespace);
        let k = line_start + (trimmed.as_ptr() as usize - line.as_ptr() as usize);
        // Диапазоны данных, закончившиеся до этой строки, — вон.
        while skip_i < skip.len() && (skip[skip_i].1 as usize) <= k {
            skip_i += 1;
        }
        let in_data =
            skip_i < skip.len() && (skip[skip_i].0 as usize) <= k && k < (skip[skip_i].1 as usize);
        if !in_data && trimmed.starts_with("##") {
            return true;
        }
        // К терминатору следующей строки.
        i = j;
        while i < to && (bytes[i] == b'\n' || bytes[i] == b'\r') {
            i += 1;
        }
    }
    false
}

/// Тело нетривиального скаляра (только скалярные варианты); лексика
/// заимствуется у replacement.
fn scalar_body(v: &Value) -> Result<Body<'_>, EditError> {
    match v {
        Value::Null => Ok(Body::Borrowed("null")),
        Value::Bool(true) => Ok(Body::Borrowed("true")),
        Value::Bool(false) => Ok(Body::Borrowed("false")),
        Value::Integer(s) | Value::Float(s) => Ok(Body::Borrowed(s.as_str())),
        _ => Err(EditError::Unsupported {
            reason: "internal: scalar_body called on a compound value".to_string(),
        }),
    }
}

/// Физически inline-значение. `force_string` — для plain-маркера:
/// числоподобное/keyword-подобное тело обязано получить String-
/// провенанс (первый символ через \uXXXX).
fn inline_body(replacement: &Value, force_string: bool) -> Result<Body<'_>, EditError> {
    match replacement {
        Value::String(s) => Ok(Body::Owned(encode_inline(
            s,
            force_string && needs_raw_marker(s),
        ))),
        Value::Object(_) | Value::Array(_) => inline_compound_body(replacement, 0).map(Body::Owned),
        _ => scalar_body(replacement),
    }
}

/// § 3.7-кодирование inline-значения: все ломающие сплит/деку байты
/// (`\\` `\,` `\}` `\]` `\{` `\[`) и переводы строк (`\n` `\r`)
/// экранируются; краевые § 3.3-пробелы (кроме LF/CR) — `\uXXXX`,
/// поскольку физические края тела триммируются разбором. Под
/// plain-маркером (`force_string`) числоподобное/keyword-подобное
/// тело получает String-провенанс через `\uXXXX` первого символа
/// (escapes форсируют String; см. parse_inline_value_raw).
fn encode_inline(s: &str, force_string: bool) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    push_inline_string(s, force_string, &mut out);
    out
}

fn push_inline_string(s: &str, force_string: bool, out: &mut String) {
    let last = s.char_indices().next_back().map_or(0, |(index, _)| index);
    for (index, c) in s.char_indices() {
        if force_string && index == 0 {
            write!(out, "\\u{:04X}", c as u32).expect("writing to String cannot fail");
        } else {
            encode_inline_char(out, c, index, last);
        }
    }
}

fn encode_inline_char(out: &mut String, c: char, i: usize, last: usize) {
    match c {
        '\\' => out.push_str("\\\\"),
        ',' => out.push_str("\\,"),
        '}' => out.push_str("\\}"),
        ']' => out.push_str("\\]"),
        '{' => out.push_str("\\{"),
        '[' => out.push_str("\\["),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        _ if (i == 0 || i == last) && is_ktav_whitespace(c) => {
            // Все члены § 3.3 ≤ U+3000 — четырёх hex-цифр хватает
            // (§ 3.7.1, uppercase).
            write!(out, "\\u{:04X}", c as u32).expect("writing to String cannot fail");
        }
        _ => out.push(c),
    }
}

/// Inline-рендер контейнера одной строкой: `{k: v, ...}` / `[v1, v2]`;
/// ключи — через общий эмиттер § 5.9.10 (bare/quoted + re-escape),
/// значения — рекурсивно с тем же лимитом глубины, что и у парсера.
fn inline_compound_body(v: &Value, depth: usize) -> Result<String, EditError> {
    let mut out = String::new();
    push_inline_value(v, depth, &mut out)?;
    Ok(out)
}

fn push_inline_value(v: &Value, depth: usize, out: &mut String) -> Result<(), EditError> {
    if depth > crate::parser::inline::MAX_INLINE_DEPTH {
        return Err(EditError::Unsupported {
            reason: "replacement nesting exceeds inline depth limit (128)".to_string(),
        });
    }
    match v {
        Value::Object(o) => {
            out.push('{');
            for (index, (key, value)) in o.iter().enumerate() {
                if index != 0 {
                    out.push_str(", ");
                }
                push_escaped_key_segment(key, false, out);
                out.push_str(": ");
                match value {
                    Value::String(s) => push_inline_string(s, needs_raw_marker(s), out),
                    _ => push_inline_value(value, depth + 1, out)?,
                }
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (index, value) in a.iter().enumerate() {
                if index != 0 {
                    out.push_str(", ");
                }
                match value {
                    Value::String(s) if s.is_empty() => out.push_str("()"),
                    Value::String(s) => push_inline_string(s, needs_raw_marker(s), out),
                    _ => push_inline_value(value, depth + 1, out)?,
                }
            }
            out.push(']');
        }
        Value::String(s) => push_inline_string(s, needs_raw_marker(s), out),
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Integer(s) | Value::Float(s) => out.push_str(s),
    }
    Ok(())
}

/// План замены: (тело без маркерного префикса, нужен ли raw-маркер);
/// scalar/String-лексика без escapes заимствуется.
fn plan_value<'r>(replacement: &'r Value, pc: &PlanCtx) -> Result<(Body<'r>, bool), EditError> {
    match replacement {
        Value::String(s) => plan_string(s, pc),
        Value::Null | Value::Bool(_) | Value::Integer(_) | Value::Float(_) => {
            Ok((scalar_body(replacement)?, false))
        }
        Value::Object(o) if o.is_empty() => Ok((Body::Borrowed("{}"), false)),
        Value::Array(a) if a.is_empty() => Ok((Body::Borrowed("[]"), false)),
        Value::Object(_) | Value::Array(_) => {
            let body = render_compound_body(replacement)?;
            Ok((Body::Owned(reindent(&body, pc.indent, pc.eol)), false))
        }
    }
}

/// План String-замены для линейных слотов вне inline.
fn plan_string<'r>(s: &'r str, pc: &PlanCtx) -> Result<(Body<'r>, bool), EditError> {
    if s.contains('\r') {
        return Err(EditError::Unsupported {
            reason: "replacement string contains CR (U+000D)".to_string(),
        });
    }
    let edge_ws = s.chars().next().is_some_and(is_ktav_whitespace)
        || s.chars().last().is_some_and(is_ktav_whitespace);
    if s.contains('\n') || edge_ws {
        // plain-маркер обязателен: raw не открывает multiline-блок
        // ('k:: ((' — literal строка '((', а не блок) и триммирует края.
        return Ok((
            Body::Owned(verbatim_block_body(s, pc.indent, pc.eol)?),
            false,
        ));
    }
    let mut desired_raw = if pc.slot == SlotKind::Item {
        item_needs_raw_marker(s) || s.is_empty()
    } else {
        needs_raw_marker(s)
    };
    // § 5.0.1 root-dispatch: pair-подобное (и BOM-начинающее) тело
    // первого элемента неявного корня-массива обязано быть raw.
    if pc.slot == SlotKind::Item && pc.root_first {
        desired_raw = desired_raw || bare_item_is_pair_candidate(s) || s.starts_with('\u{FEFF}');
    }
    // Сохранение существующего raw: совместимое однострочное тело
    // оставляет маркер как есть («raw unnecessary» ≠ «raw forbidden»;
    // forbidden-случаи вышли раньше через early-return).
    if pc.existing_raw {
        desired_raw = true;
    }
    Ok((Body::Borrowed(s), desired_raw))
}

/// Verbatim-блок `(( ... ))`: контент дословно, '\n' → EOL документа,
/// финальный LF — пустая строка перед `))`. Один буфер, checked
/// capacity; для LF-документов контент копируется одним push_str.
fn verbatim_block_body(s: &str, indent: &str, eol: &str) -> Result<String, EditError> {
    for line in s.lines() {
        if line.trim_matches(|c: char| c == ' ' || c == '\t') == "))" {
            return Err(EditError::Unsupported {
                reason: "multiline string contains a line equal to '))'; \
                         no lossless verbatim form exists"
                    .to_string(),
            });
        }
    }
    // Расширение (1 байт '\n' → 2 байта eol) считается только для не-LF;
    // рамка: opener/closer + 2 eol + отступ.
    let newlines = if eol == "\n" {
        0
    } else {
        memchr::memchr_iter(b'\n', s.as_bytes()).count()
    };
    let extra_per_nl = eol.len() - 1;
    let capacity =
        s.len()
            .checked_add(newlines.checked_mul(extra_per_nl).ok_or_else(|| {
                EditError::Unsupported {
                    reason: "patched document length overflows usize".to_string(),
                }
            })?)
            .and_then(|n| n.checked_add(2 * eol.len()))
            .and_then(|n| n.checked_add(indent.len()))
            .and_then(|n| n.checked_add(4))
            .ok_or_else(|| EditError::Unsupported {
                reason: "patched document length overflows usize".to_string(),
            })?;
    let mut out = String::with_capacity(capacity);
    out.push_str("((");
    out.push_str(eol);
    if eol == "\n" {
        out.push_str(s);
    } else {
        // split('\n') сохраняет хвостовой LF пустой последней строкой.
        let mut first = true;
        for line in s.split('\n') {
            if !first {
                out.push_str(eol);
            }
            first = false;
            out.push_str(line);
        }
    }
    out.push_str(eol);
    out.push_str(indent);
    out.push_str("))");
    Ok(out)
}

/// Замена в multiline-БЛОКЕ; роль задаёт форму результата:
/// - pair-роль (открывшая пара есть): String — verbatim-блок при
///   multiline/краевых пробелах, иначе при необходимости raw —
///   переключение маркера (второй `:`) с телом;
///   не-String — канонический текст / переотмпунченный контейнер.
/// - item-роль (блок — элемент массива): пустая строка → `()`
///   (parsed Scalar("")), item-сырое `:: …` для нуждающихся, multiline
///   — verbatim-блок; root-dispatch первого элемента неявного корня
///   сохраняется.
fn block_replacement<'t, 'r>(
    replacement: &'r Value,
    pc: &PlanCtx<'t>,
    a: usize,
    b0: usize,
    ws_pair: &'t str,
) -> Result<SplicePlan<'t, 'r>, EditError> {
    match replacement {
        Value::String(s) => {
            if s.contains('\r') {
                return Err(EditError::Unsupported {
                    reason: "replacement string contains CR (U+000D)".to_string(),
                });
            }
            let multiline = s.contains('\n')
                || s.chars().next().is_some_and(is_ktav_whitespace)
                || s.chars().last().is_some_and(is_ktav_whitespace);
            if multiline {
                return Ok(SplicePlan {
                    from: b0,
                    prefix: ["", ""],
                    body: Body::Owned(verbatim_block_body(s, pc.indent, pc.eol)?),
                });
            }
            // Item-роль: пустая строка → `()` (парсится как Scalar(""),
            // голая пустая строка была бы blank-линией).
            if pc.role_item_block && s.is_empty() {
                return Ok(SplicePlan {
                    from: b0,
                    prefix: ["", ""],
                    body: Body::Borrowed("()"),
                });
            }
            let item_raw = pc.role_item_block
                && (item_needs_raw_marker(s)
                    || (pc.root_first
                        && (bare_item_is_pair_candidate(s) || s.starts_with('\u{FEFF}'))));
            let pair_raw = !pc.role_item_block && needs_raw_marker(s);
            if item_raw {
                Ok(SplicePlan {
                    from: b0,
                    prefix: [":: ", ""],
                    body: Body::Borrowed(s),
                })
            } else if pair_raw {
                // Сменить маркер пары на raw: второй `:` + исходные
                // пробелы + тело.
                Ok(SplicePlan {
                    from: a + 1,
                    prefix: [":", ws_pair],
                    body: Body::Borrowed(s),
                })
            } else {
                Ok(SplicePlan {
                    from: b0,
                    prefix: ["", ""],
                    body: Body::Borrowed(s),
                })
            }
        }
        _ => {
            let mut pc = *pc;
            pc.slot = SlotKind::Block;
            let (body, _) = plan_value(replacement, &pc)?;
            Ok(SplicePlan {
                from: b0,
                prefix: ["", ""],
                body,
            })
        }
    }
}

/// Pretty-рендер контейнера без копии Value и корневой обёртки.
fn render_compound_body(v: &Value) -> Result<String, EditError> {
    crate::render::render_compound_item(v).map_err(|e| EditError::Unsupported {
        reason: format!("replacement is not representable: {e}"),
    })
}

/// Переотступить pretty-рендер под строку цели. Строки ВНУТРИ
/// verbatim-блоков рендера (между строкой с хвостовым `((` и строкой
/// `))`) не переотступаются — их содержимое дословно. Эвристика
/// контекстно-зависима, но любой промах ловится обязательной
/// семантической верификацией (fail closed), тишина невозможна.
fn reindent(doc: &str, indent: &str, eol: &str) -> String {
    let mut out = String::with_capacity(doc.len());
    let mut in_verbatim = false;
    for (i, line) in doc.lines().enumerate() {
        let line = line.strip_prefix("    ").unwrap_or(line);
        let verbatim_open = line.ends_with("((");
        let verbatim_close = line.trim_matches(|c: char| c == ' ' || c == '\t') == "))";
        if i > 0 {
            out.push_str(eol);
            let skip_prefix = in_verbatim || line.is_empty();
            if !skip_prefix {
                out.push_str(indent);
            }
        }
        out.push_str(line);
        if verbatim_open {
            in_verbatim = true;
        } else if verbatim_close {
            in_verbatim = false;
        }
    }
    out
}

/// Отступ строки, содержащей офсет `pos` (ведущие ' '/'\t' строки).
fn line_indent(text: &str, pos: usize) -> String {
    let start = text[..pos].rfind(['\n', '\r']).map(|i| i + 1).unwrap_or(0);
    let bytes = text.as_bytes();
    let mut end = start;
    while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
        end += 1;
    }
    text[start..end].to_string()
}

/// EOL-стиль документа, определённый по первому терминатору строки
/// после `pos`; для файла без переводов строк — '\n'.
fn detect_eol(text: &str, pos: usize) -> &'static str {
    let bytes = text.as_bytes();
    let mut i = pos;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => return "\n",
            b'\r' => {
                return if bytes.get(i + 1) == Some(&b'\n') {
                    "\r\n"
                } else {
                    "\r"
                };
            }
            _ => i += 1,
        }
    }
    "\n"
}
