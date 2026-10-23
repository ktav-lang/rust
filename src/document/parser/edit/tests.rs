//! Тесты точных byte-диапазонов позиционного дерева (`edit::tree::build`).
//!
//! Ожидания формулируются слайсами исходного текста `&text[s..e]` —
//! тест самодоказателен: диапазон верен тогда и только тогда, когда
//! слайс совпадает с литеральным ожиданием.

use super::tree::{build, Node, Slot};

/// Достать (s, e) тела значения узла из его слота.
fn body_range(n: &Node) -> (u32, u32) {
    match n.slot().expect("узел должен иметь слот") {
        Slot::Pair { body, .. } | Slot::Item { body, .. } => body,
        Slot::Block { start, end, .. } => (start, end),
    }
}

/// Слайс исходника, покрываемый телом значения узла.
fn body_of<'t>(n: &Node, t: &'t str) -> &'t str {
    let (s, e) = body_range(n);
    &t[s as usize..e as usize]
}

/// Спуск по цепочке декодированных ключей: `child(child(...))`.
fn descend<'n>(root: &'n Node, path: &[&str]) -> &'n Node {
    let mut n = root;
    for k in path {
        n = n.child(k).unwrap_or_else(|| panic!("нет ключа {k:?}"));
    }
    n
}

/// Тело значения, найденного по пути ключей.
fn pair_body<'t>(root: &Node, t: &'t str, path: &[&str]) -> &'t str {
    body_of(descend(root, path), t)
}

// 1. Отступ перед парой: база офсетов — начало trimmed, не raw.
#[test]
fn indented_pair_body() {
    let t = "  a: 1\n";
    let root = build(t).unwrap();
    assert_eq!(pair_body(&root, t, &["a"]), "1");
}

// 2. Raw-маркер `::` и обычная пара: colon — первый `:`, body — после маркера.
#[test]
fn raw_marker_and_plain_pair() {
    let t = "k:: abc\n";
    let root = build(t).unwrap();
    let k = descend(&root, &["k"]);
    match k.slot().unwrap() {
        Slot::Pair { colon, .. } => {
            assert_eq!(&t[colon as usize..colon as usize + 1], ":");
            assert_eq!(colon, 1); // офсет первого ':'
        }
        _ => panic!("ожидался Slot::Pair"),
    }
    assert_eq!(body_of(k, t), "abc");

    let t2 = "k: zz\n";
    let root2 = build(t2).unwrap();
    assert_eq!(pair_body(&root2, t2, &["k"]), "zz");
}

// 3. Multiline-контейнер `{ ... }`: Block от opener до closer, листья точны.
#[test]
fn multiline_container_block() {
    let t = "a: {\n  x: 1\n}\nb: 2\n";
    let root = build(t).unwrap();
    let a = descend(&root, &["a"]);
    match a.slot().unwrap() {
        Slot::Block { .. } => {}
        _ => panic!("ожидался Slot::Block"),
    }
    assert_eq!(body_of(a, t), "{\n  x: 1\n}");
    assert_eq!(pair_body(&root, t, &["a", "x"]), "1");
    assert_eq!(pair_body(&root, t, &["b"]), "2");
}

// 4. Multiline-строка `(...)`: Block-слайс целиком.
#[test]
fn multiline_string_block() {
    let t = "a: (\n hi\n)\n";
    let root = build(t).unwrap();
    assert_eq!(body_of(descend(&root, &["a"]), t), "(\n hi\n)");
}

// 5. Терминатор multiline с ведущим отступом: end = конец `)` в строке.
#[test]
fn multiline_indented_terminator() {
    let t = "a: (\n hi\n  )\n";
    let root = build(t).unwrap();
    assert_eq!(body_of(descend(&root, &["a"]), t), "(\n hi\n  )");
}

// 6. Вложенный inline: a — inline-объект, y — inline-массив.
#[test]
fn nested_inline_compound() {
    let t = "a: {x: 1, y: [2, 3]}\n";
    let root = build(t).unwrap();
    let a = descend(&root, &["a"]);
    assert!(!a.inline, "a находится в строковой паре");
    assert!(descend(&root, &["a", "x"]).inline);
    assert_eq!(pair_body(&root, t, &["a", "x"]), "1");
    let y = descend(&root, &["a", "y"]);
    assert!(y.inline);
    match y.slot().unwrap() {
        Slot::Pair { .. } => {}
        _ => panic!("ожидался Slot::Pair у y"),
    }
    assert_eq!(body_of(y, t), "[2, 3]");
    assert_eq!(body_of(y.item(0).unwrap(), t), "2");
    assert_eq!(body_of(y.item(1).unwrap(), t), "3");
}

// 7. Inline-корень: inline=true, слота нет, дети позиционированы.
#[test]
fn root_inline_object() {
    let t = "{a: 1}\n";
    let root = build(t).unwrap();
    assert!(root.inline);
    assert!(root.slot().is_none());
    assert_eq!(pair_body(&root, t, &["a"]), "1");
}

// 8. Dotted key: промежуточный `a` — синтетический (slot None).
#[test]
fn dotted_key_synthetic_parent() {
    let t = "a.b: 1\n";
    let root = build(t).unwrap();
    let a = root.child("a").expect("есть a");
    assert!(a.slot().is_none(), "синтетический узел без слота");
    assert_eq!(pair_body(&root, t, &["a", "b"]), "1");
}

// 9. Reopen / noncontiguous: `a` без слота, оба листа и сосед на месте.
#[test]
fn reopened_object() {
    let t = "a.x: 1\nb: 2\na.y: 3\n";
    let root = build(t).unwrap();
    assert!(descend(&root, &["a"]).slot().is_none());
    assert_eq!(pair_body(&root, t, &["a", "x"]), "1");
    assert_eq!(pair_body(&root, t, &["a", "y"]), "3");
    assert_eq!(pair_body(&root, t, &["b"]), "2");
}

// 10. BOM: офсеты в исходных координатах, тело после BOM.
#[test]
fn leading_bom_offsets() {
    let t = "\u{FEFF}a: 1\n";
    let root = build(t).unwrap();
    let (s, e) = body_range(descend(&root, &["a"]));
    assert!(s >= 3, "диапазон должен начинаться после BOM, got {s}");
    assert_eq!(&t[s as usize..e as usize], "1");
}

// 11. CRLF: офсеты второй строки корректны.
#[test]
fn crlf_lines() {
    let t = "a: 1\r\nb: 2\r\n";
    let root = build(t).unwrap();
    assert_eq!(pair_body(&root, t, &["a"]), "1");
    assert_eq!(pair_body(&root, t, &["b"]), "2");
}

// 12. LF без финального newline.
#[test]
fn no_trailing_newline() {
    let t = "a: 1";
    let root = build(t).unwrap();
    assert_eq!(pair_body(&root, t, &["a"]), "1");
}

// 13. CR-only терминаторы строк.
#[test]
fn cr_only_lines() {
    let t = "a: 1\rb: 2\r";
    let root = build(t).unwrap();
    assert_eq!(pair_body(&root, t, &["a"]), "1");
    assert_eq!(pair_body(&root, t, &["b"]), "2");
}

// 14. Quoted ключ: декодируется insert_value, по нему есть lookup.
#[test]
fn quoted_key_lookup() {
    let t = "\"a.b\": 1\n";
    let root = build(t).unwrap();
    assert_eq!(pair_body(&root, t, &["a.b"]), "1");
}

// 15. Inline raw-пара `k:: v` внутри inline-объекта.
#[test]
fn inline_raw_pair() {
    let t = "a: {k:: v, j: w}\n";
    let root = build(t).unwrap();
    assert_eq!(pair_body(&root, t, &["a", "k"]), "v");
    assert_eq!(pair_body(&root, t, &["a", "j"]), "w");
}

// 16. Inline-корень с quoted-ключом: запятая внутри кавычек не сплитит.
#[test]
fn inline_root_quoted_key_with_comma() {
    let t = "{\"a,b\": 1}\n";
    let root = build(t).unwrap();
    assert!(root.inline);
    assert_eq!(pair_body(&root, t, &["a,b"]), "1");
}

// Точность colon-офсета Slot::Pair на multiline-контейнере.
#[test]
fn pair_colon_offset_is_exact() {
    let t = "a: {\n  x: yy\n}\n";
    let root = build(t).unwrap();
    let x = descend(&root, &["a", "x"]);
    match x.slot().unwrap() {
        Slot::Pair { colon, .. } => {
            assert_eq!(&t[colon as usize..colon as usize + 1], ":");
            // colon указывает на `:` строки `x: yy`, т.е. сразу за ключом.
            assert_eq!(&t[colon as usize - 1..colon as usize], "x");
        }
        _ => panic!("ожидался Slot::Pair"),
    }
}

// --- reopen позиционированного объекта: ложный contiguous-слот инвалидируется ---

// Reopen пустого inline-объекта: `a` — без слота, добавленный y позиционирован.
#[test]
fn reopen_empty_inline() {
    let t = "a: {}\na.y: 2\n";
    assert!(crate::parse(t).is_ok(), "документ обязан быть валидным");
    let root = build(t).unwrap();
    let a = root.child("a").expect("есть a");
    assert!(a.slot().is_none(), "слот `a` инвалидируется reopen'ом");
    assert_eq!(pair_body(&root, t, &["a", "y"]), "2");
}

// Reopen заполненного inline-объекта: слот снят, оба листа на месте.
#[test]
fn reopen_filled_inline() {
    let t = "a: {x: 1}\na.y: 2\n";
    assert!(crate::parse(t).is_ok(), "документ обязан быть валидным");
    let root = build(t).unwrap();
    let a = root.child("a").expect("есть a");
    assert!(a.slot().is_none(), "слот `a` инвалидируется reopen'ом");
    assert_eq!(pair_body(&root, t, &["a", "x"]), "1");
    assert_eq!(pair_body(&root, t, &["a", "y"]), "2");
}

// Reopen explicit multiline-блока: слот снят, листья точны.
#[test]
fn reopen_explicit_block() {
    let t = "a: {\n  x: 1\n}\na.y: 2\n";
    assert!(crate::parse(t).is_ok(), "документ обязан быть валидным");
    let root = build(t).unwrap();
    let a = root.child("a").expect("есть a");
    assert!(a.slot().is_none(), "слот `a` инвалидируется reopen'ом");
    assert_eq!(pair_body(&root, t, &["a", "x"]), "1");
    assert_eq!(pair_body(&root, t, &["a", "y"]), "2");
}

// Соседний `b` reopen'ом не затрагивается: слот и тело сохранены.
#[test]
fn reopen_sibling_intact() {
    let t = "a: {}\nb: 7\na.y: 2\n";
    assert!(crate::parse(t).is_ok(), "документ обязан быть валидным");
    let root = build(t).unwrap();
    let b = root.child("b").expect("есть b");
    assert!(b.slot().is_some(), "сосед `b` сохраняет слот");
    assert_eq!(pair_body(&root, t, &["b"]), "7");
    assert!(root.child("a").unwrap().slot().is_none());
    assert_eq!(pair_body(&root, t, &["a", "y"]), "2");
}
