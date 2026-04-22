//! The `::` raw-string marker — in pair position and array position.

use ktav::from_str;
use serde::Deserialize;

#[test]
fn double_colon_pair_keeps_brackets_as_string() {
    #[derive(Deserialize)]
    struct Cfg {
        regex: String,
    }
    let cfg: Cfg = from_str("regex:: [a-z]+").unwrap();
    assert_eq!(cfg.regex, "[a-z]+");
}

#[test]
fn double_colon_pair_keeps_braces_as_string() {
    #[derive(Deserialize)]
    struct Cfg {
        template: String,
    }
    let cfg: Cfg = from_str("template:: {issue.id}.tpl").unwrap();
    assert_eq!(cfg.template, "{issue.id}.tpl");
}

#[test]
fn double_colon_with_dotted_key() {
    #[derive(Deserialize)]
    struct Pat {
        raw: String,
    }
    #[derive(Deserialize)]
    struct Cfg {
        pattern: Pat,
    }
    let cfg: Cfg = from_str("pattern.raw:: {template}").unwrap();
    assert_eq!(cfg.pattern.raw, "{template}");
}

#[test]
fn double_colon_yields_empty_string_without_content() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x::").unwrap();
    assert_eq!(cfg.x, "");
}

#[test]
fn double_colon_forces_string_for_true_keyword() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x:: true").unwrap();
    assert_eq!(cfg.x, "true");
}

#[test]
fn double_colon_forces_string_for_null_keyword() {
    #[derive(Deserialize)]
    struct Cfg {
        x: String,
    }
    let cfg: Cfg = from_str("x:: null").unwrap();
    assert_eq!(cfg.x, "null");
}

#[test]
fn double_colon_prefix_inside_array_is_literal() {
    #[derive(Deserialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let src = "items: [\n  ok\n  :: [bracketed]\n  :: {braced}\n]\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.items, vec!["ok", "[bracketed]", "{braced}"]);
}

#[test]
fn double_colon_in_array_forces_string_for_keyword() {
    #[derive(Deserialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let cfg: Cfg = from_str("items: [\n  :: true\n  :: null\n  :: false\n]\n").unwrap();
    assert_eq!(cfg.items, vec!["true", "null", "false"]);
}

#[test]
fn standalone_double_colon_in_array_is_empty_string() {
    #[derive(Deserialize)]
    struct Cfg {
        items: Vec<String>,
    }
    let cfg: Cfg = from_str("items: [\n  ::\n  x\n]\n").unwrap();
    assert_eq!(cfg.items, vec!["", "x"]);
}
