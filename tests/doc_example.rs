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
