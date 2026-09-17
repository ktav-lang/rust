//! Pins the README blocks that were still unverified prose: walking a
//! parsed document, building one in code, the streaming event API, the
//! duplicate-key error example, and the rendered shape of serde enums.
//!
//! Part of issue rust#265. The README carries 95 fenced blocks and only
//! a handful were executed anywhere; every one of those that makes a
//! behavioural claim is a claim a reader will act on.

use serde::{Deserialize, Serialize};

use ktav::value::{ObjectMap, Value};
use ktav::{parse, parse_events, Error, ErrorKind, ParseEvent};

/// The `SRC` constant the README's parse and walk examples share.
const SRC: &str = "\
service: web
port: 8080
ratio: 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout: 30
";

/// `### Walk — dispatch on the runtime type`. The block matches on every
/// `Value` arm; pin the classification it produces so a change to the
/// enum cannot leave the README describing arms that no longer exist.
#[test]
fn readme_walk_example_classifies_every_arm() {
    let v = parse(SRC).expect("README SRC must parse");
    let Value::Object(top) = &v else {
        panic!("the README says the top level is always an object");
    };

    let described: Vec<String> = top
        .iter()
        .map(|(k, v)| {
            let kind = match v {
                Value::Null => "null".to_string(),
                Value::Bool(b) => format!("bool={b}"),
                Value::Integer(s) => format!("int={s}"),
                Value::Float(s) => format!("float={s}"),
                Value::String(s) => format!("str={s:?}"),
                Value::Array(a) => format!("array({})", a.len()),
                Value::Object(o) => format!("object({})", o.len()),
            };
            format!("{k} -> {kind}")
        })
        .collect();

    assert_eq!(
        described,
        vec![
            r#"service -> str="web""#.to_string(),
            "port -> int=8080".to_string(),
            "ratio -> float=0.75".to_string(),
            "tls -> bool=true".to_string(),
            "tags -> array(2)".to_string(),
            "db -> object(2)".to_string(),
        ],
        "the walk example's output changed"
    );
}

/// `### Build & render — construct a document in code`. The README shows
/// the construction but no output; assert the document round-trips, which
/// is the property a reader is really relying on.
#[test]
fn readme_build_example_round_trips() {
    let mut top = ObjectMap::default();
    top.insert("name".into(), Value::String("frontend".into()));
    top.insert("port".into(), Value::Integer("8443".into()));
    top.insert("tls".into(), Value::Bool(true));
    top.insert("ratio".into(), Value::Float("0.95".into()));
    top.insert("notes".into(), Value::Null);

    let built = Value::Object(top);
    let text = ktav::render::render(&built).expect("README build example must render");
    assert_eq!(parse(&text).expect("rendered text must parse back"), built);

    // Declaration order survives, which the README states a few lines
    // further down under "Serialization preserves: field order".
    assert_eq!(
        text,
        "name: frontend\nport: 8443\ntls: true\nratio: 0.95\nnotes: null\n"
    );
}

/// The streaming-API block, whose assertion is written out in the README
/// itself. Running it here means the README's own `assert_eq!` is true.
#[test]
fn readme_parse_events_example() {
    let src = "port: 8080\nhost: example.com\n";
    let mut keys = Vec::new();
    parse_events(src, |ev| {
        if let ParseEvent::Key(k) = ev {
            keys.push(k.to_string());
        }
    })
    .expect("README event example must parse");
    assert_eq!(keys, ["port", "host"]);
}

/// `### Inspect errors`. The block prints `span.slice(src)` and claims
/// `Some("port")`; that claim is the reason a reader would reach for the
/// span at all.
#[test]
fn readme_duplicate_key_example() {
    let src = "port: 80\nport: 443\n";
    match parse(src) {
        Err(Error::Structured(ErrorKind::DuplicateKey {
            line, key, span, ..
        })) => {
            assert_eq!(line, 2);
            assert_eq!(key.as_str(), "port");
            assert_eq!(span.slice(src), Some("port"));
            // 1-based line, 0-based byte column, as the comment says.
            assert_eq!(span.line_col(src), (2, 0));
        }
        other => panic!("expected a DuplicateKey error, got {other:?}"),
    }
}

/// `## Examples: Ktav → JSON5` pairs a Ktav block with the JSON5 it is
/// equivalent to. Rather than hand-copying eleven expected values here —
/// which would drift from the document — this reads the README itself
/// and asserts every Ktav half parses.
///
/// Parsing is the claim most likely to be wrong and the cheapest to
/// check: an example that does not parse is worse than no example. The
/// JSON5 halves are not parsed (that would mean a JSON5 dependency for
/// documentation alone); their structure is covered by the dedicated
/// example tests elsewhere in tests/.
#[test]
fn readme_json5_comparison_examples_all_parse() {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("README.md must be readable next to Cargo.toml");

    let lines: Vec<&str> = readme.lines().collect();
    let mut examples: Vec<(usize, String)> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim_end() == "```text" {
            let start = i + 1;
            let mut end = start;
            while end < lines.len() && !lines[end].starts_with("```") {
                end += 1;
            }
            // Only a text block whose next non-blank line opens a json5
            // block is one of the Ktav/JSON5 comparison pairs; the other
            // `text` blocks in this document are output samples. Some
            // pairs have a blank line between the two blocks, so "next
            // non-blank" rather than "immediately next".
            let mut peek = end + 1;
            while peek < lines.len() && lines[peek].trim().is_empty() {
                peek += 1;
            }
            if peek < lines.len() && lines[peek].trim_end() == "```json5" {
                examples.push((start + 1, lines[start..end].join("\n") + "\n"));
            }
            i = end + 1;
            continue;
        }
        i += 1;
    }

    // Eight, counted by running this. Not every json5 block in the
    // document is half of a pair: one compares two JSON5 spellings to
    // each other, and one is separated from the preceding Ktav block by
    // prose and illustrates something else entirely. The floor is here
    // so a restructure that stops the scanner finding the section fails
    // loudly instead of silently checking nothing.
    assert_eq!(
        examples.len(),
        8,
        "expected 8 Ktav/JSON5 comparison pairs in the README, found {}; \
         the section changed shape — re-count deliberately rather than \
         relaxing this number",
        examples.len()
    );

    for (line, src) in &examples {
        if let Err(e) = parse(src) {
            panic!("README example starting at line {line} does not parse: {e}\n{src}");
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Fast,
    #[allow(dead_code)]
    Slow,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Action {
    Log(String),
    #[allow(dead_code)]
    Count(u32),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Enums {
    mode: Mode,
    action: Action,
}

/// `### Enums`. The README shows the rendered form in a `text` block —
/// a unit variant as a bare name, a newtype variant as a single-entry
/// object. Both are pinned here because the rendered shape is the whole
/// content of that section.
#[test]
fn readme_enum_rendering() {
    let value = Enums {
        mode: Mode::Fast,
        action: Action::Log("hello".into()),
    };
    let text = ktav::to_string(&value).expect("enums must render");

    assert_eq!(text, "mode: fast\naction: {\n    Log: hello\n}\n");

    let back: Enums = ktav::from_str(&text).expect("enums must round-trip");
    assert_eq!(back, value);
}
