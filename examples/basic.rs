//! End-to-end demo: parse a Ktav document into a typed struct, walk
//! the dynamic [`Value`] tree, then build a fresh document in Rust and
//! render it back to Ktav text.
//!
//! Run with:
//!
//!     cargo run -p ktav --example basic

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use ktav::value::{ObjectMap, Value};

const SRC: &str = "\
service: web
port:i 8080
ratio:f 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout:i 30
";

#[derive(Debug, Deserialize, Serialize)]
struct Db {
    host: String,
    timeout: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port: u16,
    ratio: f64,
    tls: bool,
    tags: Vec<String>,
    db: Db,
}

fn main() {
    // ── 1. Decode straight into a typed struct via serde. ──────────────
    let cfg: Config = ktav::from_str(SRC).expect("valid Ktav");
    println!(
        "service={} port={} tls={} ratio={:.2}",
        cfg.service, cfg.port, cfg.tls, cfg.ratio
    );
    println!("tags={:?}", cfg.tags);
    println!("db: {} (timeout={}s)\n", cfg.db.host, cfg.db.timeout);

    // ── 2. Or work with the dynamic Value tree, matching the variants. ─
    let dyn_val = ktav::parse(SRC).expect("valid Ktav");
    let Value::Object(top) = &dyn_val else {
        unreachable!("top-level is always an object");
    };
    println!("shape:");
    for (k, v) in top {
        println!("  {:<12} -> {}", k, describe(v));
    }

    // ── 3. Build a config in code, render it as Ktav text. ─────────────
    let upstreams = vec![
        upstream("a.example", 1080),
        upstream("b.example", 1080),
        upstream("c.example", 1080),
    ];

    let mut top = obj_map();
    top.insert("name".into(), Value::String("frontend".into()));
    top.insert("port".into(), Value::Integer("8443".into()));
    top.insert("tls".into(), Value::Bool(true));
    top.insert("ratio".into(), Value::Float("0.95".into()));
    top.insert("upstreams".into(), Value::Array(upstreams));
    top.insert("notes".into(), Value::Null);

    let rendered = ktav::render::render(&Value::Object(top)).expect("render");
    println!("\n--- rendered ---");
    print!("{}", rendered);
}

fn describe(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::Bool(b) => format!("bool={b}"),
        Value::Integer(s) => format!("int={s}"),
        Value::Float(s) => format!("float={s}"),
        Value::String(s) => format!("str={s:?}"),
        Value::Array(a) => format!("array({})", a.len()),
        Value::Object(o) => format!("object({})", o.len()),
    }
}

fn upstream(host: &str, port: u16) -> Value {
    let mut m = obj_map();
    m.insert("host".into(), Value::String(host.into()));
    m.insert("port".into(), Value::Integer(port.to_string().into()));
    Value::Object(m)
}

fn obj_map() -> ObjectMap {
    // ObjectMap is the crate's IndexMap alias preserving insertion order.
    let _ = IndexMap::<String, Value>::new();
    ObjectMap::default()
}
