//! Round-trip tests for multi-line string values.

use ktav::{from_str, to_string};
use serde::{Deserialize, Serialize};

#[test]
fn simple_multiline() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "line1\nline2\nline3".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_with_trailing_newline() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "a\nb\n".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_with_indented_content() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        body: String,
    }
    let cfg = Cfg {
        body: "    a\n        b\n    c".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_with_braces_and_brackets_inside() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        json: String,
    }
    let cfg = Cfg {
        json: "{\n  \"arr\": [1, 2, 3]\n}".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_with_hash_inside_does_not_become_comment() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        script: String,
    }
    let cfg = Cfg {
        script: "#!/bin/sh\necho hi".into(),
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn mixed_multiline_and_regular_fields() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        name: String,
        doc: String,
        port: u16,
    }
    let cfg = Cfg {
        name: "demo".into(),
        doc: "a multi\nline doc\n".into(),
        port: 8080,
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}

#[test]
fn multiline_inside_array() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Cfg {
        items: Vec<String>,
    }
    let cfg = Cfg {
        items: vec![
            "simple".into(),
            "multi\nline".into(),
            "another simple".into(),
        ],
    };
    let back: Cfg = from_str(&to_string(&cfg).unwrap()).unwrap();
    assert_eq!(cfg, back);
}
