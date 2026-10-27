//! Вставка нового ключа в СУЩЕСТВУЮЩИЙ Object: родитель существует,
//! конечный сегмент пути — новый ключ. Байты вне точки вставки не
//! трогаются; в блочный/неявный контейнер пара добавляется отдельной
//! строкой в конец, в inline — сегментом перед closer'ом. Вставка в
//! Array — явный Unsupported, существующий ключ — AlreadyExists
//! (никогда не overwrite). После сплайса — layout-раскрытие inline
//! при необходимости и семантическая верификация.

use super::ops::{
    line_end_with_eol, line_start, node_at, render_pair, unsupported, value_at, verify,
};
use super::tree::{self, Kind, Node, Slot};
use super::{detect_eol, line_indent, PlanCtx, SlotKind};
use crate::edit::EditError;
use crate::value::Value;

pub(super) fn apply<S: AsRef<str>>(
    text: &str,
    segments: &[S],
    replacement: &Value,
) -> Result<String, EditError> {
    if text.len() > u32::MAX as usize {
        return Err(unsupported(
            "document exceeds 4 GiB; offsets do not fit u32",
        ));
    }
    let (key, parent_segs) = segments.split_last().ok_or(EditError::NotFound)?;
    let key = key.as_ref();
    // Оригинал обязан парситься; дерево значений даёт ошибочный контракт.
    let original =
        crate::parse(text).map_err(|e| unsupported(&format!("source text does not parse: {e}")))?;
    let parent_value = super::lookup_value(&original, parent_segs).ok_or(EditError::NotFound)?;
    match parent_value {
        Value::Object(o) => {
            if o.get(key).is_some() {
                return Err(EditError::AlreadyExists);
            }
        }
        // Явный контракт: вставка/append в Array не поддерживается.
        Value::Array(_) => {
            return Err(unsupported(
                "insertion into an Array is not supported; only Object members can be created",
            ))
        }
        _ => return Err(EditError::NotFound),
    }
    let (root, root_explicit) = tree::build_with_meta(text)
        .map_err(|e| unsupported(&format!("source text does not parse: {e}")))?;
    let parent_node = node_at(&root, parent_segs)?;
    if !matches!(parent_node.kind, Kind::Object(_)) {
        // Расхождение дерева значений и позиционного дерева: fail closed.
        return Err(unsupported("internal: parent node is not an object"));
    }
    let expected = {
        let mut expected = original;
        match value_at(&mut expected, parent_segs)? {
            Value::Object(o) => {
                o.insert(key.into(), replacement.clone());
            }
            _ => return Err(unsupported("internal: parent value is not an object")),
        }
        expected
    };

    // План: (диапазон замены, текст вставки, нужно ли block-раскрытие).
    let (from, to, inserted_text, force) = insert_plan(
        text,
        &root,
        root_explicit,
        parent_node,
        parent_segs,
        key,
        replacement,
    )?;
    let mut result = String::with_capacity(text.len() + inserted_text.len());
    result.push_str(&text[..from]);
    result.push_str(&inserted_text);
    result.push_str(&text[to..]);
    // force => inline-кандидат уже собран; layout раскрывает МИНИМАЛЬНУЮ
    // цепочку внешних inline-обёрток (outermost owner + затронутые части),
    // нетронутые ветви остаются inline. Если block-форма невыразима
    // (CR, строка `))`), кандидат остаётся проверяемым inline.
    let result = if force
        || super::layout::physical_line(&result, from).chars().count()
            > super::layout::physical_line(text, from)
                .chars()
                .count()
                .max(100)
    {
        super::layout::adjust(
            text,
            result,
            &root,
            parent_segs,
            replacement,
            (from, to, inserted_text.len()),
        )?
    } else {
        result
    };
    verify(&result, &expected)?;
    Ok(result)
}

/// Возможна ли lossless standalone-блок-форма значения: CR и строка
/// `))` не представимы verbatim-блоком — тогда остаётся inline
/// (guarantee impossible_block_encoding_keeps_valid_inline_form).
fn block_encodable(v: &Value) -> bool {
    match v {
        Value::String(s) => {
            !s.contains('\r')
                && !s
                    .lines()
                    .any(|l| l.trim_matches(|c: char| c == ' ' || c == '\t') == "))")
        }
        Value::Object(o) => o.values().all(block_encodable),
        Value::Array(a) => a.iter().all(block_encodable),
        _ => true,
    }
}

/// План вставки: диапазон [from, to) исходника заменяется на
/// inserted_text; expanded=true — уже раскрытый блок (adjust не нужен).
#[allow(clippy::type_complexity)]
fn insert_plan<'s, S: AsRef<str>>(
    text: &'s str,
    root: &'s Node,
    root_explicit: bool,
    parent_node: &'s Node,
    parent_segs: &[S],
    key: &str,
    replacement: &'s Value,
) -> Result<(usize, usize, String, bool), EditError> {
    // Inline-родитель: тело — односторонние braces текущего слота.
    if let Some((p0, p1)) = inline_body_of(root, parent_node, parent_segs, text)? {
        let inner = text
            .get(p0 + 1..p1 - 1)
            .ok_or_else(|| unsupported("internal: invalid inline parent range"))?;
        let wants_block = super::layout::requires_block_for(replacement)?
            || matches!(
                replacement,
                Value::String(s) if crate::render::helpers::string_needs_multiline(s)
            );
        // Блок — только если standalone-форма выразима; иначе owning
        // inline encoder даёт безопасную inline-форму (CR/LF/`))` —
        // экранируются, семантика Value сохраняется).
        let force = wants_block && block_encodable(replacement);
        let indent = line_indent(text, p0);
        let eol = detect_eol(text, p1);
        let pc = inline_pair_ctx(&indent, eol);
        let pair = render_pair(key, replacement, &pc, false)?;
        if inner
            .trim_matches(crate::whitespace::is_ktav_whitespace)
            .is_empty()
        {
            return Ok((p0 + 1, p1 - 1, pair, force));
        }
        let trimmed = inner.trim_end_matches(crate::whitespace::is_ktav_whitespace);
        let trailing_comma = trimmed.ends_with(',');
        let comma_is_structural = if trailing_comma {
            let bytes = trimmed.as_bytes();
            let mut slash_count = 0;
            let mut i = bytes.len() - 1;
            while i > 0 && bytes[i - 1] == b'\\' {
                slash_count += 1;
                i -= 1;
            }
            slash_count % 2 == 0
        } else {
            false
        };
        let insertion = if comma_is_structural {
            p0 + 1 + trimmed.len()
        } else {
            p1 - 1
        };
        let mut seg = String::with_capacity(pair.len() + 3);
        if comma_is_structural {
            seg.push(' ');
            seg.push_str(&pair);
            seg.push(',');
        } else {
            seg.push_str(", ");
            seg.push_str(&pair);
        }
        return Ok((insertion, insertion, seg, force));
    }
    if parent_segs.is_empty() {
        return root_insert(text, root_explicit, parent_node, key, replacement);
    }
    // Блочный (многострочный) родитель: новая строка перед closer-строкой.
    let Slot::Block { start, end, .. } = parent_node.slot().ok_or_else(|| {
        unsupported(
            "parent is a noncontiguous value (dotted-key/reopen intermediate); \
             no lossless insertion point exists",
        )
    })?
    else {
        return Err(unsupported("internal: unexpected parent slot"));
    };
    // Отступ по source: отступ последнего соседа; для пустого блока —
    // новый уровень: opener + четыре пробела.
    let opener_indent = line_indent(text, start as usize);
    let child_indent =
        sibling_indent(text, parent_node).unwrap_or_else(|| format!("{opener_indent}    "));
    let eol = detect_eol(text, end as usize);
    let pc = block_pair_ctx(&child_indent, eol);
    let pair = render_pair(key, replacement, &pc, false)?;
    // Точка вставки — начало строки closer'а: после всех членов и их
    // комментариев, до закрывающей скобки.
    let at = line_start(text, end as usize);
    Ok((at, at, format!("{child_indent}{pair}{eol}"), false))
}

/// Вставка в корневой Object: неявный корень — строка в самый конец
/// документа; пустой/явный корень — по месту открывающей скобки.
fn root_insert(
    text: &str,
    root_explicit: bool,
    root: &Node,
    key: &str,
    replacement: &Value,
) -> Result<(usize, usize, String, bool), EditError> {
    let Kind::Object(t) = &root.kind else {
        return Err(unsupported("internal: root is not an object"));
    };
    let eol = detect_eol(text, text.len());
    if root_explicit && t.entries.is_empty() {
        // Пара перед closer: исходный EOL открывающей строки остаётся.
        let closer = root_closer_line_start(text)
            .ok_or_else(|| unsupported("internal: explicit root without closer"))?;
        let pc = block_pair_ctx("    ", eol);
        let pair = render_pair(key, replacement, &pc, false)?;
        return Ok((closer, closer, format!("    {pair}{eol}"), false));
    }
    // Конец документа: правило финального EOL едино для пустого
    // (комментарии/BOM-only) и непустого неявного корня.
    let eof_append = |pair: String| {
        let ins = if text.is_empty() {
            pair
        } else if text.ends_with(['\n', '\r']) {
            // Финальный EOL уже терминирует последнюю строку; новая пара
            // получает свой EOL — наличие финального EOL сохраняется.
            format!("{pair}{eol}")
        } else {
            // Без финального EOL: терминируем предыдущую строку, свой не
            // добавляем — отсутствие финального EOL сохраняется.
            format!("{eol}{pair}")
        };
        (text.len(), text.len(), ins, false)
    };
    if t.entries.is_empty() {
        // Пустой (в том числе комментарийный/BOM-only) документ.
        let pc = block_pair_ctx("", eol);
        let pair = render_pair(key, replacement, &pc, false)?;
        return Ok(eof_append(pair));
    }
    if root_explicit {
        // Явный блочный корень с членами: новая строка ПЕРЕД строкой
        // closer'а с отступом соседей — блок остаётся блоком, закрывающая
        // скобка не покидает свою строку.
        let closer = root_closer_line_start(text)
            .ok_or_else(|| unsupported("internal: explicit root without closer"))?;
        let child_indent = sibling_indent(text, root).unwrap_or_default();
        let eol = detect_eol(text, closer);
        let pc = block_pair_ctx(&child_indent, eol);
        let pair = render_pair(key, replacement, &pc, false)?;
        return Ok((closer, closer, format!("{child_indent}{pair}{eol}"), false));
    }
    // Неявный корень с членами (включая dotted/reopen-определения):
    // строка в самый конец документа — хвостовые комментарии остаются
    // на своих строках, никакие байты не попадают в чужую строку.
    let pc = block_pair_ctx("", eol);
    let pair = render_pair(key, replacement, &pc, false)?;
    Ok(eof_append(pair))
}

/// Старт строки закрывающей `}` явного корня (последняя `}`-строка
/// документа; после неё бывают только комментарии).
fn root_closer_line_start(text: &str) -> Option<usize> {
    let mut end = text.len();
    while end > 0 {
        let ls = line_start(text, end - 1);
        let le = line_end_with_eol(text, ls);
        if text[ls..le].trim_matches(crate::whitespace::is_ktav_whitespace) == "}" {
            return Some(ls);
        }
        end = ls;
    }
    None
}

/// Диапазон inline-тела родителя: слот Pair/Item с односторонними
/// braces, либо весь inline-корень (слота нет — тело берётся со строки).
fn inline_body_of<'s, S: AsRef<str>>(
    root: &'s Node,
    parent_node: &'s Node,
    parent_segs: &[S],
    text: &'s str,
) -> Result<Option<(usize, usize)>, EditError> {
    if let Some(slot) = parent_node.slot() {
        let (p0, p1) = match slot {
            Slot::Pair { body, .. } | Slot::Item { body, .. } => (body.0 as usize, body.1 as usize),
            Slot::Block { .. } => return Ok(None),
        };
        let b = text.as_bytes();
        if b.get(p0) == Some(&b'{') && b.get(p1 - 1) == Some(&b'}') {
            return Ok(Some((p0, p1)));
        }
        return Ok(None);
    }
    // Корень: только физически inline-корень имеет braces-тело.
    if parent_segs.is_empty() && root.inline {
        let line = super::layout::physical_line(text, 0)
            .trim_matches(crate::whitespace::is_ktav_whitespace)
            .trim_start_matches('\u{FEFF}');
        let start = line.as_ptr() as usize - text.as_ptr() as usize;
        let body = text
            .get(start..start + line.len())
            .ok_or_else(|| unsupported("internal: invalid inline root range"))?;
        if body.starts_with('{') && body.ends_with('}') {
            return Ok(Some((start, start + body.len())));
        }
    }
    Ok(None)
}

/// Отступ последнего существующего члена блока (по source); None —
/// членов нет либо у них нет диапазонов.
fn sibling_indent<'t>(text: &'t str, node: &'t Node) -> Option<String> {
    let Kind::Object(t) = &node.kind else {
        return None;
    };
    let (_, last) = t.entries.iter().next_back()?;
    let pos = match last.slot()? {
        Slot::Pair { colon, .. } => colon as usize,
        Slot::Item { start, .. } => start as usize,
        Slot::Block { start, .. } => start as usize,
    };
    Some(line_indent(text, line_start(text, pos)))
}

fn block_pair_ctx<'t>(indent: &'t str, eol: &'t str) -> PlanCtx<'t> {
    PlanCtx {
        slot: SlotKind::Pair,
        role_item_block: false,
        existing_raw: false,
        root_first: false,
        indent,
        eol,
        inline_compound: false,
        block_compound: true,
    }
}

fn inline_pair_ctx<'t>(indent: &'t str, eol: &'t str) -> PlanCtx<'t> {
    PlanCtx {
        slot: SlotKind::Pair,
        role_item_block: false,
        existing_raw: false,
        root_first: false,
        indent,
        eol,
        inline_compound: true,
        block_compound: false,
    }
}
