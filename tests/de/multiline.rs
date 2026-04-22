//! Multi-line string deserialization: `( ... )` and `(( ... ))`.

use ktav::from_str;
use serde::Deserialize;

#[test]
fn stripped_form_removes_common_indent() {
    #[derive(Deserialize)]
    struct Cfg {
        value: String,
    }
    let src = "value: (\n   {\n     \"qwe\": 1\n   }\n)\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.value, "{\n  \"qwe\": 1\n}");
}

#[test]
fn stripped_form_with_flush_content_leaves_content_intact() {
    #[derive(Deserialize)]
    struct Cfg {
        note: String,
    }
    let src = "note: (\nline1\nline2\n)\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.note, "line1\nline2");
}

#[test]
fn verbatim_form_preserves_original_indentation() {
    #[derive(Deserialize)]
    struct Cfg {
        block: String,
    }
    let src = "block: ((\n    a\n        b\n    c\n))\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.block, "    a\n        b\n    c");
}

#[test]
fn empty_multiline_inline_is_empty_string() {
    #[derive(Deserialize)]
    struct Cfg {
        a: String,
        b: String,
    }
    let cfg: Cfg = from_str("a: ()\nb: (())\n").unwrap();
    assert_eq!(cfg.a, "");
    assert_eq!(cfg.b, "");
}

#[test]
fn multiline_with_no_content_lines_is_empty_string() {
    #[derive(Deserialize)]
    struct Cfg {
        a: String,
    }
    let cfg: Cfg = from_str("a: (\n)\n").unwrap();
    assert_eq!(cfg.a, "");
}

#[test]
fn multiline_skips_bracket_and_hash_interpretation() {
    // Inside the multi-line block, `{` / `[` / `#` are just content — no
    // compound parsing, no comment skipping.
    #[derive(Deserialize)]
    struct Cfg {
        body: String,
    }
    let src = "body: (\n{\n# not a comment\n[x]\n}\n)\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.body, "{\n# not a comment\n[x]\n}");
}

#[test]
fn multiline_as_array_item() {
    #[derive(Deserialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let src = "items: [\n    simple\n    (\n        block\n        of lines\n    )\n    after\n]\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.items, vec!["simple", "block\nof lines", "after"]);
}

#[test]
fn verbatim_as_array_item_preserves_indent() {
    #[derive(Deserialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let src = "items: [\n    ((\n      indented\n        more\n    ))\n]\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.items, vec!["      indented\n        more"]);
}

#[test]
fn unclosed_multiline_errors() {
    #[derive(Debug, Deserialize)]
    struct Cfg {
        _x: String,
    }
    let err = from_str::<Cfg>("x: (\ncontent\n").unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Unclosed multi-line"), "got: {}", msg);
}

#[test]
fn preserves_relative_indent_inside_stripped() {
    #[derive(Deserialize)]
    struct Cfg {
        value: String,
    }
    // Common indent of non-blank lines is 4 spaces; the inner line with
    // 6 extra spaces keeps 2 spaces after stripping.
    let src = "value: (\n    root\n      child\n    back\n)\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.value, "root\n  child\nback");
}
