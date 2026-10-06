use super::envelope::line_of_offset;
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

    /// § 3.2/§ 6.15 witness: multibyte content, CRLF terminators, then a
    /// lone continuation byte at offset 33 of 34 (physical line 3).
    fn multibyte_crlf_bad() -> Vec<u8> {
        let mut src = Vec::new();
        src.extend_from_slice(b"name: ");
        src.extend_from_slice("שלום".as_bytes());
        src.extend_from_slice(b"\r\nport: 8080\r\nbad: ");
        src.push(0x80);
        src
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
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["spec_section"], "§6.15");
        // No terminators, first invalid byte at 0: raw line 1.
        assert_eq!(v["line"], 1);
        assert!(v["line_text"].is_null());
        // 0xFF is a 1-byte invalid sequence at offset 0.
        assert_eq!(v["span"]["start"], 0);
        assert_eq!(v["span"]["end"], 1);
        assert!(
            v["body"]
                .as_str()
                .unwrap()
                .starts_with("input is not valid UTF-8"),
            "body = {:?}",
            v["body"]
        );
        assert!(
            v["message"].as_str().unwrap().starts_with("InvalidUtf8"),
            "message = {:?}",
            v["message"]
        );
    }

    #[test]
    fn loads_strict_reports_invalid_utf8_envelope() {
        let err = loads_strict(&multibyte_crlf_bad()).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["line"], 3);
        assert!(v["line_text"].is_null());
        assert_eq!(v["span"]["start"], 33);
        assert_eq!(v["span"]["end"], 34);
        assert_eq!(v["spec_section"], "§6.15");
    }

    #[test]
    fn format_reports_invalid_utf8_envelope() {
        let err = format(&multibyte_crlf_bad()).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["line"], 3);
        assert!(v["line_text"].is_null());
        assert_eq!(v["span"]["start"], 33);
        assert_eq!(v["span"]["end"], 34);
    }

    #[test]
    fn canonical_from_source_reports_invalid_utf8_envelope() {
        let err = canonical_from_source(&multibyte_crlf_bad()).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["line"], 3);
        assert_eq!(v["span"]["start"], 33);
        assert_eq!(v["span"]["end"], 34);
    }

    /// A truncated 3-byte sequence (E1 80) at end of input: error_len is
    /// None, so the span must reach EOF and stay non-empty. The bad bytes
    /// start after the line-1 terminator, on line 2.
    #[test]
    fn loads_invalid_utf8_truncated_sequence_spans_to_eof() {
        let src: &[u8] = b"a: 1\n\xE1\x80";
        let err = loads(src).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["line"], 2);
        assert_eq!(v["span"]["start"], 5);
        assert_eq!(v["span"]["end"], src.len());
        assert!(v["span"]["start"].as_u64().unwrap() < v["span"]["end"].as_u64().unwrap());
    }

    /// E1 80 interrupted by ASCII 'A': error_len is Some(2), span covers
    /// exactly the two bad bytes.
    #[test]
    fn loads_invalid_utf8_interrupted_sequence() {
        let err = loads(b"a: \xE1\x80\x41\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["line"], 1);
        assert_eq!(v["span"]["start"], 3);
        assert_eq!(v["span"]["end"], 5);
    }

    /// C0 AF is an overlong encoding; C0 is an illegal lead byte, the
    /// sequence is 2 bytes long.
    #[test]
    fn loads_invalid_utf8_overlong_two_byte() {
        let err = loads(b"a: \xC0\xAF\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["line"], 1);
        assert_eq!(v["span"]["start"], 3);
        assert_eq!(v["span"]["end"], 5);
    }

    /// A leading BOM (never skipped for raw diagnostics) and lone CR
    /// terminators: bytes are EF BB BF "a: 1" CR "b: 2" CR 80 CR "c: 3";
    /// the 0x80 is at offset 13 on the third physical line.
    #[test]
    fn loads_invalid_utf8_after_bom_and_lone_cr() {
        let src: &[u8] = b"\xEF\xBB\xBFa: 1\rb: 2\r\x80\rc: 3";
        let err = loads(src).unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_eq!(v["line"], 3);
        assert_eq!(v["span"]["start"], 13);
        assert_eq!(v["span"]["end"], 14);
    }

    /// § 6.15: UTF-8 validation happens before any grammar-level
    /// processing, so a prefix that also violates the grammar (the bare
    /// line `just-some-text` would be MissingSeparator) still reports
    /// InvalidUtf8 for the bad byte on line 3.
    #[test]
    fn invalid_utf8_beats_grammar_error_in_valid_prefix() {
        let err = loads(b"anchor: ok\njust-some-text\na: \x80").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "InvalidUtf8");
        assert_ne!(v["error"], "MissingSeparator");
        assert_eq!(v["line"], 3);
        assert_eq!(v["span"]["start"], 29);
        assert_eq!(v["span"]["end"], 30);
    }

    /// § 6: an EOF-detected UnclosedCompound carries a span (the opener
    /// through EOF) but no line in the Error accessors; through the C ABI
    /// the envelope must derive the 1-based opener line from the span.
    #[test]
    fn loads_reports_line_for_unclosed_compound_at_eof() {
        let err = loads(b"a: {\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "UnclosedCompound");
        assert_eq!(v["line"], 1);
        assert!(v["span"].is_object());
        assert_eq!(v["line_text"], "a: {");

        let err = loads(b"x: 1\nobj: {\n").unwrap_err();
        let v = envelope_json(&err);
        assert_eq!(v["error"], "UnclosedCompound");
        assert_eq!(v["line"], 2);
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

    /// § 5.2's redundant-leading-zero exception means `01234` is never
    /// an Integer, so a `$i` payload spelled that way must be
    /// normalized to `1234` at construction time — otherwise the
    /// writer echoes the stored text verbatim (it trusts an `Integer`
    /// is already canonical), and re-parsing `n: 01234` through the
    /// real parser reclassifies it as a String, silently changing the
    /// round-tripped type.
    #[test]
    fn wire_integer_payload_drops_a_redundant_leading_zero() {
        let text = dumps(br#"{"n":{"$i":"01234"}}"#).unwrap();
        let text = std::str::from_utf8(&text).unwrap();
        assert_eq!(text, "n: 1234\n");
        let roundtrip = loads(text.as_bytes()).unwrap();
        assert_eq!(roundtrip, br#"{"n":{"$i":"1234"}}"#);
    }

    #[test]
    fn wire_integer_payload_drops_a_leading_plus_and_folds_signed_zero() {
        let out = loads(
            dumps(br#"{"a":{"$i":"+7"},"b":{"$i":"-0"}}"#)
                .unwrap()
                .as_slice(),
        )
        .unwrap();
        assert_eq!(out, br#"{"a":{"$i":"7"},"b":{"$i":"0"}}"#);
    }

    /// The wire format deliberately supports integers beyond `i64` (the
    /// whole reason `$i` exists instead of a bare JSON number) — a
    /// bignum payload with no leading zero must survive normalization
    /// unchanged rather than overflow. (It does not round-trip back to
    /// Integer through `loads`: the text parser's own Integer grammar
    /// caps at `i64` and falls through to String beyond it, same as
    /// `dumps_big_integer_survives_arbitrary_precision` above already
    /// establishes — this test only pins that normalization itself
    /// does not corrupt or reject the digits on the way out.)
    #[test]
    fn wire_integer_payload_preserves_arbitrary_precision() {
        let big = "123456789012345678901234567890";
        let text = dumps(format!(r#"{{"n":{{"$i":"{big}"}}}}"#).as_bytes()).unwrap();
        assert_eq!(std::str::from_utf8(&text).unwrap(), format!("n: {big}\n"));
    }

    /// Same defect as the Integer case, for Float: both writers trust a
    /// stored `Float` is already `ryu`-canonical and echo it verbatim
    /// (the canonical writer's decimal-region branch is a documented
    /// no-op for exactly this reason), so a non-canonical `$f` spelling
    /// must be normalized at construction time — otherwise
    /// re-canonicalizing the output is not a no-op (idempotent), it is a
    /// second, different answer.
    #[test]
    fn wire_float_payload_normalizes_leading_zero_and_trailing_zero() {
        let leading = emit_canonical(br#"{"n":{"$f":"01.5"}}"#).unwrap();
        assert_eq!(std::str::from_utf8(&leading).unwrap(), "n: 1.5\n");

        let trailing = emit_canonical(br#"{"n":{"$f":"0.50"}}"#).unwrap();
        let text = std::str::from_utf8(&trailing).unwrap();
        assert_eq!(text, "n: 0.5\n");

        // Idempotent: canonicalizing the wire form of the already-canonical
        // output must reproduce the same text, not drift a second time.
        let wire_again = loads(text.as_bytes()).unwrap();
        let again = emit_canonical(&wire_again).unwrap();
        assert_eq!(again, trailing);
    }

    /// A non-finite `$f` payload — `f64::from_str` silently overflows an
    /// extreme exponent like `1e400` to `Infinity` rather than erroring
    /// — is a deliberate wire-only capability, not something to reject
    /// or run through `ryu` (which only documents finite input) — it
    /// must construct successfully. `dumps` (the § 5.9.0 writer) still
    /// rejects a non-finite Float at render time, as it always has;
    /// `dumps_force_strings` bypasses that check, so it is what proves
    /// the value was constructed and carries the payload through
    /// verbatim.
    #[test]
    fn wire_float_payload_passes_non_finite_through_verbatim() {
        assert!(dumps(br#"{"n":{"$f":"1e400"}}"#).is_err());

        let out = dumps_force_strings(br#"{"n":{"$f":"1e400"}}"#).unwrap();
        let text = std::str::from_utf8(&out).unwrap();
        // The forced string's content still matches § 3.6's float
        // grammar, so the writer needs the `::` raw marker to keep a
        // lax reader from reading it back as a number.
        assert_eq!(text, "n:: 1e400\n");
    }

    #[test]
    fn version_bytes_are_nul_terminated() {
        assert!(VERSION_BYTES.ends_with(&[0u8]));
        assert_eq!(
            &VERSION_BYTES[..VERSION_BYTES.len() - 1],
            env!("CARGO_PKG_VERSION").as_bytes()
        );
    }

    // -----------------------------------------------------------------------
    // line_of_offset: the raw-byte line counter (§ 3.2 / § 6.15)
    // -----------------------------------------------------------------------

    #[test]
    fn line_of_offset_empty_and_no_terminator() {
        assert_eq!(line_of_offset(b"", 0), 1);
        assert_eq!(line_of_offset(b"abc", 0), 1);
        // == len: still the (only) line
        assert_eq!(line_of_offset(b"abc", 3), 1);
        // Beyond the end clamps to the end.
        assert_eq!(line_of_offset(b"abc", 99), 1);
    }

    #[test]
    fn line_of_offset_counts_lf_cr_crlf_as_one_terminator() {
        // LF: an offset AT the terminator belongs to the line it closes.
        assert_eq!(line_of_offset(b"a\nb", 1), 1);
        assert_eq!(line_of_offset(b"a\nb", 2), 2);
        // Lone CR, same rule.
        assert_eq!(line_of_offset(b"a\rb", 1), 1);
        assert_eq!(line_of_offset(b"a\rb", 2), 2);
        // CRLF is ONE terminator: the LF byte is still on line 1.
        assert_eq!(line_of_offset(b"a\r\nb", 1), 1);
        assert_eq!(line_of_offset(b"a\r\nb", 2), 1);
        assert_eq!(line_of_offset(b"a\r\nb", 3), 2);
    }

    #[test]
    fn line_of_offset_mixed_terminators() {
        // a \n \r\n \r b: lines are "a", "", "", "b".
        let src = b"a\n\r\n\rb";
        assert_eq!(line_of_offset(src, 0), 1);
        // The CR of the CRLF closes line 2.
        assert_eq!(line_of_offset(src, 2), 2);
        // The lone CR closes line 3.
        assert_eq!(line_of_offset(src, 4), 3);
        assert_eq!(line_of_offset(src, 5), 4);
    }

    #[test]
    fn line_of_offset_after_final_terminator_and_over_bom() {
        // An offset after the final terminator starts the next (empty)
        // line.
        assert_eq!(line_of_offset(b"a\n", 2), 2);
        assert_eq!(line_of_offset(b"a\r\n", 3), 2);
        // The LF byte of a final CRLF is still line 1.
        assert_eq!(line_of_offset(b"a\r\n", 2), 1);
        // A leading BOM is never skipped: its bytes are ordinary line-1
        // content, and the terminator after them is counted at its raw
        // position.
        assert_eq!(line_of_offset(b"\xEF\xBB\xBFa\nb", 1), 1);
        assert_eq!(line_of_offset(b"\xEF\xBB\xBFa\nb", 5), 2);
        assert_eq!(line_of_offset(b"\xEF\xBB\xBF\n", 4), 2);
    }
}
