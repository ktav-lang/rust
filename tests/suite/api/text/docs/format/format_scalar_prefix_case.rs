//! `format_str` and `emit_canonical` on uppercase base prefixes (§ 3.6
//! defines the prefixes lowercase-only; § 5.2 rule 13 matches numeric
//! literals case-sensitively): `0X1A` is a String, so both writers must
//! keep it bare — never rewritten to the integer it is not, never forced
//! into a `::` marker — and stay idempotent.

#[test]
fn format_str_keeps_uppercase_prefix_bare_and_idempotent() {
    for src in ["a: 0X1A\n", "a: -0XFF\n", "a: +0XFF\n"] {
        let once = ktav::format_str(src).unwrap_or_else(|e| panic!("format_str({src:?}): {e}"));
        assert_eq!(once, src, "uppercase-prefixed String must survive verbatim");
        let twice = ktav::format_str(&once).unwrap_or_else(|e| panic!("format_str({once:?}): {e}"));
        assert_eq!(twice, once, "format_str must be idempotent");
    }
}

#[test]
fn emit_canonical_keeps_uppercase_prefix_bare_and_idempotent() {
    let v = ktav::parse("a: 0X1A\n").unwrap();
    let canon = ktav::render::emit_canonical(&v).unwrap();
    assert_eq!(canon, "a: 0X1A\n");
    let back = ktav::parse(&canon).unwrap();
    assert_eq!(back, v);
    assert_eq!(ktav::render::emit_canonical(&back).unwrap(), canon);
}

/// The exact shapes the spec 0.8 corpus pins in
/// `valid/numbers/prefix_case/*.canonical.ktav`.
#[test]
fn canonical_shapes_match_the_prefix_case_oracles() {
    // Inline Array: § 5.9.4 expands it to the multiline shape; lowercase
    // items become decimal Integers, uppercase items stay bare Strings.
    let v = ktav::parse("mixed: [0X1A, 0x1A, 0B1, 0b1]\n").unwrap();
    assert_eq!(
        ktav::render::emit_canonical(&v).unwrap(),
        "mixed: [\n    0X1A\n    26\n    0B1\n    1\n]\n"
    );
    // Top-level multiline Array: § 5.9.3 / § 5.9.6 bare item lines.
    let v = ktav::parse("[\n0X1A\n0x1A\n0B1\n0b1\n]\n").unwrap();
    assert_eq!(
        ktav::render::emit_canonical(&v).unwrap(),
        "0X1A\n26\n0B1\n1\n"
    );
}

/// `format_str` of a comment-free, blank-free document equals
/// `emit_canonical` of its parse (see `format_str`'s docs in lib.rs) — and
/// both equal the corpus oracle bytes, so the agreement cannot hold by
/// both writers being wrong the same way.
#[test]
fn format_str_agrees_with_emit_canonical_on_comment_free_inputs() {
    let cases = [
        ("a: 0X1A\n", "a: 0X1A\n"),
        (
            "mixed: [0X1A, 0x1A, 0B1, 0b1]\n",
            "mixed: [\n    0X1A\n    26\n    0B1\n    1\n]\n",
        ),
        ("[\n0X1A\n0x1A\n0B1\n0b1\n]\n", "0X1A\n26\n0B1\n1\n"),
    ];
    for (src, want) in cases {
        let fmt = ktav::format_str(src).unwrap();
        assert_eq!(fmt, want, "format_str for {src:?}");
        let canon = ktav::render::emit_canonical(&ktav::parse(src).unwrap()).unwrap();
        assert_eq!(canon, want, "emit_canonical for {src:?}");
    }
}
