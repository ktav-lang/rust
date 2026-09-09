//! The example shown in `src/lib.rs` module documentation, extracted as
//! a regular integration test. Kept in sync with the rendered doc snippet
//! (marked `rust,ignore` there so it is not a doctest).

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Upstream {
    host: String,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Config {
    port: u16,
    upstreams: Vec<Upstream>,
}

#[test]
fn lib_doc_example_round_trips() {
    let text = "\
port: 8080

upstreams: [
    {
        host: a.example
        port: 1080
    }
    {
        host: b.example
        port: 1080
    }
]
";
    let cfg: Config = ktav::from_str(text).unwrap();
    assert_eq!(cfg.port, 8080);
    assert_eq!(cfg.upstreams.len(), 2);

    let back = ktav::to_string(&cfg).unwrap();
    let round: Config = ktav::from_str(&back).unwrap();
    assert_eq!(cfg, round);
}

// Pins the crate-level rustdoc's Syntax section (single `#` is content,
// only `##` is a comment, § 3.4) so it can't silently drift from the parser
// again (R12-F3).
#[test]
fn lib_doc_single_hash_is_content_not_comment() {
    let text = "# comment\nkey: value\n";
    let value = ktav::parse(text).unwrap();
    assert_eq!(
        value,
        ktav::Value::Array(vec![
            ktav::Value::String("# comment".into()),
            ktav::Value::String("key: value".into()),
        ])
    );
}

#[test]
fn lib_doc_double_hash_is_comment() {
    let text = "## comment\nkey: value\n";
    let value = ktav::parse(text).unwrap();
    let mut expected = ktav::ObjectMap::with_hasher(Default::default());
    expected.insert("key".into(), ktav::Value::String("value".into()));
    assert_eq!(value, ktav::Value::Object(expected));
}
