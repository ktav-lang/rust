use super::*;
use crate::value::Value;
use serde_json::Value as Json;

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope_json(payload: &str) -> Json {
        let v: Json = serde_json::from_str(payload)
            .unwrap_or_else(|e| panic!("Err payload is not JSON: {e}: {payload}"));
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            [
                "error",
                "reason",
                "line",
                "line_text",
                "span",
                "path",
                "body",
                "canonical",
                "spec_section",
                "message",
            ]
        );
        v
    }

    #[test]
    fn abi_version_is_deliberate() {
        // Changing this value is a breaking change for every host: update
        // this test and the module docs together with the bump.
        assert_eq!(ABI_VERSION, 1);
    }

    #[test]
    fn library_file_name_matches_the_binding_loaders() {
        assert_eq!(
            library_file_name("windows", "amd64").as_deref(),
            Some("ktav_cabi-windows-amd64.dll")
        );
        assert_eq!(
            library_file_name("windows", "arm64").as_deref(),
            Some("ktav_cabi-windows-arm64.dll")
        );
        assert_eq!(
            library_file_name("macos", "amd64").as_deref(),
            Some("libktav_cabi-darwin-amd64.dylib")
        );
        assert_eq!(
            library_file_name("darwin", "arm64").as_deref(),
            Some("libktav_cabi-darwin-arm64.dylib")
        );
        assert_eq!(
            library_file_name("linux", "amd64").as_deref(),
            Some("libktav_cabi-linux-amd64.so")
        );
        assert_eq!(
            library_file_name("linux", "arm64").as_deref(),
            Some("libktav_cabi-linux-arm64.so")
        );
        // Rust-const target spellings alias onto the same files.
        assert_eq!(
            library_file_name("linux", "x86_64").as_deref(),
            Some("libktav_cabi-linux-amd64.so")
        );
        assert_eq!(
            library_file_name("macos", "aarch64").as_deref(),
            Some("libktav_cabi-darwin-arm64.dylib")
        );
        assert_eq!(library_file_name("freebsd", "amd64"), None);
        assert_eq!(library_file_name("linux", "riscv64"), None);
    }

    #[test]
    fn loads_wire_form_pins_order_and_tags() {
        let out = loads(b"port: 8080\nhost: a.example\n").unwrap();
        assert_eq!(out, br#"{"port":{"$i":"8080"},"host":"a.example"}"#);
    }

    #[test]
    fn loads_reports_envelope_on_parse_error() {
        let err = loads(b"anchor: ok\nport:8080\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "MissingSeparatorSpace");
        assert_eq!(v["line"], 2);
        assert_eq!(v["line_text"], "port:8080");
        assert!(v["span"].is_object());
        assert!(v["body"].is_null());
        // `MissingSeparatorSpace` has a governing spec section, so the
        // envelope populates it rather than emitting null.
        assert_eq!(v["spec_section"], "§6.10");
    }

    #[test]
    fn loads_reports_envelope_on_invalid_utf8() {
        let err = loads(&[0xFF, 0xFE]).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "Message");
        assert!(
            v["body"]
                .as_str()
                .unwrap()
                .starts_with("input is not valid UTF-8"),
            "body = {:?}",
            v["body"]
        );
        assert!(v["line"].is_null());
        assert!(v["line_text"].is_null());
        assert!(v["span"].is_null());
    }

    #[test]
    fn loads_strict_reports_lossy_scalar_envelope() {
        let err = loads_strict(b"a: 1.10\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "LossyScalar");
        assert_eq!(v["body"], "1.10");
        assert_eq!(v["canonical"], "1.1");
        assert_eq!(v["line"], 1);
    }

    /// A writer refusal must carry its § 5.9.0 reason code through the
    /// ABI, not just a class name. Four bindings tested this in their own
    /// shims; it belongs here now that the shims are one line each.
    #[test]
    fn dumps_writer_refusal_carries_a_reason_code() {
        // An empty key name is unrepresentable (§ 5.9.0 EmptyKeyName):
        // it parses as wire JSON but no writer can emit it.
        let err = dumps(br#"{"a":{"":{"$i":"1"}}}"#).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["reason"], "EmptyKeyName");
        assert!(
            v["error"] == "Unrepresentable" || v["error"] == "UnrepresentableAt",
            "error = {:?}, expected one of the two writer rejections",
            v["error"]
        );
    }

    /// For a document with no comments and no blank lines, formatting it
    /// and canonically emitting its parse must agree — the two ABI
    /// operations take different inputs (Ktav text vs wire JSON) and are
    /// easy to let drift apart. Four bindings asserted this
    /// independently before the shims collapsed.
    #[test]
    fn format_agrees_with_emit_canonical_on_a_trivia_free_document() {
        let src: &[u8] = b"server: {host: a, port: 80}\nname: x\n";
        let formatted = format(src).unwrap();
        let wire = loads(src).unwrap();
        let canonical = emit_canonical(&wire).unwrap();
        assert_eq!(
            std::str::from_utf8(&formatted).unwrap(),
            std::str::from_utf8(&canonical).unwrap()
        );
    }

    #[test]
    fn dumps_error_payloads_are_envelopes() {
        let err = dumps(b"{").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "Message");
        assert!(
            v["body"].as_str().unwrap().starts_with("input JSON"),
            "body = {:?}",
            v["body"]
        );

        let err = dumps(br#""just a string""#).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(
            v["body"],
            "top-level Ktav document must be an object or array"
        );
    }

    #[test]
    fn dumps_bare_float_survives_arbitrary_precision() {
        let text = dumps(br#"{"r": 3.5}"#).unwrap();
        let text = std::str::from_utf8(&text).unwrap();
        assert!(text.contains("r: 3.5"), "text = {text:?}");
        assert_eq!(
            crate::parse(text).unwrap(),
            crate::parse("r: 3.5\n").unwrap()
        );
    }

    #[test]
    fn dumps_big_integer_survives_arbitrary_precision() {
        let text = dumps(br#"{"n": 123456789012345678901234567890}"#).unwrap();
        let text = std::str::from_utf8(&text).unwrap();
        assert!(
            text.contains("123456789012345678901234567890"),
            "text = {text:?}"
        );
        assert_eq!(
            crate::parse(text).unwrap(),
            crate::parse("n: 123456789012345678901234567890\n").unwrap()
        );
    }

    #[test]
    fn emit_canonical_smoke_and_force_strings_coercion() {
        let wire = br#"{"a":{"$i":"7"},"b":[true,null,"s"]}"#;
        let canon = emit_canonical(wire).unwrap();
        let text = std::str::from_utf8(&canon).unwrap();
        let from_canonical = crate::parse(text).unwrap();
        // `loads` re-encodes to wire form; `dumps` produces the Ktav text
        // that both must agree on when parsed back.
        let dumped = dumps(wire).unwrap();
        let from_dumps = crate::parse(std::str::from_utf8(&dumped).unwrap()).unwrap();
        assert_eq!(from_canonical, from_dumps);

        let forced = dumps_force_strings(br#"{"a":{"$i":"7"}}"#).unwrap();
        let text = std::str::from_utf8(&forced).unwrap();
        let v = crate::parse(text).unwrap();
        match v {
            Value::Object(o) => assert!(matches!(o.get("a"), Some(Value::String(_)))),
            other => panic!("expected object, got {other:?}"),
        }
    }

    #[test]
    fn format_is_a_fixed_point_and_normalises_inline() {
        let src: &[u8] = b"a: {x: 1}\n";
        let out = format(src).unwrap();
        assert_ne!(out, src, "inline compound should be normalised");
        let out_text = std::str::from_utf8(&out).unwrap();
        crate::parse(out_text).unwrap();
        let again = format(&out).unwrap();
        assert_eq!(again, out, "format must be a fixed point");

        // Comments survive formatting, in place.
        let out = format(b"## c\na: 1\n").unwrap();
        assert!(out.starts_with(b"## c\n"), "out = {out:?}");
    }

    #[test]
    fn format_reports_envelope_with_source_position() {
        let err = format(b"anchor: ok\nport:8080\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "MissingSeparatorSpace");
        assert_eq!(v["line"], 2);
        assert_eq!(v["line_text"], "port:8080");
    }

    #[test]
    fn version_bytes_are_nul_terminated() {
        assert!(VERSION_BYTES.ends_with(&[0u8]));
        assert_eq!(
            &VERSION_BYTES[..VERSION_BYTES.len() - 1],
            env!("CARGO_PKG_VERSION").as_bytes()
        );
    }
}
