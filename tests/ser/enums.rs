//! Enum serialization (externally-tagged).

use ktav::to_string;
use serde::Serialize;

#[test]
fn unit_variant_emits_name() {
    #[derive(Serialize)]
    #[serde(rename_all = "lowercase")]
    enum Mode {
        Fast,
        Slow,
    }
    #[derive(Serialize)]
    struct Cfg {
        mode: Mode,
    }
    let s = to_string(&Cfg { mode: Mode::Fast }).unwrap();
    assert_eq!(s, "mode: fast\n");
    let s = to_string(&Cfg { mode: Mode::Slow }).unwrap();
    assert_eq!(s, "mode: slow\n");
}

#[test]
fn newtype_variant_emits_single_entry_object() {
    #[derive(Serialize)]
    enum Action {
        Log(String),
        Count(u32),
    }
    #[derive(Serialize)]
    struct Cfg {
        action: Action,
    }

    let s = to_string(&Cfg {
        action: Action::Log("hello".into()),
    })
    .unwrap();
    assert_eq!(s, "action: {\n    Log: hello\n}\n");

    let s = to_string(&Cfg {
        action: Action::Count(7),
    })
    .unwrap();
    assert_eq!(s, "action: {\n    Count:i 7\n}\n");
}

#[test]
fn struct_variant_emits_single_entry_object() {
    #[derive(Serialize)]
    enum Message {
        Greet { who: String },
    }
    #[derive(Serialize)]
    struct Cfg {
        msg: Message,
    }

    let s = to_string(&Cfg {
        msg: Message::Greet {
            who: "world".into(),
        },
    })
    .unwrap();
    assert_eq!(s, "msg: {\n    Greet: {\n        who: world\n    }\n}\n");
}
