//! Verifies the strict-mode snippet shown in the READMEs (en/ru/zh)
//! actually compiles and passes, mirroring the `doc_example.rs`
//! convention of keeping rendered doc snippets under test.

use ktav::{parse, parse_strict, Error, ErrorKind};

#[test]
fn readme_strict_mode_snippet() {
    let src = "zip: 01234\n";

    assert!(parse(src).is_ok()); // Integer(1234) — leading zero gone

    match parse_strict(src) {
        Err(Error::Structured(ErrorKind::LossyScalar {
            body, canonical, ..
        })) => {
            assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
        }
        other => panic!("expected LossyScalar, got {other:?}"),
    }
}
