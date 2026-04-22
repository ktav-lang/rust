//! Object serialization.

use ktav::to_string;
use serde::Serialize;

#[test]
fn empty_object_inline() {
    #[derive(Serialize)]
    struct Empty {}
    #[derive(Serialize)]
    struct Cfg {
        meta: Empty,
    }
    let s = to_string(&Cfg { meta: Empty {} }).unwrap();
    assert_eq!(s, "meta: {}\n");
}

#[test]
fn nested_struct_multiline() {
    #[derive(Serialize)]
    struct Timeouts {
        read: u32,
        write: u32,
    }
    #[derive(Serialize)]
    struct Cfg {
        timeouts: Timeouts,
    }
    let s = to_string(&Cfg {
        timeouts: Timeouts {
            read: 30,
            write: 10,
        },
    })
    .unwrap();
    assert_eq!(s, "timeouts: {\n    read:i 30\n    write:i 10\n}\n");
}

#[test]
fn deep_nested_struct() {
    #[derive(Serialize)]
    struct Inner {
        leaf: u16,
    }
    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
    }
    #[derive(Serialize)]
    struct Cfg {
        outer: Outer,
    }
    let s = to_string(&Cfg {
        outer: Outer {
            inner: Inner { leaf: 42 },
        },
    })
    .unwrap();
    assert_eq!(s, "outer: {\n    inner: {\n        leaf:i 42\n    }\n}\n");
}

#[test]
fn struct_field_order_is_preserved() {
    #[derive(Serialize)]
    struct Cfg {
        z: u16,
        a: u16,
        m: u16,
    }
    let s = to_string(&Cfg { z: 1, a: 2, m: 3 }).unwrap();
    assert_eq!(s, "z:i 1\na:i 2\nm:i 3\n");
}
