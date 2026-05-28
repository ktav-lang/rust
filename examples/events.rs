//! Demonstrates the public event-based parser: `parse_events` invokes
//! a callback for each parse event, with strings borrowed directly into
//! the input buffer — no allocation per event, no intermediate `Value`
//! tree.
//!
//! Useful when you don't need the full document in memory: counting
//! keys, streaming to a different format, or building a custom shape.
//!
//! Run with:
//!
//!     cargo run -p ktav --example events

use ktav::{parse_events, ParseEvent};

const SRC: &str = "\
service: web
port: 8080
tls: true
upstreams: [
    a.example
    b.example
]
db: {
    host: primary.internal
    timeout: 30
}
";

fn main() {
    // ── 1. Just collect top-level keys without building a tree. ────────
    let mut depth = 0u32;
    let mut top_keys: Vec<String> = Vec::new();
    parse_events(SRC, |ev| match ev {
        ParseEvent::BeginObject | ParseEvent::BeginArray => depth += 1,
        ParseEvent::EndObject | ParseEvent::EndArray => depth -= 1,
        // depth == 1 is inside the implicit top-level object, before
        // any nested compound has opened.
        ParseEvent::Key(k) if depth == 1 => top_keys.push(k.to_string()),
        _ => {}
    })
    .expect("valid Ktav");
    println!("top-level keys: {top_keys:?}");

    // ── 2. Print every event with its kind. ────────────────────────────
    println!("\nevent stream:");
    let mut indent = 0;
    parse_events(SRC, |ev| {
        match &ev {
            ParseEvent::EndObject | ParseEvent::EndArray => indent -= 1,
            _ => {}
        }
        let pad = "  ".repeat(indent.max(0) as usize);
        match ev {
            ParseEvent::Null => println!("{pad}null"),
            ParseEvent::Bool(b) => println!("{pad}bool={b}"),
            ParseEvent::Integer(s) => println!("{pad}int={s}"),
            ParseEvent::Float(s) => println!("{pad}float={s}"),
            ParseEvent::Str(s) => println!("{pad}str={s:?}"),
            ParseEvent::Key(k) => println!("{pad}key={k:?}"),
            ParseEvent::BeginObject => {
                println!("{pad}{{");
                indent += 1;
            }
            ParseEvent::EndObject => println!("{pad}}}"),
            ParseEvent::BeginArray => {
                println!("{pad}[");
                indent += 1;
            }
            ParseEvent::EndArray => println!("{pad}]"),
            _ => println!("{pad}<unknown — ParseEvent is #[non_exhaustive]>"),
        }
    })
    .expect("valid Ktav");
}
