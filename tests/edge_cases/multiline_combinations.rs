//! Multi-line string combined with other tricky content.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn multiline_containing_brackets_and_hash_round_trips() {
    // Inside a multi-line block `{`, `[`, `#` are just bytes — no
    // compound parsing, no comment skipping.
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        doc: String,
    }
    let cfg = Cfg {
        doc: "# not a comment\n{\n[array]\n}\nend".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_with_lines_that_look_like_double_colon_prefix() {
    // `:: foo` inside a multi-line block is not a raw-marker — it's
    // just text content.
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        doc: String,
    }
    let cfg = Cfg {
        doc: ":: pretending to be raw\nand normal text".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_inside_deeply_nested_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Leaf {
        body: String,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Branch {
        leaf: Leaf,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Root {
        branch: Branch,
    }

    let cfg = Root {
        branch: Branch {
            leaf: Leaf {
                body: "line1\nline2\nline3".into(),
            },
        },
    };
    let s = to_string(&cfg).unwrap();
    // The `((` / `))` markers should be present.
    assert!(s.contains("((\n"));
    assert!(s.contains("))\n"));
    let back: Root = from_str(&s).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_in_array_item_surrounded_by_scalars() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        items: Vec<String>,
    }
    let cfg = Cfg {
        items: vec![
            "one".into(),
            "two\nlines".into(),
            "three".into(),
            "four\nlines\nhere".into(),
            "five".into(),
        ],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_with_empty_lines_in_middle() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "top\n\n\nbottom".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn stripped_form_with_tabs_in_indent() {
    // Parser dedents based on ASCII whitespace bytes. Tabs are treated
    // as single-byte whitespace — common prefix is compared byte-by-byte.
    #[derive(Debug, Deserialize, PartialEq)]
    struct Cfg {
        body: String,
    }
    let src = "body: (\n\t\tfirst\n\t\t\tindented\n\t\tback\n)\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.body, "first\n\tindented\nback");
}

#[test]
fn verbatim_preserves_mixed_indentation_exactly() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Cfg {
        body: String,
    }
    let src = "body: ((\n   three\n  two\n one\n))\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.body, "   three\n  two\n one");
}
