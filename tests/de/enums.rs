//! Enum deserialization (externally-tagged, serde's default).

use ktav::from_str;
use serde::Deserialize;

#[test]
fn unit_variant_from_string() {
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Mode {
        Fast,
        Slow,
    }
    #[derive(Deserialize)]
    struct Cfg {
        mode: Mode,
    }
    let cfg: Cfg = from_str("mode: fast").unwrap();
    assert_eq!(cfg.mode, Mode::Fast);
}

#[test]
fn newtype_variant_from_single_entry_object() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Action {
        Log(String),
        Count(u32),
    }
    #[derive(Deserialize)]
    struct Cfg {
        action: Action,
    }

    let cfg: Cfg = from_str("action: {\n  Log: hello\n}\n").unwrap();
    assert_eq!(cfg.action, Action::Log("hello".to_string()));

    let cfg: Cfg = from_str("action: {\n  Count: 7\n}\n").unwrap();
    assert_eq!(cfg.action, Action::Count(7));
}

#[test]
fn struct_variant_from_single_entry_object() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Message {
        Greet { who: String },
    }
    #[derive(Deserialize)]
    struct Cfg {
        msg: Message,
    }
    let src = "msg: {\n  Greet: {\n    who: world\n  }\n}\n";
    let cfg: Cfg = from_str(src).unwrap();
    assert_eq!(
        cfg.msg,
        Message::Greet {
            who: "world".into()
        }
    );
}
