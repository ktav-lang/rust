// Deterministic Ktav fixture generator for crate-level Criterion benches.
//
// Mirrors the synthesizer shape used by `editor/lsp/benches/fixtures.rs`
// (round-robin mix of pairs, dotted keys, plain numeric scalars (the
// `:i`/`:f` markers were removed in spec 0.7), raw `::` literals, nested
// objects/arrays, multi-line raw blocks, and `##` comments) so the parsing baseline reflects realistic config workloads
// rather than a single artificial pattern. No randomness — the index `i`
// drives every choice and every payload, so successive runs and
// successive machines see byte-identical input.
//
// Each bench file `include!`s this module rather than promoting it to a
// shared crate (Criterion benches can't share a `mod common` cleanly).

// Not every bench binary references every helper here, so silence the
// per-binary unused-warnings rather than scatter `#[allow]`s.
#![allow(dead_code)]

use std::fmt::Write as _;

/// 1 KiB target.
pub fn small_1k() -> String {
    synth(1_024)
}

/// 50 KiB target.
pub fn medium_50k() -> String {
    synth(50 * 1_024)
}

/// 500 KiB target.
pub fn large_500k() -> String {
    synth(500 * 1_024)
}

/// Synthesize a Ktav document at least `target_bytes` long.
pub fn synth(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 256);
    out.push_str("## generated benchmark fixture\n");
    let mut i = 0u32;
    while out.len() < target_bytes {
        match i % 12 {
            0 => {
                let _ = writeln!(out, "name_{}: value_{}", i, i);
            }
            1 => {
                let _ = writeln!(out, "port_{}: {}", i, 8000 + (i as u64 % 1000));
            }
            2 => {
                let _ = writeln!(out, "ratio_{}: {}.{}", i, i % 100, i % 1000);
            }
            3 => {
                let _ = writeln!(
                    out,
                    "flag_{}: {}",
                    i,
                    if i % 2 == 0 { "true" } else { "false" }
                );
            }
            4 => {
                let _ = writeln!(
                    out,
                    "label_{}:: literal text with spaces and symbols !@#${}",
                    i, i
                );
            }
            5 => {
                let _ = writeln!(out, "service.{}.host: 10.0.0.{}", i, i % 256);
            }
            6 => {
                let _ = writeln!(out, "service.{}.port: {}", i, 30000 + (i as u64 % 5000));
            }
            7 => {
                let _ = writeln!(out, "## section {}", i / 12);
            }
            8 => {
                let _ = writeln!(out, "obj_{}: {{", i);
                let _ = writeln!(out, "    inner_a: {}", i);
                let _ = writeln!(out, "    inner_b: {}.5", i % 100);
                let _ = writeln!(out, "    inner_c:: raw body for {}", i);
                let _ = writeln!(out, "}}");
            }
            9 => {
                let _ = writeln!(out, "list_{}: [", i);
                let _ = writeln!(out, "    item-a-{}", i);
                let _ = writeln!(out, "    item-b-{}", i);
                let _ = writeln!(out, "    :: literal-{}", i);
                let _ = writeln!(out, "]");
            }
            10 => {
                let _ = writeln!(out, "doc_{}: (", i);
                let _ = writeln!(out, "first line of body {}", i);
                let _ = writeln!(out, "second line of body {}", i);
                let _ = writeln!(out, ")");
            }
            _ => {
                let _ = writeln!(out, "tag_{}: alpha-beta-gamma-{}", i, i);
            }
        }
        i = i.wrapping_add(1);
    }
    out
}

/// The line the generator emits for round-robin branch 0. It can only
/// ever appear at top level (nested content uses other shapes), so it
/// is a safe anchor: the line AFTER it is guaranteed to be parsed in
/// root-Object context.
fn is_name_pair(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("name_") else {
        return false;
    };
    let Some((index, value)) = rest.split_once(": ") else {
        return false;
    };
    !index.is_empty()
        && index.bytes().all(|b| b.is_ascii_digit())
        && value
            .strip_prefix("value_")
            .is_some_and(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
}

/// Produce a copy of `text` with one invalid line injected right after
/// the `name_N: value_N` anchor line nearest the byte midpoint, and
/// return it together with the 1-based line number the injected line
/// will occupy. The bad line `"key:value"` (no space after the colon)
/// is rejected with `ErrorKind::MissingSeparatorSpace`, so the error
/// benches exercise error construction on a realistic-shaped input —
/// and the caller can assert the exact category and location instead
/// of merely `is_err()`. Panics if `text` contains no anchor line
/// (i.e. it was not produced by [`synth`]).
pub fn inject_bad_line(text: &str) -> (String, u32) {
    let target = text.len() / 2;
    let mut offset = 0;
    let mut best: Option<(usize, usize)> = None; // (distance from midpoint, offset after the anchor line)
    for line in text.split_inclusive('\n') {
        let body = line.strip_suffix('\n').unwrap_or(line);
        if is_name_pair(body) {
            let after = offset + line.len();
            let distance = after.abs_diff(target);
            if best.map_or(true, |(bd, _)| distance < bd) {
                best = Some((distance, after));
            }
        }
        offset += line.len();
    }
    let split = best
        .expect("inject_bad_line requires synth() output with a `name_N: value_N` anchor")
        .1;
    let bad_line = text[..split].matches('\n').count() as u32 + 1;
    let mut out = String::with_capacity(text.len() + "key:value\n".len());
    out.push_str(&text[..split]);
    out.push_str("key:value\n");
    out.push_str(&text[split..]);
    (out, bad_line)
}

/// Parse `text` and verify it is EXACTLY the document [`synth`]
/// promises: an Object root whose every entry has the shape and value
/// kind the round-robin branch that emitted it implies. Panics naming
/// the offending key on any drift — this is what keeps the success
/// benches honest (a stale-syntax regression must fail loudly here,
/// not silently turn `parse_synth` into a crash or a String-only doc).
pub fn validate_synth(text: &str) {
    let value = ktav::parse(text).unwrap_or_else(|e| panic!("synth fixture must parse: {e:?}"));
    let ktav::Value::Object(root) = &value else {
        panic!("synth fixture root must be Object, got {value:?}");
    };
    let mut seen = Seen {
        name: false,
        port: false,
        ratio: false,
        flag: false,
        label: false,
        obj: false,
        list: false,
        doc: false,
        tag: false,
        service_host: false,
        service_port: false,
    };
    for (key, entry) in root {
        check_entry(key, entry, &mut seen);
    }
    let missing = [
        ("name_", seen.name),
        ("port_", seen.port),
        ("ratio_", seen.ratio),
        ("flag_", seen.flag),
        ("label_", seen.label),
        ("obj_", seen.obj),
        ("list_", seen.list),
        ("doc_", seen.doc),
        ("tag_", seen.tag),
        ("service host", seen.service_host),
        ("service port", seen.service_port),
    ]
    .iter()
    .filter(|(_, ok)| !ok)
    .map(|(name, _)| *name)
    .collect::<Vec<_>>()
    .join(", ");
    assert!(
        missing.is_empty(),
        "synth fixture missing expected key shapes: {missing}"
    );
}

/// Which round-robin branch shapes were observed in the root object.
struct Seen {
    name: bool,
    port: bool,
    ratio: bool,
    flag: bool,
    label: bool,
    obj: bool,
    list: bool,
    doc: bool,
    tag: bool,
    service_host: bool,
    service_port: bool,
}

fn index_suffix(key: &str, prefix: &str) -> u32 {
    key.strip_prefix(prefix)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| panic!("malformed index suffix in key `{key}`"))
}

fn check_entry(key: &str, entry: &ktav::Value, seen: &mut Seen) {
    macro_rules! as_kind {
        ($method:ident, $kind:literal) => {
            entry
                .$method()
                .unwrap_or_else(|| panic!("{key}: {entry:?} expected {}", $kind))
        };
    }
    if key == "service" {
        let obj = as_kind!(as_object, "Object");
        for (sub_key, sub_value) in obj.iter() {
            let n = index_suffix(sub_key, "");
            let sub_obj = sub_value
                .as_object()
                .unwrap_or_else(|| panic!("service.{sub_key}: {sub_value:?} expected Object"));
            assert_eq!(
                sub_obj.len(),
                1,
                "service.{sub_key}: expected exactly one entry, got {sub_obj:?}"
            );
            let (k, v) = sub_obj.iter().next().unwrap();
            match k.as_str() {
                "host" => {
                    assert_eq!(
                        v.as_str(),
                        Some(format!("10.0.0.{}", n % 256).as_str()),
                        "service.{sub_key}.host: {v:?}"
                    );
                    seen.service_host = true;
                }
                "port" => {
                    assert_eq!(
                        v.as_integer(),
                        Some((30000u64 + (n as u64 % 5000)).to_string()).as_deref(),
                        "service.{sub_key}.port: {v:?}"
                    );
                    seen.service_port = true;
                }
                other => panic!("service.{sub_key}: unexpected sub-entry `{other}`: {v:?}"),
            }
        }
        return;
    }
    match key {
        k if k.starts_with("name_") => {
            let i = index_suffix(k, "name_");
            assert_eq!(
                as_kind!(as_str, "String"),
                format!("value_{i}"),
                "{k}: {entry:?}"
            );
            seen.name = true;
        }
        k if k.starts_with("port_") => {
            let i = index_suffix(k, "port_");
            assert_eq!(
                as_kind!(as_integer, "Integer"),
                (8000u64 + (i as u64 % 1000)).to_string(),
                "{k}: {entry:?}"
            );
            seen.port = true;
        }
        k if k.starts_with("ratio_") => {
            let i = index_suffix(k, "ratio_");
            let expected = format!("{}.{}", i % 100, i % 1000);
            let actual = as_kind!(as_float, "Float");
            assert_eq!(
                actual.parse::<f64>().unwrap(),
                expected.parse::<f64>().unwrap(),
                "{k}: {entry:?} expected numeric value {expected}"
            );
            seen.ratio = true;
        }
        k if k.starts_with("flag_") => {
            let i = index_suffix(k, "flag_");
            assert_eq!(entry.as_bool(), Some(i % 2 == 0), "{k}: {entry:?}");
            seen.flag = true;
        }
        k if k.starts_with("label_") => {
            let i = index_suffix(k, "label_");
            assert_eq!(
                as_kind!(as_str, "String"),
                format!("literal text with spaces and symbols !@#${i}"),
                "{k}: {entry:?}"
            );
            seen.label = true;
        }
        k if k.starts_with("obj_") => {
            let i = index_suffix(k, "obj_");
            let obj = as_kind!(as_object, "Object");
            let keys: Vec<_> = obj.iter().map(|(k, _)| k).collect();
            assert_eq!(keys, ["inner_a", "inner_b", "inner_c"], "{k}: {obj:?}");
            let a = obj.get("inner_a").unwrap();
            assert_eq!(
                a.as_integer(),
                Some(i.to_string()).as_deref(),
                "{k}.inner_a: {a:?}"
            );
            let b = obj.get("inner_b").unwrap();
            assert_eq!(
                b.as_float().unwrap_or("").parse::<f64>().unwrap(),
                format!("{}.5", i % 100).parse::<f64>().unwrap(),
                "{k}.inner_b: {b:?}"
            );
            let c = obj.get("inner_c").unwrap();
            assert_eq!(
                c.as_str(),
                Some(format!("raw body for {i}").as_str()),
                "{k}.inner_c: {c:?}"
            );
            seen.obj = true;
        }
        k if k.starts_with("list_") => {
            let i = index_suffix(k, "list_");
            let arr = as_kind!(as_array, "Array");
            assert_eq!(arr.len(), 3, "{k}: {arr:?}");
            assert_eq!(
                arr[0].as_str(),
                Some(format!("item-a-{i}").as_str()),
                "{k}: {arr:?}"
            );
            assert_eq!(
                arr[1].as_str(),
                Some(format!("item-b-{i}").as_str()),
                "{k}: {arr:?}"
            );
            assert_eq!(
                arr[2].as_str(),
                Some(format!("literal-{i}").as_str()),
                "{k}: {arr:?}"
            );
            seen.list = true;
        }
        k if k.starts_with("doc_") => {
            let i = index_suffix(k, "doc_");
            assert_eq!(
                as_kind!(as_str, "String"),
                format!("first line of body {i}\nsecond line of body {i}"),
                "{k}: {entry:?}"
            );
            seen.doc = true;
        }
        k if k.starts_with("tag_") => {
            let i = index_suffix(k, "tag_");
            assert_eq!(
                as_kind!(as_str, "String"),
                format!("alpha-beta-gamma-{i}"),
                "{k}: {entry:?}"
            );
            seen.tag = true;
        }
        _ => panic!("unexpected root key shape {key}: {entry:?}"),
    }
}
