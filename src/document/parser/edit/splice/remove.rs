//! Удаление существующего листа/поддерева/элемента массива: диапазон
//! — целые строки цели (inline-сегмент — сегмент с соседней запятой).
//! Пустые предки не чистятся: удаление последнего dotted-листа
//! сохраняет родителя пустым контейнером (семантика `a: {}`).
//! Внутренние комментарии удаляемого контейнера — typed Unsupported.

use super::ops::{line_end_with_eol, line_start, node_at, path_get, unsupported, value_at, verify};
use super::tree::{self, Kind, Node, Slot};
use super::{collect_multiline_scalar_ranges, has_comment_line, parse_index};
use crate::edit::EditError;
use crate::error::Span;
use crate::parser::inline::{has_quote_bytes, split_top_level, InlineBody, InlineBounds};
use crate::render::helpers::push_escaped_key_segment;
use crate::value::Value;
use crate::whitespace::is_ktav_whitespace;

pub(super) fn apply<S: AsRef<str>>(text: &str, segments: &[S]) -> Result<String, EditError> {
    if text.len() > u32::MAX as usize {
        return Err(unsupported(
            "document exceeds 4 GiB; offsets do not fit u32",
        ));
    }
    if segments.is_empty() {
        return Err(EditError::NotFound);
    }
    let original =
        crate::parse(text).map_err(|e| unsupported(&format!("source text does not parse: {e}")))?;
    // Цель обязана существовать семантически.
    super::lookup_value(&original, segments).ok_or(EditError::NotFound)?;
    let (root, _) = tree::build_with_meta(text)
        .map_err(|e| unsupported(&format!("source text does not parse: {e}")))?;
    let node = node_at(&root, segments)?;
    let (from, to, stripped_lead) = removal_range(text, &root, segments, node)?;
    // Комментарии внутри удаляемого диапазона (вне multiline-данных
    // поддерева) терялись бы — typed отказ до сплайса.
    if matches!(node.kind, Kind::Object(_) | Kind::Array(_)) {
        let mut ranges = Vec::new();
        collect_multiline_scalar_ranges(node, &mut ranges);
        if has_comment_line(text, from, to, &mut ranges) {
            return Err(unsupported(
                "target contains comment lines; removal cannot preserve them losslessly",
            ));
        }
    }
    // Ожидаемое дерево: цель удалена у родителя; элемент массива
    // сдвигает хвост естественным Vec::remove. Пустые предки НЕ
    // удаляются (no-prune).
    let mut expected = original;
    let (last, parents) = segments.split_last().ok_or(EditError::NotFound)?;
    let parent = value_at(&mut expected, parents)?;
    let last = last.as_ref();
    match parent {
        Value::Object(o) => {
            if o.shift_remove(last).is_none() {
                return Err(EditError::NotFound);
            }
        }
        Value::Array(a) => {
            let idx = parse_index(last)?;
            if idx >= a.len() {
                return Err(EditError::NotFound);
            }
            a.remove(idx);
        }
        _ => return Err(EditError::NotFound),
    }
    // Материализация опустевшей цепочки dotted-предков: без неё
    // удаление единственного `a.b: 1` стёрло бы самого предка `a`,
    // а no-prune требует `a: {}`.
    let mut materialized: Option<String> = None;
    let ancestors: Vec<&str> = parents.iter().map(|s| s.as_ref()).collect();
    if !ancestors.is_empty() {
        let mut emptied_from = None;
        for start in 0..ancestors.len() {
            let all_empty = (start..ancestors.len()).all(|k| {
                matches!(
                    path_get(&expected, &ancestors[..=k]),
                    Some(Value::Object(o)) if o.is_empty()
                )
            });
            if all_empty {
                emptied_from = Some(start);
                break;
            }
        }
        if let Some(m) = emptied_from {
            // Внешний опустевший предок — синтетический (реальный
            // блок сохраняет фигурные скобки); путь от КОРНЯ.
            let synthetic = node_at(&root, &ancestors[..=m])
                .ok()
                .and_then(|n| n.slot())
                .is_none();
            // dotted-ключ отсчитывается от ближайшего реального
            // контейнера (последний slotted-предок), не от корня.
            let mut container_depth = 0usize;
            for i in 0..ancestors.len() {
                if node_at(&root, &ancestors[..=i])
                    .ok()
                    .and_then(|n| n.slot())
                    .is_some()
                {
                    container_depth = i + 1;
                }
            }
            if synthetic && m >= container_depth {
                let lead_len = if !stripped_lead {
                    0
                } else if text[from..].starts_with("\r\n") {
                    2
                } else {
                    1
                };
                let line_start_off = from + lead_len;
                let indent_end = text[line_start_off..]
                    .find(|c: char| !is_ktav_whitespace(c))
                    .map_or(line_start_off, |i| line_start_off + i);
                let indent = &text[line_start_off..indent_end];
                let mut key = String::new();
                for (i, seg) in ancestors[container_depth..].iter().enumerate() {
                    if i > 0 {
                        key.push('.');
                    }
                    push_escaped_key_segment(seg, false, &mut key);
                }
                // Строка занимает ЦЕЛИКОМ диапазон удалённой: ведущий
                // терминатор (съеденный при хвосте без финального EOL)
                // и хвостовой терминатор диапазона берутся из исходных
                // байтов — границы строк и финальный EOL сохраняются.
                let bytes = text.as_bytes();
                let mut rep = String::new();
                rep.push_str(&text[from..line_start_off]);
                rep.push_str(indent);
                rep.push_str(&key);
                rep.push_str(": {}");
                if bytes[to - 1] == b'\n' || bytes[to - 1] == b'\r' {
                    let tstart = if bytes[to - 1] == b'\n' && to >= 2 && bytes[to - 2] == b'\r' {
                        to - 2
                    } else {
                        to - 1
                    };
                    rep.push_str(&text[tstart..to]);
                }
                materialized = Some(rep);
            }
        }
    }
    // Сначала чистое вырезание: если явное определение пустого предка
    // уже остаётся в документе (reopen-случай `a: {}` + `a.y: 1`),
    // материализация продублировала бы строку.
    let cut = |rep: Option<&str>| {
        let extra = rep.map_or(0, str::len);
        let mut r = String::with_capacity(text.len() - (to - from) + extra);
        r.push_str(&text[..from]);
        if let Some(rep) = rep {
            r.push_str(rep);
        }
        r.push_str(&text[to..]);
        r
    };
    if matches_expected(&cut(None), &expected) {
        return Ok(cut(None));
    }
    let result = cut(materialized.as_deref());
    verify(&result, &expected)?;
    Ok(result)
}

/// Тихая проверка совпадения с ожидаемым деревом (для выбора варианта
/// сплайса); ошибка разбора трактуется как несовпадение.
fn matches_expected(candidate: &str, expected: &Value) -> bool {
    crate::parse(candidate).is_ok_and(|actual| actual == *expected)
}

/// Диапазон удаления: [from, to) в исходнике.
fn removal_range<S: AsRef<str>>(
    text: &str,
    root: &Node,
    segments: &[S],
    node: &Node,
) -> Result<(usize, usize, bool), EditError> {
    let slot = node.slot().ok_or_else(|| {
        unsupported(
            "target is a noncontiguous value (dotted-key/reopen intermediate); \
             no lossless removal range exists",
        )
    })?;
    // Физически inline-цель: удаляем сегмент внутри inline-родителя.
    if node.inline {
        return inline_range(text, root, segments, slot).map(|(f, t)| (f, t, false));
    }
    let (anchor, end) = match slot {
        Slot::Pair { colon, body } => (colon as usize, body.1 as usize),
        Slot::Item { start, body } => (start as usize, body.1 as usize),
        Slot::Block { start, end, .. } => (start as usize, end as usize),
    };
    let mut from = line_start(text, anchor);
    if from == 0 {
        // BOM — метаданные исходника: не попадает в диапазон удаления.
        from = crate::parser::leading_bom_len(text);
    }
    let to = line_end_with_eol(text, end);
    // Хвост без финального EOL: съедаем и предыдущий терминатор,
    // чтобы отсутствие финального EOL документа сохранилось.
    let mut stripped_lead = false;
    if to == text.len() && !text[from..to].ends_with(['\n', '\r']) {
        let bytes = text.as_bytes();
        if from > 0 && bytes[from - 1] == b'\n' {
            from -= 1;
            if from > 0 && bytes[from - 1] == b'\r' {
                from -= 1;
            }
            stripped_lead = true;
        } else if from > 0 && bytes[from - 1] == b'\r' {
            from -= 1;
            stripped_lead = true;
        }
    }
    Ok((from, to, stripped_lead))
}

/// Диапазон сегмента inline-родителя с соседней запятой; единственный
/// сегмент — всё внутреннее пространство braces.
fn inline_range<S: AsRef<str>>(
    text: &str,
    root: &Node,
    segments: &[S],
    slot: Slot,
) -> Result<(usize, usize), EditError> {
    let anchor = match slot {
        Slot::Pair { colon, .. } => colon as usize,
        Slot::Item { start, .. } => start as usize,
        Slot::Block { .. } => {
            return Err(unsupported("internal: inline block target"));
        }
    };
    let (p0, p1) = parent_inline_body(text, root, segments)?;
    let body = text
        .get(p0..p1)
        .ok_or_else(|| unsupported("internal: invalid inline parent range"))?;
    let is_object = body.starts_with('{');
    let inner = &body[1..body.len() - 1];
    let parts = split_top_level(
        inner,
        1,
        Span { start: 0, end: 0 },
        if is_object {
            InlineBody::Object
        } else {
            InlineBody::Array
        },
        InlineBounds::for_input(body),
        has_quote_bytes(body.as_bytes()),
    )
    .map_err(|_| unsupported("cannot split inline parent for removal"))?;
    let mut spans: Vec<(usize, usize)> = Vec::with_capacity(parts.len());
    for part in parts {
        let t = part.trim_matches(is_ktav_whitespace);
        if t.is_empty() {
            continue;
        }
        let base = p0 + 1 + (part.as_ptr() as usize - inner.as_ptr() as usize);
        let t0 = base + (t.as_ptr() as usize - part.as_ptr() as usize);
        spans.push((t0, t0 + t.len()));
    }
    let pos = spans
        .iter()
        .position(|&(s, e)| s <= anchor && anchor < e)
        .ok_or_else(|| unsupported("internal: target segment not found in inline parent"))?;
    if spans.len() == 1 {
        return Ok((p0 + 1, p1 - 1));
    }
    if pos + 1 < spans.len() {
        // Сегмент + следующая запятая (и пробелы до следующего).
        Ok((spans[pos].0, spans[pos + 1].0))
    } else {
        // Последний сегмент: предыдущая запятая (после предыдущего).
        Ok((spans[pos - 1].1, spans[pos].1))
    }
}

/// Inline-тело родителя цели: слот с braces либо inline-корень.
fn parent_inline_body<S: AsRef<str>>(
    text: &str,
    root: &Node,
    segments: &[S],
) -> Result<(usize, usize), EditError> {
    let (_, parents) = segments.split_last().ok_or(EditError::NotFound)?;
    let parent = node_at(root, parents)?;
    if let Some(slot) = parent.slot() {
        match slot {
            Slot::Pair { body, .. } | Slot::Item { body, .. } => {
                return Ok((body.0 as usize, body.1 as usize))
            }
            Slot::Block { .. } => {
                return Err(unsupported("internal: inline target in block parent"));
            }
        }
    }
    if parents.is_empty() && root.inline {
        let line = super::layout::physical_line(text, 0)
            .trim_matches(is_ktav_whitespace)
            .trim_start_matches('\u{FEFF}');
        let start = line.as_ptr() as usize - text.as_ptr() as usize;
        return Ok((start, start + line.len()));
    }
    Err(unsupported("internal: inline target without inline parent"))
}
