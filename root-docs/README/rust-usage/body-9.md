>>>>> lang=en
let src = "port: 8080\nhost: example.com\n";
let mut keys = Vec::new();
parse_events(src, |ev| {
    if let ParseEvent::Key(k) = ev {
        keys.push(k.to_string());
    }
})?;
assert_eq!(keys, ["port", "host"]);
```

The root is `BeginObject`/`EndObject` or `BeginArray`/`EndArray`
depending on the document's first content line (an Object here, since
`port: 8080` is a pair); nested compounds bracket their contents the
same way. `ParseEvent` is `#[non_exhaustive]`. A complete runnable
example with depth tracking
and a pretty-printer:
[`examples/events.rs`](examples/events.rs) — `cargo run --example events`.

### Numbers

Rust numeric types (`u8`..`u128`, `i8`..`i128`, `usize`, `isize`, `f32`,
`f64`) serialize to Ktav as bare numbers: `port: 8080`, `ratio: 0.5`.
Coming back, a bare integer/decimal body deserializes straight into
the target numeric type; a value that arrived as a string (e.g. forced
with `::`) is still accepted via `FromStr`. `NaN` / `±Infinity` are
rejected by the serializer (Ktav does not represent them).

>>>>> lang=ru
let src = "port: 8080\nhost: example.com\n";
let mut keys = Vec::new();
parse_events(src, |ev| {
    if let ParseEvent::Key(k) = ev {
        keys.push(k.to_string());
    }
})?;
assert_eq!(keys, ["port", "host"]);
```

Корень — `BeginObject`/`EndObject` или `BeginArray`/`EndArray`, в
зависимости от первой содержательной строки документа (здесь Object,
так как `port: 8080` — это пара); вложенные компаунды обрамляют своё
содержимое так же. `ParseEvent` помечен
`#[non_exhaustive]`. Полный запускаемый пример с tracking-ом глубины
и pretty-print-ом:
[`examples/events.rs`](examples/events.rs) — `cargo run --example events`.

### Числа

Числовые Rust-типы (`u8`..`u128`, `i8`..`i128`, `usize`, `isize`, `f32`,
`f64`) сериализуются в Ktav как голые числа: `port: 8080`,
`ratio: 0.5`. На обратном пути голое целое/десятичное тело
десериализуется прямо в нужный числовой тип; значение, пришедшее
строкой (например, форсированное через `::`), по-прежнему принимается
через `FromStr`. `NaN` / `±Infinity` отвергаются сериализатором
(Ktav их не представляет).

>>>>> lang=zh
let src = "port: 8080\nhost: example.com\n";
let mut keys = Vec::new();
parse_events(src, |ev| {
    if let ParseEvent::Key(k) = ev {
        keys.push(k.to_string());
    }
})?;
assert_eq!(keys, ["port", "host"]);
```

根据文档第一条内容行的形状,根节点是 `BeginObject`/`EndObject` 或
`BeginArray`/`EndArray`(此处因为 `port: 8080` 是一个 pair,所以是
Object);嵌套复合值以同样方式括起自身内容。`ParseEvent` 标注
`#[non_exhaustive]`。完整的、带深度跟踪
和漂亮打印的可运行示例:
[`examples/events.rs`](examples/events.rs) —— `cargo run --example events`。

### 数字

Rust 数值类型(`u8`..`u128`、`i8`..`i128`、`usize`、`isize`、`f32`、
`f64`)会以裸数字序列化到 Ktav:`port: 8080`、`ratio: 0.5`。回程时,
裸整数/小数 body 直接反序列化为目标数值类型;以字符串形式到达的值
(例如用 `::` 强制的)仍通过 `FromStr` 接受。`NaN` / `±Infinity` 会被
序列化器拒绝(Ktav 不表示这些值)。

