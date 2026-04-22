//! Array deserialization.

use ktav::from_str;
use serde::Deserialize;

#[test]
fn multiline_array_of_strings() {
    #[derive(Deserialize)]
    struct Cfg {
        tags: Vec<String>,
    }
    let cfg: Cfg = from_str("tags: [\n  primary\n  eu\n  prod\n]\n").unwrap();
    assert_eq!(cfg.tags, vec!["primary", "eu", "prod"]);
}

#[test]
fn multiline_array_of_integers() {
    #[derive(Deserialize)]
    struct Cfg {
        ports: Vec<u16>,
    }
    let cfg: Cfg = from_str("ports: [\n  80\n  443\n  8080\n]\n").unwrap();
    assert_eq!(cfg.ports, vec![80, 443, 8080]);
}

#[test]
fn empty_array_inline() {
    #[derive(Deserialize)]
    struct Cfg {
        tags: Vec<String>,
    }
    let cfg: Cfg = from_str("tags: []\n").unwrap();
    assert!(cfg.tags.is_empty());
}

#[test]
fn empty_array_multiline() {
    #[derive(Deserialize)]
    struct Cfg {
        tags: Vec<String>,
    }
    let cfg: Cfg = from_str("tags: [\n]\n").unwrap();
    assert!(cfg.tags.is_empty());
}

#[test]
fn missing_array_with_serde_default_is_empty() {
    #[derive(Deserialize)]
    struct Cfg {
        port: u16,
        #[serde(default)]
        tags: Vec<String>,
    }
    let cfg: Cfg = from_str("port: 1").unwrap();
    assert_eq!(cfg.port, 1);
    assert!(cfg.tags.is_empty());
}

#[test]
fn array_of_objects() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Upstream {
        host: String,
        port: u16,
    }
    #[derive(Deserialize)]
    struct Cfg {
        upstreams: Vec<Upstream>,
    }
    let src = "\
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
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.upstreams.len(), 2);
    assert_eq!(
        cfg.upstreams[0],
        Upstream {
            host: "a.example".into(),
            port: 1080
        }
    );
    assert_eq!(
        cfg.upstreams[1],
        Upstream {
            host: "b.example".into(),
            port: 1080
        }
    );
}

#[test]
fn nested_array() {
    #[derive(Deserialize)]
    struct Cfg {
        outer: Vec<Vec<String>>,
    }
    let src = "outer: [\n  [\n    a\n    b\n  ]\n  [\n    c\n  ]\n]\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.outer, vec![vec!["a", "b"], vec!["c"]]);
}

#[test]
fn mixed_scalars_and_objects_in_array() {
    // Heterogeneous arrays need serde's untagged enum on the Rust side.
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(untagged)]
    enum Item {
        Text(String),
        Named { name: String },
    }
    #[derive(Deserialize)]
    struct Cfg {
        items: Vec<Item>,
    }
    let src = "items: [\n  hello\n  {\n    name: world\n  }\n]\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.items[0], Item::Text("hello".into()));
    assert_eq!(
        cfg.items[1],
        Item::Named {
            name: "world".into()
        }
    );
}
