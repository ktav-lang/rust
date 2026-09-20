//! Root-position Array serialization (spec § 5.0.1) — direct serde path.

use ktav::render::emit_canonical;
use ktav::ser::to_value;
use ktav::{from_str, to_string};
use serde::Serialize;

fn assert_canonical_identity<T: Serialize>(value: &T, text: &str) {
    let v = to_value(value).unwrap();
    let canonical = emit_canonical(&v).unwrap();
    assert_eq!(text, canonical);
}

#[test]
fn root_seq_of_integers_round_trips() {
    let v = vec![1_i64, 2, 3];
    let text = to_string(&v).unwrap();
    assert_eq!(text, "1\n2\n3\n");
    assert_eq!(from_str::<Vec<i64>>(&text).unwrap(), v);
    assert_canonical_identity(&v, &text);
}

#[test]
fn root_seq_empty() {
    let v: Vec<i64> = Vec::new();
    let text = to_string(&v).unwrap();
    assert_eq!(text, "[]\n");
    assert_eq!(from_str::<Vec<i64>>(&text).unwrap(), v);
    assert_canonical_identity(&v, &text);
}

#[test]
fn root_tuple_round_trips() {
    let t = (1_i64, "two".to_string(), 3.0_f64);
    let text = to_string(&t).unwrap();
    assert_eq!(text, "1\ntwo\n3.0\n");
    assert_eq!(from_str::<(i64, String, f64)>(&text).unwrap(), t);
    assert_canonical_identity(&t, &text);
}

#[test]
fn root_tuple_struct_round_trips() {
    #[derive(Serialize)]
    struct Pt(i64, i64);
    let text = to_string(&Pt(7, 8)).unwrap();
    assert_eq!(text, "7\n8\n");
    assert_eq!(from_str::<(i64, i64)>(&text).unwrap(), (7, 8));
    assert_canonical_identity(&Pt(7, 8), &text);
}

#[test]
fn root_seq_compound_first_item_wraps() {
    // § 5.9.3 lone-`[` wrap — first item is a compound, so the whole root
    // Array wraps and EVERY item moves to indent 1.
    let v = vec![vec![1_i64, 2], vec![3, 4]];
    let text = to_string(&v).unwrap();
    assert_eq!(
        text,
        "[\n    [\n        1\n        2\n    ]\n    [\n        3\n        4\n    ]\n]\n"
    );
    assert_eq!(from_str::<Vec<Vec<i64>>>(&text).unwrap(), v);
    assert_canonical_identity(&v, &text);
}

#[test]
fn root_seq_first_item_empty_compound_wraps() {
    // An empty compound first item still triggers the wrap (bare `[]` at
    // offset 0 would be read as the whole root, § 5.9.3 rule 2 — see
    // render/helpers.rs first_item_needs_wrap).
    let v = vec![Vec::<i64>::new()];
    let text = to_string(&v).unwrap();
    assert_eq!(text, "[\n    []\n]\n");
    assert_eq!(from_str::<Vec<Vec<i64>>>(&text).unwrap(), v);
    assert_canonical_identity(&v, &text);
}

#[test]
fn root_seq_first_item_pair_candidate_takes_raw_marker() {
    // Fixture oracle quoted_keys/array_item_raw_marker_needed; mirrors the
    // owned-writer test render_lossless.rs::
    // render_array_root_first_item_pair_candidate_takes_raw_marker.
    // Without the ported § 5.9.6/§ 5.9.12 first-item safeguard the bare
    // form `"tis the season": fa` would be root-detected as an Object's
    // first pair and the document would not round-trip as Vec<String>.
    let v = vec!["\"tis the season\": fa".to_string(), "plain".to_string()];
    let text = to_string(&v).unwrap();
    assert_eq!(text, ":: \"tis the season\": fa\nplain\n");
    assert_eq!(from_str::<Vec<String>>(&text).unwrap(), v);
}

#[test]
fn root_seq_first_item_bom_takes_raw_marker() {
    // § 3.1/§ 5.9.12 — bare form would place the BOM at byte offset 0
    // where readers strip it as metadata; the raw marker moves it off
    // offset 0.
    let v = vec!["\u{FEFF}x".to_string()];
    let text = to_string(&v).unwrap();
    assert_eq!(text, ":: \u{FEFF}x\n");
    assert_eq!(from_str::<Vec<String>>(&text).unwrap(), v);
}

#[test]
fn root_tuple_variant_matches_owned_path() {
    // Owned to_value maps a root tuple variant to
    // Value::Object({variant: items}) — single-pair root Object
    // (§ 8.2 byte-identity); variant name takes the § 5.9.10 rule (c)
    // root-first-key guard.
    // (No enum round-trip assertions — externally-tagged enum
    // deserialization from that document is a reader-side concern not
    // exercised here.)
    #[derive(Serialize)]
    enum E {
        V(i64, i64),
    }
    let text = to_string(&E::V(1, 2)).unwrap();
    assert_eq!(text, "V: [\n    1\n    2\n]\n");
    assert_canonical_identity(&E::V(1, 2), &text);
}

#[test]
fn root_tuple_variant_empty_matches_owned_path() {
    #[derive(Serialize)]
    enum F {
        U(),
    }
    let text = to_string(&F::U()).unwrap();
    assert_eq!(text, "U: []\n");
    assert_canonical_identity(&F::U(), &text);
}

#[test]
fn root_seq_without_length_hint_round_trips() {
    // A `None` length hint defers the wrap decision and must still
    // produce the canonical unwrapped form.
    struct NoLen<'a>(&'a [i64]);
    impl Serialize for NoLen<'_> {
        fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            use serde::ser::SerializeSeq;
            let mut seq = s.serialize_seq(None)?;
            for x in self.0 {
                seq.serialize_element(x)?;
            }
            seq.end()
        }
    }
    let text = to_string(&NoLen(&[1, 2, 3])).unwrap();
    assert_eq!(text, "1\n2\n3\n");
}
