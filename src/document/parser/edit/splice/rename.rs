//! Переименование ключа существующего члена Object: позиция, порядок
//! и значение сохраняются, меняются только байты ключа. Одинаковый
//! ключ — byte-identical no-op; занятый сосед — AlreadyExists;
//! индекс массива и noncontiguous-цели — Unsupported. Новый ключ
//! кодируется общим эмиттером § 5.9.10 (bare/quoted + re-escape).

use super::ops::{first_key_of, node_at, pair_key_range, unsupported, value_at, verify};
use super::tree::{self, Kind, Slot};
use crate::edit::EditError;
use crate::value::Value;

pub(super) fn apply<S: AsRef<str>>(
    text: &str,
    segments: &[S],
    new_key: &str,
) -> Result<String, EditError> {
    if text.len() > u32::MAX as usize {
        return Err(unsupported(
            "document exceeds 4 GiB; offsets do not fit u32",
        ));
    }
    if segments.is_empty() {
        return Err(EditError::NotFound);
    }
    if new_key.is_empty() {
        return Err(unsupported("empty key name is not representable"));
    }
    let (key, parent_segs) = segments.split_last().ok_or(EditError::NotFound)?;
    let key = key.as_ref();
    let original =
        crate::parse(text).map_err(|e| unsupported(&format!("source text does not parse: {e}")))?;
    let parent_value = super::lookup_value(&original, parent_segs).ok_or(EditError::NotFound)?;
    match parent_value {
        Value::Object(o) => {
            if o.get(key).is_none() {
                return Err(EditError::NotFound);
            }
            // Идентичный ключ — byte-identical no-op (проверка до
            // коллизии: цель и есть «занятый» ключ).
            if key == new_key {
                return Ok(text.to_string());
            }
            if o.get(new_key).is_some() {
                return Err(EditError::AlreadyExists);
            }
        }
        // Переименование индекса массива — не переименование ключа.
        Value::Array(_) => {
            return Err(unsupported(
                "rename of an Array element is not supported; only Object keys can be renamed",
            ))
        }
        _ => return Err(EditError::NotFound),
    }
    let (root, _) = tree::build_with_meta(text)
        .map_err(|e| unsupported(&format!("source text does not parse: {e}")))?;
    let parent_node = node_at(&root, parent_segs)?;
    if !matches!(parent_node.kind, Kind::Object(_)) {
        return Err(unsupported("internal: parent node is not an object"));
    }
    let target = parent_node.child(key).ok_or(EditError::NotFound)?;
    // Слот Pair или Block с anchor (контейнер, открытый парой);
    // Item/Block без anchor — элемент массива.
    let colon = match target.slot() {
        Some(Slot::Pair { colon, .. }) => colon as usize,
        Some(Slot::Block {
            anchor: Some(colon),
            ..
        }) => colon as usize,
        Some(_) => {
            return Err(unsupported(
                "rename of an Array element is not supported; only Object keys can be renamed",
            ))
        }
        None => {
            return Err(unsupported(
                "target is a noncontiguous value (dotted-key/reopen intermediate); \
                 no lossless rename range exists",
            ))
        }
    };
    let (key_start, key_end) = pair_key_range(text, colon);
    if key_start >= key_end {
        return Err(unsupported("internal: empty key range"));
    }
    // Заменяется только ПОСЛЕДНИЙ сегмент dotted-ключа: префикс предков
    // (написание, кавычки, escapes, пробелы) сохраняется байт-в-байт.
    // Разбиение — СОБСТВЕННАЯ грамматика парсера (`split_key_path`):
    // quoted-сегмент " ' ` непрозрачен для точки, `\` экранирует
    // следующий байт, сегмент-слайс включает кавычки — замена идёт
    // целым сегментом без смешивания представлений. BOM-защита только
    // для первого ключа корня.
    let key_text = text
        .get(key_start..key_end)
        .ok_or_else(|| unsupported("internal: invalid key range"))?;
    let segments = crate::parser::inline::split_key_path(key_text);
    let last = segments
        .last()
        .ok_or_else(|| unsupported("internal: key without segments"))?;
    let seg_start = last.as_ptr() as usize - key_text.as_ptr() as usize;
    let seg_end = seg_start + last.len();
    let abs_start = key_start + seg_start;
    let abs_end = key_start + seg_end;
    let root_first = parent_segs.is_empty() && first_key_of(parent_node) == Some(key);
    let mut encoded_key = String::new();
    crate::render::helpers::push_escaped_key_segment(new_key, root_first, &mut encoded_key);
    let mut result = String::with_capacity(text.len() + encoded_key.len());
    result.push_str(&text[..abs_start]);
    result.push_str(&encoded_key);
    result.push_str(&text[abs_end..]);
    // Ожидаемое дерево: ключ переименован на месте, порядок сохранён.
    let mut expected = original;
    let parent = value_at(&mut expected, parent_segs)?;
    match parent {
        Value::Object(o) => {
            let renamed: Vec<(crate::value::Scalar, Value)> = o
                .iter()
                .map(|(k, v)| {
                    let k = if k.as_str() == key {
                        new_key.into()
                    } else {
                        k.clone()
                    };
                    (k, v.clone())
                })
                .collect();
            *o = renamed.into_iter().collect();
        }
        _ => return Err(unsupported("internal: parent value is not an object")),
    }
    verify(&result, &expected)?;
    Ok(result)
}
