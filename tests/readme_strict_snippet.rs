//! Verifies the strict-mode snippet shown in the READMEs (en/ru/zh)
//! actually compiles and passes, mirroring the `doc_example.rs`
//! convention of keeping rendered doc snippets under test.

use ktav::{parse, parse_strict, Error, ErrorKind, Value};

#[test]
fn readme_strict_mode_snippet() {
    let src = "version: 1.10\n";

    assert!(parse(src).is_ok()); // Float(1.1) — trailing zero gone

    // § 5.2's one exception: a redundant leading zero is never inferred as
    // a number, so the lax parse keeps the identifier intact and the
    // strict parse has nothing to reject.
    let zip = parse("zip: 01234\n").expect("a leading-zero decimal is an ordinary String");
    let Value::Object(root) = &zip else {
        panic!("expected an Object root, got {zip:?}");
    };
    assert_eq!(root.get("zip"), Some(&Value::String("01234".into())));
    assert!(parse_strict("zip: 01234\n").is_ok());

    match parse_strict(src) {
        Err(Error::Structured(ErrorKind::LossyScalar {
            body, canonical, ..
        })) => {
            assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
        }
        other => panic!("expected LossyScalar, got {other:?}"),
    }
}
