//! Ограниченный CI-прогон mutations; оракулы проверяются вне таймера.

use std::fmt::Write;
use std::hint::black_box;
use std::time::{Duration, Instant};

use ktav::edit::{
    insert_value, insert_value_segments, remove_value, remove_value_segments, rename_key,
    rename_key_segments, EditError,
};
use ktav::{ObjectMap, Value};

const SAMPLE: Duration = Duration::from_millis(60);
const WARMUP: Duration = Duration::from_millis(5);

#[derive(Clone, Copy)]
enum Operation {
    Insert,
    Remove,
    Rename,
}

struct Case {
    name: String,
    operation: Operation,
    source: String,
    path: String,
    segments: Vec<String>,
    expected: String,
    semantic: Value,
}

impl Case {
    fn run(&self, segments: bool) -> Result<String, EditError> {
        let source = black_box(self.source.as_str());
        let value = Value::Integer("1".into());
        match (self.operation, segments) {
            (Operation::Insert, false) => insert_value(source, black_box(&self.path), &value),
            (Operation::Insert, true) => {
                insert_value_segments(source, black_box(&self.segments), &value)
            }
            (Operation::Remove, false) => remove_value(source, black_box(&self.path)),
            (Operation::Remove, true) => remove_value_segments(source, black_box(&self.segments)),
            (Operation::Rename, false) => rename_key(source, black_box(&self.path), "renamed"),
            (Operation::Rename, true) => {
                rename_key_segments(source, black_box(&self.segments), "renamed")
            }
        }
    }

    fn validate(&self, segments: bool) -> usize {
        let output = self.run(segments).expect("benchmark mutation must succeed");
        assert_eq!(output, self.expected, "{}: source bytes", self.name);
        assert_eq!(
            ktav::parse(&output).expect("benchmark result must parse"),
            self.semantic,
            "{}: semantic tree",
            self.name,
        );
        output.len()
    }
}

fn object(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Object(entries.into_iter().map(|(k, v)| (k.into(), v)).collect())
}

fn integer() -> Value {
    Value::Integer("1".into())
}

fn flat_cases(rows: usize) -> Vec<Case> {
    let mut source = String::new();
    let mut original = ObjectMap::default();
    for i in 0..rows {
        let key = format!("key{i}");
        writeln!(source, "{key}: old").expect("String writes cannot fail");
        original.insert(key.into(), Value::String("old".into()));
    }
    let target = format!("key{}", rows / 2);
    let line = format!("{target}: old\n");
    let mut inserted = original.clone();
    inserted.insert("added".into(), integer());
    let mut removed = original.clone();
    removed.shift_remove(target.as_str());
    let renamed: ObjectMap = original
        .into_iter()
        .map(|(key, value)| {
            if key == target.as_str() {
                ("renamed".into(), value)
            } else {
                (key, value)
            }
        })
        .collect();
    vec![
        Case {
            name: format!("insert_flat_{rows}"),
            operation: Operation::Insert,
            expected: format!("{source}added: 1\n"),
            source: source.clone(),
            path: "added".into(),
            segments: vec!["added".into()],
            semantic: Value::Object(inserted),
        },
        Case {
            name: format!("remove_flat_{rows}"),
            operation: Operation::Remove,
            expected: source.replacen(&line, "", 1),
            source: source.clone(),
            path: target.clone(),
            segments: vec![target.clone()],
            semantic: Value::Object(removed),
        },
        Case {
            name: format!("rename_flat_{rows}"),
            operation: Operation::Rename,
            expected: source.replacen(&line, "renamed: old\n", 1),
            source,
            path: target.clone(),
            segments: vec![target],
            semantic: Value::Object(renamed),
        },
    ]
}

fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for rows in [4, 4096, 32768] {
        cases.extend(flat_cases(rows));
    }
    let payload = "x".repeat(128 * 1024);
    cases.push(Case {
        name: "rename_large_sibling".into(),
        operation: Operation::Rename,
        source: format!("payload: {payload}\ntarget: 1\n"),
        path: "target".into(),
        segments: vec!["target".into()],
        expected: format!("payload: {payload}\nrenamed: 1\n"),
        semantic: object([
            ("payload", Value::String(payload.into())),
            ("renamed", integer()),
        ]),
    });
    cases.extend([
        Case {
            name: "insert_inline_unicode".into(),
            operation: Operation::Insert,
            source: "\u{feff}  {ключ: old,}\r\n".into(),
            path: "added".into(),
            segments: vec!["added".into()],
            expected: "\u{feff}  {ключ: old, added: 1,}\r\n".into(),
            semantic: object([("ключ", Value::String("old".into())), ("added", integer())]),
        },
        Case {
            name: "insert_array_item".into(),
            operation: Operation::Insert,
            source: "items: [{x: 1}]\n".into(),
            path: "items.0.added".into(),
            segments: vec!["items".into(), "0".into(), "added".into()],
            expected: "items: [{x: 1, added: 1}]\n".into(),
            semantic: object([(
                "items",
                Value::Array(vec![object([("x", integer()), ("added", integer())])]),
            )]),
        },
        Case {
            name: "remove_dotted_materialize".into(),
            operation: Operation::Remove,
            source: "a.b.c: 1\r\nkeep: 1\r\n".into(),
            path: "a.b.c".into(),
            segments: vec!["a".into(), "b".into(), "c".into()],
            expected: "a.b: {}\r\nkeep: 1\r\n".into(),
            semantic: object([("a", object([("b", object([]))])), ("keep", integer())]),
        },
        Case {
            name: "remove_array_element".into(),
            operation: Operation::Remove,
            source: "items: [1, 2, 3]\n".into(),
            path: "items.1".into(),
            segments: vec!["items".into(), "1".into()],
            expected: "items: [1, 3]\n".into(),
            semantic: object([(
                "items",
                Value::Array(vec![integer(), Value::Integer("3".into())]),
            )]),
        },
        Case {
            name: "rename_escaped_key".into(),
            operation: Operation::Rename,
            source: "\"a.b\": 1\nkeep: 1\n".into(),
            path: "a\\.b".into(),
            segments: vec!["a.b".into()],
            expected: "renamed: 1\nkeep: 1\n".into(),
            semantic: object([("renamed", integer()), ("keep", integer())]),
        },
        Case {
            name: "rename_noop".into(),
            operation: Operation::Rename,
            source: "renamed:: 0007\r\n".into(),
            path: "renamed".into(),
            segments: vec!["renamed".into()],
            expected: "renamed:: 0007\r\n".into(),
            semantic: object([("renamed", Value::String("0007".into()))]),
        },
    ]);
    cases
}

fn main() {
    let label = std::env::args()
        .nth(1)
        .expect("baseline or candidate label");
    assert!(matches!(label.as_str(), "baseline" | "candidate"));
    println!("label,case,path,iterations,elapsed_ns,ns_per_op,output_len");
    for case in cases() {
        for segments in [false, true] {
            let len = case.validate(segments);
            let warmup = Instant::now();
            while warmup.elapsed() < WARMUP {
                drop(black_box(
                    case.run(segments).expect("validated benchmark case"),
                ));
            }
            let start = Instant::now();
            let mut iterations = 0u64;
            loop {
                drop(black_box(
                    case.run(segments).expect("validated benchmark case"),
                ));
                iterations += 1;
                if start.elapsed() >= SAMPLE {
                    break;
                }
            }
            let elapsed = start.elapsed().as_nanos();
            let path = if segments { "segments" } else { "text" };
            println!(
                "{label},{},{path},{iterations},{elapsed},{:.1},{len}",
                case.name,
                elapsed as f64 / iterations as f64,
            );
        }
    }
}
