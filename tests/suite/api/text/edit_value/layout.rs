//! Exact-output регрессии формы и минимального раскрытия предков.

use super::{edit_value, edit_value_segments, parse, s, EditError, Value};

fn check(source: &str, path: &[&str], replacement: &Value, expected: &str) {
    let textual = path
        .iter()
        .map(|s| s.replace('\\', "\\\\").replace('.', "\\."))
        .collect::<Vec<_>>()
        .join(".");
    let mut wanted = parse(source).unwrap();
    let mut target = &mut wanted;
    for segment in path {
        target = match target {
            Value::Object(object) => object.get_mut(*segment).unwrap(),
            Value::Array(array) => &mut array[segment.parse::<usize>().unwrap()],
            _ => panic!("неверный путь теста"),
        };
    }
    *target = replacement.clone();
    for result in [
        edit_value(source, &textual, replacement),
        edit_value_segments(source, path, replacement),
    ] {
        let result = result.unwrap();
        assert_eq!(result, expected);
        assert_eq!(parse(&result).unwrap(), wanted);
    }
}

#[test]
fn deep_leaf_preserves_each_source_form() {
    for source in [
        "targets.storagehost.address: old\ntargets.storagehost.user: mcp-agent\n",
        "targets: {storagehost: {address: old, user: mcp-agent}}\n",
        "targets: {\n  storagehost: {\n    address: old\n    user: mcp-agent\n  }\n}\n",
    ] {
        check(
            source,
            &["targets", "storagehost", "address"],
            &s("192.168.0.88:22"),
            &source.replace("old", "192.168.0.88:22"),
        );
    }
}

#[test]
fn deep_array_leaf_preserves_mixed_forms() {
    for source in [
        "targets: [{storagehost: [old, keep]}, other]\n",
        "targets: [\n  {storagehost: [old, keep]}\n  other\n]\n",
        "targets: [\n  {\n    storagehost: [\n      old\n      keep\n    ]\n  }\n  other\n]\n",
    ] {
        check(
            source,
            &["targets", "0", "storagehost", "0"],
            &s("new"),
            &source.replace("old", "new"),
        );
    }
}

#[test]
fn whole_inline_containers_remain_inline() {
    for (source, value, expected) in [
        ("a: {x: 1}\n", parse("{y: 2}").unwrap(), "a: {y: 2}\n"),
        ("a: [1, 2]\n", parse("[3, 4]").unwrap(), "a: [3, 4]\n"),
        ("a: {}\n", parse("{y: 2}").unwrap(), "a: {y: 2}\n"),
        ("a: []\n", parse("[3, 4]").unwrap(), "a: [3, 4]\n"),
        ("a: {}\n", parse("[3, 4]").unwrap(), "a: [3, 4]\n"),
    ] {
        check(source, &["a"], &value, expected);
    }
}

#[test]
fn emptying_block_containers_does_not_collapse_them() {
    check(
        "a: {\n  x: 1\n}\n",
        &["a"],
        &parse("{}").unwrap(),
        "a: {\n}\n",
    );
    check("a: [\n  1\n]\n", &["a"], &parse("[]").unwrap(), "a: [\n]\n");
}

#[test]
fn newline_unfolds_only_inline_ancestor_chain() {
    check(
        "## top\na: [{x: old, untouched: [1, 2]}, {keep: yes}]\nz: keep\n",
        &["a", "0", "x"], &s("one\ntwo"),
        "## top\na: [\n    {\n        x: ((\none\ntwo\n        ))\n        untouched: [1, 2]\n    }\n    {keep: yes}\n]\nz: keep\n",
    );
}

#[test]
fn newline_under_dotted_key_does_not_expand_dotted_ancestors() {
    check(
        "a.b: [old, keep]\na.c: other\n",
        &["a", "b", "0"],
        &s("one\ntwo"),
        "a.b: [\n    ((\none\ntwo\n    ))\n    keep\n]\na.c: other\n",
    );
}

#[test]
fn dense_escapes_unfold_but_sparse_escapes_do_not() {
    check(
        "a: [old, keep]\n",
        &["a", "0"],
        &s("a,b,c,d,e"),
        "a: [\n    a,b,c,d,e\n    keep\n]\n",
    );
    check(
        "a: [old]\n",
        &["a", "0"],
        &s("a,b,c,d"),
        "a: [a\\,b\\,c\\,d]\n",
    );
    let sparse = format!("{}a,b,c,d,e", "x".repeat(30));
    check(
        "a: [old]\n",
        &["a", "0"],
        &s(&sparse),
        &format!("a: [{}]\n", sparse.replace(',', "\\,")),
    );
}

#[test]
fn width_boundary_is_in_characters_not_bytes() {
    for (size, expanded) in [(95, false), (96, true)] {
        let value = "я".repeat(size);
        let expected = if expanded {
            format!("a: [\n    {value}\n]\n")
        } else {
            format!("a: [{value}]\n")
        };
        check("a: [old]\n", &["a", "0"], &s(&value), &expected);
    }
}

#[test]
fn already_long_line_only_unfolds_when_it_grows() {
    let source = format!("a: [{}, keep]\n", "x".repeat(120));
    check(
        &source,
        &["a", "0"],
        &s(&"y".repeat(120)),
        &source.replace('x', "y"),
    );
    check(
        &source,
        &["a", "0"],
        &s(&"y".repeat(121)),
        &format!("a: [\n    {}\n    keep\n]\n", "y".repeat(121)),
    );
}

#[test]
fn width_expansion_stops_when_inner_line_fits() {
    let value = "x".repeat(91);
    check(
        "a: [{x: old}]\n",
        &["a", "0", "x"],
        &s(&value),
        &format!("a: [\n    {{x: {value}}}\n]\n"),
    );
}

#[test]
fn escape_overhead_boundary_is_exact() {
    let dense = format!("{}a,b,c,d,e", "x".repeat(7));
    let sparse = format!("{}a,b,c,d,e", "x".repeat(8));
    check(
        "a: [old]\n",
        &["a", "0"],
        &s(&dense),
        &format!("a: [\n    {dense}\n]\n"),
    );
    check(
        "a: [old]\n",
        &["a", "0"],
        &s(&sparse),
        &format!("a: [{}]\n", sparse.replace(',', "\\,")),
    );
}

#[test]
fn long_changed_leaf_does_not_unfold_unrelated_nested_branches() {
    let value = "x".repeat(100);
    check(
        "a: [old, {untouched: [1, 2]}]\n",
        &["a", "0"],
        &s(&value),
        &format!("a: [\n    {value}\n    {{untouched: [1, 2]}}\n]\n"),
    );
}

#[test]
fn promoted_siblings_keep_values_when_inline_escapes_lose_meaning() {
    check(
        "a: [old, a\\,b, \\u00342, :: literal]\n",
        &["a", "0"],
        &s("x\ny"),
        "a: [\n    ((\nx\ny\n    ))\n    a,b\n    :: 42\n    :: :: literal\n]\n",
    );
}

#[test]
fn promoted_pairs_keep_key_spelling_and_raw_markers() {
    check(
        "a: {\"x.y\": old, nested.key::  42, n: 0007}\n",
        &["a", "x.y"],
        &s("x\ny"),
        "a: {\n    \"x.y\": ((\nx\ny\n    ))\n    nested.key::  42\n    n: 0007\n}\n",
    );
}

#[test]
fn root_inline_promotion_preserves_bom_and_eof() {
    check(
        "\u{FEFF}[old, keep]",
        &["0"],
        &s("x\ny"),
        "\u{FEFF}[\n    ((\nx\ny\n    ))\n    keep\n]",
    );
}

#[test]
fn promotion_preserves_eol_comments_and_no_final_terminator() {
    for eol in ["\n", "\r", "\r\n"] {
        let source = format!("## header{eol}a: [old, keep]");
        let expected =
            format!("## header{eol}a: [{eol}    (({eol}x{eol}y{eol}    )){eol}    keep{eol}]");
        check(&source, &["a", "0"], &s("x\ny"), &expected);
    }
}

#[test]
fn comments_outside_promoted_inline_value_remain_byte_exact() {
    check(
        "list: [\n  ## before\n  {x: old}\n  ## after\n]\n",
        &["list", "0", "x"],
        &s("x\ny"),
        "list: [\n  ## before\n  {\n      x: ((\nx\ny\n      ))\n  }\n  ## after\n]\n",
    );
}

#[test]
fn noop_ignores_all_readability_thresholds() {
    let source = format!("a: [{}, a\\,b\\,c\\,d\\,e]\n", "x".repeat(140));
    check(&source, &["a", "0"], &s(&"x".repeat(140)), &source);
    check(&source, &["a", "1"], &s("a,b,c,d,e"), &source);
}

#[test]
fn type_change_does_not_force_unrelated_layout_changes() {
    check(
        "a: {x: [1], keep: [2]}\n",
        &["a", "x"],
        &Value::Bool(true),
        "a: {x: true, keep: [2]}\n",
    );
    check("a: [1]\n", &["a"], &s("42"), "a:: 42\n");
}

#[test]
fn reopened_inline_parent_promotes_only_its_physical_definition() {
    check(
        "a: {x: old}\n## gap\na.y: keep\n",
        &["a", "x"],
        &s("x\ny"),
        "a: {\n    x: ((\nx\ny\n    ))\n}\n## gap\na.y: keep\n",
    );
}

#[test]
fn impossible_block_encoding_keeps_valid_inline_form() {
    for value in ["a\rb", "a\n))\nb"] {
        let encoded = value.replace('\r', "\\r").replace('\n', "\\n");
        check(
            "a: [old]\n",
            &["a", "0"],
            &s(value),
            &format!("a: [{encoded}]\n"),
        );
    }
    check(
        "a: [old, a\\rb]\n",
        &["a", "0"],
        &s("x\ny"),
        "a: [x\\ny, a\\rb]\n",
    );
}

#[test]
fn unsafe_comment_loss_still_fails_closed() {
    for source in ["a: {\n ## keep\n x: 1\n}\n", "a: [\n ## keep\n 1\n]\n"] {
        assert!(matches!(
            edit_value_segments(source, &["a"], &Value::Null),
            Err(EditError::Unsupported { .. })
        ));
    }
}
