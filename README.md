# Ktav (כְּתָב)

> A plain configuration format. JSON5-shaped, but without the quotes,
> without the commas, with dotted keys for nesting. Native `serde`
> integration.

**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)

**Specification:** this crate implements **Ktav 0.1**. The format is
versioned and maintained independently of this crate — see
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) for the formal
document.

---

## Name

*Ktav* (Hebrew: **כְּתָב**) means "writing, that which is written" — a
thing recorded in a form fixed enough that its meaning does not depend
on who passes it along. The name fits literally: a config file *is*
ktav on disk, and the library reads it and hands you back a live
structure without making anything up along the way.

## Motto

> **Be the config's friend, not its examiner. The config isn't perfect —
> but it's the best one.**

Every rule is local. Every line either stands on its own or depends only
on visible brackets. No indentation pitfalls, no forgotten quotes, no
trailing-comma arithmetic.

## The rules

A Ktav document is an implicit top-level object. Inside any object you
have pairs; inside any array you have items.

```text
# comment              — any line starting with '#'
key: value             — scalar pair; key may be a dotted path (a.b.c)
key:: value            — scalar pair; value is ALWAYS a literal string
key: { ... }           — multi-line object; `}` closes on its own line
key: [ ... ]           — multi-line array; `]` closes on its own line
key: {}   /   key: []  — empty compound, inline
key: ( ... )           — multi-line string; common indent stripped
key: (( ... ))         — multi-line string; verbatim (no stripping)
:: value               — inside an array: literal-string item
```

That's the whole language. No commas, no quotes, no escape inside the
value itself — the only "escape" is the `::` marker, and it lives in the
separator (for pairs) or as a line prefix (for array items).

## Values and special tokens

### Strings

Default for any scalar. Stored internally as `Value::String`. The value
is whatever follows `:` after trimming.

```text
name: Russia
path: /etc/hosts
greeting: hello world
# `::` forces a literal string
pattern:: [a-z]+
```

### Numbers

Numbers are written bare (no quotes). At the `Value` level they are
strings; serde parses them into the target Rust type (`u16`, `i64`,
`f64`, …) using `FromStr` on deserialization, and formats them with
`Display` on serialization.

```text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
```

A value like `port: abc` parses fine *at the Ktav level* (string
`"abc"`), but `serde::deserialize` into `u16` will return a clear
`ParseError`.

### Booleans: `true` / `false`

Strict lowercase. Anything else is a string.

```text
# Value::Bool(true)
on: true
# Value::Bool(false)
off: false
# Value::String("True")
capitalized: True
# Value::String("FALSE")
yelling:    FALSE
# Value::String("true")
literal:: true
```

### Null: `null`

Strict lowercase. Matches `Option::None` on the Rust side, as well as
`()` for unit.

```text
# Value::Null
label: null
# Value::String("Null")
capitalized: Null
# Value::String("null")
literal:: null
```

When serializing, `Option::None` is emitted as `null`. Suppress with
`#[serde(skip_serializing_if = "Option::is_none")]` if you prefer the
field absent.

### Empty object / empty array

The **only** inline compound values allowed — nothing to separate, no
commas needed.

```text
# empty object
meta: {}
# empty array
tags: []
```

### Keyword-like strings need `::`

If a string's content happens to equal a keyword (`true`, `false`,
`null`) or begin with `{` or `[`, the **serializer emits `::`
automatically** so the round-trip is lossless. On the writing side you
do the same:

```text
# the string "true", not a bool
flag:: true
# the string "null", not a null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
```

## Compound values are multi-line

Non-empty `{ ... }` / `[ ... ]` **must** span multiple lines, with the
closing bracket on its own line. `x: { a: 1 }` and `x: [1, 2, 3]` are
rejected with a clear error — Ktav has no comma-separation rules and
no escape mechanism for them.

```text
# rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]

# accepted — multi-line form
server: {
    host: 127.0.0.1
    port: 8080
}

tags: [
    primary
    eu
    prod
]
```

## Using it from Rust

Ktav is serde-native. Any type implementing `Serialize` / `Deserialize`
(including `#[derive]`-generated ones) round-trips through Ktav out of
the box.

### Parse — decode straight into a typed struct

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Db { host: String, timeout: u32 }

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port:    u16,
    ratio:   f64,
    tls:     bool,
    tags:    Vec<String>,
    db:      Db,
}

const SRC: &str = "\
service: web
port:i 8080
ratio:f 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout:i 30
";

let cfg: Config = ktav::from_str(SRC)?;
println!("port={} db.host={}", cfg.port, cfg.db.host);
```

### Walk — match on the dynamic `Value` enum

```rust
use ktav::value::Value;

let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("top is always an object") };

for (k, v) in top {
    let kind = match v {
        Value::Null         => "null".into(),
        Value::Bool(b)      => format!("bool={b}"),
        Value::Integer(s)   => format!("int={s}"),
        Value::Float(s)     => format!("float={s}"),
        Value::String(s)    => format!("str={s:?}"),
        Value::Array(a)     => format!("array({})", a.len()),
        Value::Object(o)    => format!("object({})", o.len()),
    };
    println!("{k} -> {kind}");
}
```

### Build & render — construct a document in code

```rust
use ktav::value::{ObjectMap, Value};

let mut top = ObjectMap::default();
top.insert("name".into(),  Value::String("frontend".into()));
top.insert("port".into(),  Value::Integer("8443".into()));
top.insert("tls".into(),   Value::Bool(true));
top.insert("ratio".into(), Value::Float("0.95".into()));
top.insert("notes".into(), Value::Null);

let text = ktav::render::render(&Value::Object(top))?;
```

For typical app use prefer the serde path — `ktav::to_string(&cfg)` —
and reach for `Value` only when the schema is dynamic.

Four public entry points: [`from_str`](https://docs.rs/ktav) /
[`from_file`](https://docs.rs/ktav) for reading, [`to_string`](https://docs.rs/ktav) /
[`to_file`](https://docs.rs/ktav) for writing. A complete runnable
example lives in [`examples/basic.rs`](examples/basic.rs).

### Typed markers

Rust numeric types (`u8`..`u128`, `i8`..`i128`, `usize`, `isize`, `f32`,
`f64`) serialize to Ktav with explicit typed markers: `port:i 8080`,
`ratio:f 0.5`. Deserialization accepts *both* typed-marker and plain-string
forms — documents written without markers still work, exactly as before.
`NaN` / `±Infinity` are rejected by the serializer (Ktav 0.1.0 does not
represent them).

## Examples: Ktav → JSON5

JSON5 is on the right because it reads like ordinary JavaScript, allows
comments, and shows exactly what the parser produces.

### 1. Scalars

```text
name: Russia
port: 20082
```
```json5
{
  name: "Russia",
  port: "20082"
}
```

All scalars come out as strings at the `Value` level; numeric / boolean
types are parsed through serde when you deserialize into `u16` / `bool`
/ `f64` / …

### 2. Dotted keys = nested objects

```text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
```
```json5
{
  server: { host: "127.0.0.1", port: "8080" },
  app: { debug: "true" }
}
```

Any depth works. The full address is on every line.

### 3. Nested object as a value

```text
server: {
    host: 127.0.0.1
    port: 8080
    endpoints.api: /v1
    endpoints.admin: /admin
}
```
```json5
{
  server: {
    host: "127.0.0.1",
    port: "8080",
    endpoints: { api: "/v1", admin: "/admin" }
  }
}
```

### 4. Array of scalars

```text
banned_patterns: [
    .*\.onion:\d+
    .*:25
]
```
```json5
{
  banned_patterns: [".*\\.onion:\\d+", ".*:25"]
}
```

### 5. Array of objects

```text
upstreams: [
    {
        host: a.example
        port: 1080
    }
    {
        host: b.example
        port: 1080
    }
]
```
```json5
{
  upstreams: [
    { host: "a.example", port: "1080" },
    { host: "b.example", port: "1080" }
  ]
}
```

### 6. Arbitrary nesting

Every compound value spans multiple lines (single-line `{ ... }` / `[ ... ]`
with contents is not accepted — only the empty forms `{}` / `[]` are
inline). Nest as deep as needed:

```text
countries: [
    {
        name: Russia
        cities: [
            {
                name: Moscow
                buildings: [
                    {
                        name: Kremlin
                    }
                    {
                        name: Saint Basil's
                    }
                ]
            }
            {
                name: Saint Petersburg
            }
        ]
    }
    {
        name: France
    }
]
```

### 7. Literal strings: `::`

Some values would otherwise be parsed as compound (because they start
with `{` or `[`): regular expressions, IPv6 addresses, template
placeholders. The double-colon `::` flags them as "raw string, do not
parse further."

```text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl

hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
```
```json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
```

For pairs the marker sits between key and value; for array items it
stands at the start of the line. **Serialization emits `::`
automatically** when a string value begins with `{` or `[`, so
round-tripping regexes and IPv6 addresses just works.

### 8. Comments

```text
# top-level comment
port: 8080

items: [
    # this comment does not break the array
    a
    b
]
```

Comments are full lines starting with `#`. Inline comments are not
supported — they get confused with the value too easily.

### 9. Multi-line strings: `( ... )` and `(( ... ))`

Values that span multiple lines go inside parentheses. The opening and
closing lines are NOT part of the value.

`(` ... `)` — common leading whitespace is stripped, so you can indent
the block to match its surroundings without contaminating the content:

```text
body: (
    {
      "qwe": 1
    }
)
```
```json5
{ body: "{\n  \"qwe\": 1\n}" }
```

`((` ... `))` — verbatim: every character between the markers ends up in
the value, including leading whitespace:

```text
sig: ((
  -----BEGIN-----
  QUJDRA==
  -----END-----
))
```
```json5
{ sig: "  -----BEGIN-----\n  QUJDRA==\n  -----END-----" }
```

Inside a block, `{` / `[` / `#` are just content — **no compound parsing,
no comment skipping**. The only special sequence is the terminator on
its own line.

Empty inline form: `key: ()` or `key: (())` — both yield the empty
string (same as `key:`).

Serialization: any string containing `\n` is emitted with `(( ... ))`
so the round-trip is byte-for-byte lossless. Strings without newlines
use the usual single-line form.

Limitation: a line whose trimmed content is exactly `)` / `))` always
closes the block, so such a literal cannot appear as content without
using an external file.

### 10. Empty compounds

```text
meta: {}
tags: []
```

Inline empty is allowed. Anything with contents must span multiple
lines, and the closing `}` / `]` must sit on its own line.

### 11. Enums

Ktav uses serde's default *externally tagged* enum representation.

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode { Fast, Slow }

#[derive(Serialize, Deserialize)]
enum Action {
    Log(String),
    Count(u32),
}
```

```text
# unit variant — just the name
mode: fast

# newtype variant — single-entry object
action: {
    Log: hello
}
```

## Round-trip

```rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
```

Serialization preserves:
- **Field order** — `Value::Object` is backed by an `IndexMap`, so the
  order is whatever serde emits (for structs: declaration order).
- **Literal strings** — values starting with `{` or `[` are emitted
  with the `::` marker.
- **`None` fields** — skipped on output; reappear as `None` on input
  (via serde's `Option` handling).

## Architecture

```
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── render/           — pretty-printer (Value → text)
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error + serde::Error impls
└── lib.rs            — glue: from_str / from_file / to_string / to_file
```

Each file holds one exported item; implementation details are private to
their parent module.

## What Ktav does NOT do — and never will

- **Inline non-empty compounds** like `x: { a: 1, b: 2 }`. They'd bring
  commas, and commas would bring escaping. Compound values are
  multiline.
- **Anchors / aliases / merge keys** (`&anchor`, `*ref`, `<<:`). Any
  line whose meaning depends on a declaration far away stops being
  self-sufficient. If you want DRY, compose defaults in code.
- **File includes** (`@include`, `!import`). Write a wrapper in code
  for large configs.
- **Top-level arrays.** The document is always an object.

## Installation

Once published:

```toml
[dependencies]
ktav = "0.1"
serde = { version = "1", features = ["derive"] }
```

## Support the project

The author has many ideas that could be broadly useful to IT worldwide —
not limited to Ktav. Realizing them requires funding. If you'd like to
help, please reach out at **phpcraftdream@gmail.com**.

## License

MIT. See [LICENSE](LICENSE).
