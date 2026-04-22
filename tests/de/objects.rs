//! Object / nested struct deserialization.

use ktav::from_str;
use serde::Deserialize;

#[test]
fn nested_object_multiline() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Timeouts {
        read: u32,
        write: u32,
    }
    #[derive(Deserialize)]
    struct Cfg {
        timeouts: Timeouts,
    }
    let src = "timeouts: {\n  read: 30\n  write: 10\n}\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(
        cfg.timeouts,
        Timeouts {
            read: 30,
            write: 10
        }
    );
}

#[test]
fn empty_object_inline() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Meta {}
    #[derive(Deserialize)]
    struct Cfg {
        meta: Meta,
    }
    let cfg: Cfg = from_str("meta: {}\n").unwrap();
    assert_eq!(cfg.meta, Meta {});
}

#[test]
fn empty_object_multiline() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Meta {}
    #[derive(Deserialize)]
    struct Cfg {
        meta: Meta,
    }
    let cfg: Cfg = from_str("meta: {\n}\n").unwrap();
    assert_eq!(cfg.meta, Meta {});
}

#[test]
fn missing_optional_field_is_none() {
    #[derive(Deserialize)]
    struct Cfg {
        port: u16,
        label: Option<String>,
    }
    let cfg: Cfg = from_str("port: 1").unwrap();
    assert_eq!(cfg.port, 1);
    assert!(cfg.label.is_none());
}

#[test]
fn dotted_keys_inside_object() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Endpoints {
        api: String,
        admin: String,
    }
    #[derive(Deserialize)]
    struct Server {
        host: String,
        endpoints: Endpoints,
    }
    #[derive(Deserialize)]
    struct Cfg {
        server: Server,
    }
    let src = "\
server: {
    host: 127.0.0.1
    endpoints.api: /v1
    endpoints.admin: /admin
}
";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(cfg.server.host, "127.0.0.1");
    assert_eq!(cfg.server.endpoints.api, "/v1");
    assert_eq!(cfg.server.endpoints.admin, "/admin");
}

#[test]
fn deeply_nested_structs() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct L3 {
        leaf: u16,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct L2 {
        l3: L3,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct L1 {
        l2: L2,
    }
    #[derive(Deserialize)]
    struct Cfg {
        l1: L1,
    }
    let cfg: Cfg = from_str("l1.l2.l3.leaf: 42\n").unwrap();
    assert_eq!(cfg.l1.l2.l3.leaf, 42);
}
