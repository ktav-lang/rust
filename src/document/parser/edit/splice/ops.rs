//! Общие помощники операций insert/remove/rename поверх позиционного
//! дерева: спуск по узлам, строковые границы, диапазон ключа пары,
//! рендер новой пары и обязательная семантическая верификация.

use super::tree::{Kind, Node};
use super::{parse_index, plan_value, PlanCtx};
use crate::edit::EditError;
use crate::render::helpers::push_escaped_key_segment;
use crate::value::Value;
use crate::whitespace::is_ktav_whitespace;

pub(super) fn unsupported(reason: &str) -> EditError {
    EditError::Unsupported {
        reason: reason.to_string(),
    }
}

/// Узел по декодированным сегментам: в Object — ключ, в Array — индекс.
/// Отсутствующий ключ/индекс или спуск через скаляр — NotFound.
pub(super) fn node_at<'n, S: AsRef<str>>(
    root: &'n Node,
    segments: &[S],
) -> Result<&'n Node, EditError> {
    let mut cur = root;
    for seg in segments {
        cur = match &cur.kind {
            Kind::Object(_) => cur.child(seg.as_ref()).ok_or(EditError::NotFound)?,
            Kind::Array(_) => cur
                .item(parse_index(seg.as_ref())?)
                .ok_or(EditError::NotFound)?,
            Kind::Scalar => return Err(EditError::NotFound),
        };
    }
    Ok(cur)
}

/// Родитель по декодированным сегментам в распарсенном дереве значений
/// (для построения ожидаемого результата верификации).
pub(super) fn value_at<'a, S: AsRef<str>>(
    v: &'a mut Value,
    segments: &[S],
) -> Result<&'a mut Value, EditError> {
    let mut cur = v;
    for seg in segments {
        let seg = seg.as_ref();
        cur = match cur {
            Value::Object(o) => o.get_mut(seg).ok_or(EditError::NotFound)?,
            Value::Array(a) => a.get_mut(parse_index(seg)?).ok_or(EditError::NotFound)?,
            _ => return Err(EditError::NotFound),
        };
    }
    Ok(cur)
}

/// Начало строки, содержащей офсет `pos`.
pub(super) fn line_start(text: &str, pos: usize) -> usize {
    text[..pos].rfind(['\n', '\r']).map_or(0, |i| i + 1)
}

/// Конец строки, включая терминатор (LF, CRLF или один CR); на EOF
/// возвращает длину текста без терминатора.
pub(super) fn line_end_with_eol(text: &str, pos: usize) -> usize {
    match text[pos..].find(['\n', '\r']) {
        Some(i) => {
            let j = pos + i;
            let bytes = text.as_bytes();
            if bytes[j] == b'\r' && bytes.get(j + 1) == Some(&b'\n') {
                j + 2
            } else {
                j + 1
            }
        }
        None => text.len(),
    }
}

/// Диапазон байтов ключа пары: от colon назад до границы сегмента.
/// Грамматика парсера: quoted-сегмент (", ', `) открывается только на
/// старте сегмента, его содержимое (включая , { [ .) прозрачно для
/// границ; `\` экранирует следующий байт. BOM не входит в диапазон.
pub(super) fn pair_key_range(text: &str, colon: usize) -> (usize, usize) {
    let key_end = text[..colon].trim_end_matches(is_ktav_whitespace).len();
    let mut start = line_start(text, colon);
    if start == 0 {
        // BOM — метаданные исходника, не часть ключа первой пары.
        start = crate::parser::leading_bom_len(text);
    }
    let bytes = text.as_bytes();
    let mut boundary = start;
    let mut i = start;
    let mut seg_start = true;
    while i < colon {
        if seg_start {
            // § 5.3: пробелы вокруг сегмента; quoted-сегмент открывается
            // только на старте сегмента.
            let ws_len =
                text[i..colon].len() - text[i..colon].trim_start_matches(is_ktav_whitespace).len();
            i += ws_len;
            if i < colon && matches!(bytes[i], b'"' | b'\'' | b'`') {
                let quote = bytes[i];
                i += 1;
                while i < colon {
                    match bytes[i] {
                        _ if bytes[i] == b'\\' => i += 2,
                        _ if bytes[i] == quote => {
                            i += 1;
                            break;
                        }
                        _ => i += 1,
                    }
                }
                seg_start = false;
                continue;
            }
            seg_start = false;
        }
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'.' => {
                seg_start = true;
                boundary = i + 1;
            }
            b',' | b'{' | b'[' => {
                boundary = i + 1;
                seg_start = true;
            }
            _ => {}
        }
        i += 1;
    }
    let lead = text[boundary..key_end]
        .trim_start_matches(is_ktav_whitespace)
        .len();
    (key_end - lead, key_end)
}

/// Текст новой пары `key: value` (или `key:: value` по needs_raw):
/// ключ кодируется общим эмиттером § 5.9.10, тело — plan_value.
/// `root_first` включает BOM-защиту первого ключа корня.
pub(super) fn render_pair(
    key: &str,
    replacement: &Value,
    pc: &PlanCtx,
    root_first: bool,
) -> Result<String, EditError> {
    if key.is_empty() {
        return Err(unsupported("empty key name is not representable"));
    }
    let mut out = String::new();
    push_escaped_key_segment(key, root_first, &mut out);
    // Inline-родитель: собственный owning inline-encoder (§ 3.7): экранирует
    // запятые/скобки/backslash/переводы строк, числоподобные строки получают
    // String-провенанс. Standalone-план (verbatim-блок, raw-маркер) сюда не
    // попадает: inline обязан остаться однострочным.
    if pc.inline_compound {
        let body = match replacement {
            Value::String(s) => {
                let mut b = String::new();
                super::push_inline_string(s, super::needs_raw_marker(s), &mut b);
                b
            }
            _ => super::inline_compound_body(replacement, 0)?,
        };
        out.push(':');
        out.push(' ');
        out.push_str(&body);
        return Ok(out);
    }
    let (body, raw) = plan_value(replacement, pc)?;
    out.push(':');
    if raw {
        out.push(':');
    }
    out.push(' ');
    out.push_str(body.as_str());
    Ok(out)
}

/// Обязательная верификация: результат обязан парситься и совпадать с
/// независимо построенным ожидаемым деревом; иначе fail closed.
pub(super) fn verify(result: &str, expected: &Value) -> Result<(), EditError> {
    let actual = crate::parse(result).map_err(|e| EditError::Unsupported {
        reason: format!("patch produced an invalid document: {e}"),
    })?;
    if actual != *expected {
        return Err(unsupported(
            "semantic mismatch after splice; no lossless edit produced",
        ));
    }
    Ok(())
}

/// Иммутабельный обход по API-пути: Object-ключи И Array-индексы
/// (для проверки опустевших dotted-предков через смешанные контейнеры).
pub(super) fn path_get<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    let mut cur = v;
    for k in keys {
        cur = match cur {
            Value::Object(o) => o.get(*k)?,
            Value::Array(a) => a.get(parse_index(k).ok()?)?,
            _ => return None,
        };
    }
    Some(cur)
}

/// Уникальный текстовой ключ последнего элемента entries (для
/// root_first-определения переименования).
pub(super) fn first_key_of(node: &Node) -> Option<&str> {
    match &node.kind {
        Kind::Object(t) => t.entries.iter().next().map(|(k, _)| k.as_str()),
        _ => None,
    }
}
