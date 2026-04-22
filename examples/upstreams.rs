//! Reads an inline Ktav document describing a set of upstream servers,
//! deserializes it, prints the result, then serializes it back. Run with:
//!
//!     cargo run -p ktav --example upstreams

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Timeouts {
    read: u32,
    write: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Upstream {
    host: String,
    port: u16,
    timeouts: Option<Timeouts>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    port: u16,
    banned_patterns: Vec<String>,
    upstreams: Vec<Upstream>,
}

const SOURCE: &str = "\
port: 20082

banned_patterns: [
    .*\\.onion:\\d+
]

upstreams: [
    {
        host: a.example
        port: 1080
        timeouts: {
            read: 30
            write: 10
        }
    }
    {
        host: b.example
        port: 1080
        tags: [
            backup
            eu
        ]
    }
]
";

fn main() {
    let cfg: Config = ktav::from_str(SOURCE).expect("valid Ktav document");
    println!("=== Deserialized ===");
    println!("{:#?}", cfg);

    let back = ktav::to_string(&cfg).expect("serialize");
    println!("\n=== Re-serialized ===");
    println!("{}", back);
}
