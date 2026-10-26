# Ktav (כְּתָב)

[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

> A plain configuration format. JSON5-shaped, but without the quotes,
> without the commas, with dotted keys for nesting. Native `serde`
> integration.

**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)

**Playground:** convert JSON / YAML / TOML / INI ⇄ Ktav in your browser at **[ktav-lang.github.io](https://ktav-lang.github.io/)**.

**Version in preparation: 0.9.0, not published.** This branch combines
lossless source editing by path and the new source-error location model.
`Cargo.toml` targets the draft spec corpus `versions/0.9/tests`; published
0.8.1 remains unchanged. See [`ktav-lang/spec`](https://github.com/ktav-lang/spec)
for the specification and [`CHANGELOG.md`](CHANGELOG.md) for the migration.

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
## comment              — any line starting with '##'
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
## `::` forces a literal string
pattern:: [a-z]+
```

### Numbers

Numbers are written bare (no quotes) and typed by lexical form: a bare
integer body parses to `Value::Integer`, a bare decimal to
`Value::Float`. Each stores a *normalized* payload, not the original
spelling: `Integer` holds the canonical base-10 form (no underscores,
no `+`), `Float` holds the shortest decimal form that
round-trips the exact `f64` bits — `+1_000` becomes `Integer("1000")`,
`1.0e+2` becomes `Float("100.0")`. A decimal whose digits open with a
redundant `0` is not a number at all: `zip: 01234` is
`String("01234")` (§ 5.2), so a zero-padded identifier survives
verbatim. `Value`-level `Integer` covers the
i64 range; a native Rust integer type wider than i64 (`u64`, `i128`,
`u128`) that doesn't fit is stored as `Value::String` instead when
going through `ser::to_value`, matching what parsing that same decimal
text back would produce. serde deserializes numbers into the target
Rust type (`u16`, `i64`, `i128`, `f64`, …) via direct parsing, and
formats them with the same canonicalization on serialization; a value
forced to a string with `::` is still accepted.

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
## Value::Bool(true)
on: true
## Value::Bool(false)
off: false
## Value::String("True")
capitalized: True
## Value::String("FALSE")
yelling:    FALSE
## Value::String("true")
literal:: true
```

### Null: `null`

Strict lowercase. Matches `Option::None` on the Rust side, as well as
`()` for unit.

```text
## Value::Null
label: null
## Value::String("Null")
capitalized: Null
## Value::String("null")
literal:: null
```

When serializing, `Option::None` is emitted as `null`. Suppress with
`#[serde(skip_serializing_if = "Option::is_none")]` if you prefer the
field absent.

### Empty object / empty array

The **only** inline compound values allowed — nothing to separate, no
commas needed.

```text
## empty object
meta: {}
## empty array
tags: []
```

### Keyword-like strings need `::`

If a string's content happens to equal a keyword (`true`, `false`,
`null`) or begin with `{` or `[`, the **serializer emits `::`
automatically** so the round-trip is lossless. On the writing side you
do the same:

```text
## the string "true", not a bool
flag:: true
## the string "null", not a null
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
## rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]

## accepted — multi-line form
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

## Corners worth knowing

Four rules the examples above never reach. Each one is settled by the
specification rather than by this implementation, so every conforming
parser behaves the same way.

### Quoted keys — when a key contains a dot or a space

A dot in a bare key means nesting: `db.host: primary` builds
`db` → `host`. Quoting the key turns off that reading, so the dot is
part of the name (spec § 5.3.3):

```ktav
db.host: primary
"db.host": literal
"a b": spaces are fine too
```

The first line nests. The second is a single key literally named
`db.host`. This is why the error envelope reports `path` as an array
of segments rather than a joined string — a joined string could not tell
those two apart.

### Escapes apply inside inline compounds and quoted keys

`\uXXXX` and the named escapes (spec § 3.7, § 3.7.1) are read where
a delimiter would otherwise be structural — inside `{ }` and `[ ]`,
and inside a quoted key. A bare block-level value has no delimiters to
escape, so a backslash there is ordinary text:

```ktav
inline: {greek: \u03b1, csv: a\,b}
"\u00e9": 1
literal: \u0041
```

`greek` is `α`, `csv` is the single string `a,b` — the escaped
comma is not a separator — and the quoted key is `é`. But `literal` is
the eight characters `\u0041`, unchanged. A recognised escape also
forces String classification: a value written `\u0031` is the string
`1`, not the integer.

### Exactly one leading byte-order mark is skipped

A U+FEFF at the very start of the document is skipped before any
other byte is examined (spec § 3.1). Anywhere else it is ordinary
content — including the start of a later line, where it becomes part of
that key's name. Editors that add a BOM on save therefore do not break
a document, and a stray one further in does not silently disappear.

### Invalid UTF-8 is its own error, not an I/O failure

[`from_file`](https://docs.rs/ktav) validates the file's bytes as
UTF-8 before parsing and reports [`Error::InvalidUtf8`] with the byte
offset of the first bad sequence (spec § 6.15). A missing file or a
permission problem stays [`Error::Io`] — the two are worth
distinguishing, because one means "fix the file" and the other means
"fix the path". The bytes are never repaired or replaced before the
parser sees them.

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
port: 8080
ratio: 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout: 30
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

### Inspect errors — structured variants for tooling

`Error::Structured(ErrorKind)` carries a typed category plus a
byte-offset `Span` for every parse failure, so editors and linters
can highlight the exact offending range instead of the whole line.

```rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\nport: 443\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("line {line}: duplicate key {key:?}");
        println!("offending bytes: {:?}", span.slice(src));   // -> Some("port")
        let (l, c) = span.line_col(src);                      // 1-based / 0-based byte col
        println!("highlight at {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
```

Variants: `MissingSeparatorSpace`, `InvalidTypedScalar`, `DuplicateKey`,
`KeyPathConflict`, `EmptyKey`, `InvalidKey`, `UnclosedCompound`,
`UnbalancedBracket`, `InlineNonEmptyCompound`, `MissingSeparator`,
`LossyScalar` (strict mode only, see below), `Other`. The enum is
`#[non_exhaustive]` — always include a `_ =>` arm. `Error::line()` /
`Error::span()` are convenience accessors when the variant doesn't
matter. The `Display` impl produces the same human-readable string the
legacy `Error::Syntax(_)` did, so existing string-based callers keep
working.

A complete runnable example walks all variants:
[`examples/errors.rs`](examples/errors.rs) — `cargo run --example errors`.

### One JSON envelope for every structured error

The accessors above are Rust-only. `ErrorEnvelope` is the wire
contract for everyone else: one JSON object, ten fields, always all
ten, in a fixed order — `error`, `reason`, `line`, `line_text`,
`span`, `path`, `body`, `canonical`, `spec_section`,
`message`.

```rust
use ktav::{parse, ErrorEnvelope};

let src = "a: 1.10\n";
if let Err(e) = ktav::parse_strict(src) {
    println!("{}", ErrorEnvelope::from_error(&e, src).to_json());
}
```

```json
{"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
 "span":{"start":0,"end":7},"path":null,"body":"1.10",
 "canonical":"1.1","spec_section":"§3.6/§5.2",
 "message":"Syntax error: Line 1: LossyScalar: '1.10' would be …"}
```

`message` is the last field and the only one that is never
`null`: it carries this error's `Display` rendering verbatim. A
binding shows it to the user as-is rather than assembling prose from
the structured fields, so the same document produces the same error
text in every language. The nine older fields keep the positions they
shipped with — `message` was appended, not inserted.

Absent information is an explicit `null`, never an omitted key, so a
consumer can read every field positionally without negotiating a
schema first.

`span` is `{"start":N,"end":M}` in **byte offsets into the UTF-8
source**, not UTF-16 code units — the same unit [`Span`](https://docs.rs/ktav)
itself uses. An LSP consumer either converts, or negotiates
`positionEncoding: "utf-8"`.

`path` is an **array of exact decoded key segments, never a joined
string**. A key literally named `a.b` is one segment and cannot be
confused with a two-segment path — there is no separator in the wire
contract to be ambiguous about.

Writer rejections use the same envelope: `reason` carries the § 5.9.0
reason code (`NonFiniteFloat`, `EmptyKeyName`, …) and the two
rejections are named apart — `UnrepresentableAt` when the writer can
say where the offending node is (it fills `path` too),
`Unrepresentable` when it cannot.

Rendering is `to_json()` (or `push_json(&mut String)` to append into
a buffer you own). It is valid JSON for any payload — every string is
escaped per RFC 8259 — and no serde dependency is involved.

### Strict mode — catch silently canonicalised numbers

Types are inferred from a scalar's lexical form, and inferred numbers
are canonicalised: `version: 1.10` parses as `Float(1.1)`. A decimal
with a redundant leading zero is the one exception — `zip: 01234` is the
String `"01234"` (§ 5.2), because dropping that zero would destroy an
identifier. The default `parse()` does this
silently, so writing the document back out rewrites it.

`parse_strict()` rejects such **lossy scalars** instead:

```rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "version: 1.10\n";

assert!(parse(src).is_ok());                 // Float(1.1) — trailing zero gone
assert!(parse("zip: 01234\n").is_ok());       // String("01234") — leading zero kept

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
    }
    other => panic!("expected LossyScalar, got {other:?}"),
}
```

Fix either by appending `::` to keep the value a String
(`zip:: 01234`) or by writing the canonical number. Any document
`parse_strict()` accepts yields exactly the same `Value` tree as
`parse()`, so strict mode is a validation gate, not a different
dialect. The serde path (`from_str`) has no strict variant yet.

### Stream parse — events without an intermediate tree

`parse_events` invokes a callback for each parse event, with strings
borrowed directly into the input buffer — no allocation per event, no
intermediate `Value` tree. Useful when you don't need the full document:
counting keys, streaming to another format, building a custom shape.

```rust
use ktav::{parse_events, ParseEvent};

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

## Error model migration (0.9)

**0.9 migration (breaking):** `Error::InvalidUtf8` now contains
`{ valid_up_to, line, span }`; `ErrorKind::UnclosedCompound` contains
`{ kind, line, span }`. Update downstream constructors with the new fields;
use `..` in patterns when only selected fields matter. `line()` now returns
the source line for both errors. UTF-8 spans cover original malformed bytes;
EOF diagnostics cover the innermost opener through EOF and report its line.
Coordinates preserve BOM and LF/CR/CRLF; `Span::line_col` and envelope
`line_text` recognize all three terminators. I/O and writer failures retain
absent source positions. The ten-field JSON schema below is unchanged.
The new location oracles live in the draft `versions/0.9/tests` spec corpus;
this working version has not been published.

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
  port: 20082
}
```

Scalars are typed at the `Value` level from their lexical form
(`Integer`/`Float`/`Bool`/`Null`/`String`); force a literal string with
the `::` raw marker (e.g. `port:: 20082`) when a numeric-looking body
must stay a string.

### 2. Dotted keys = nested objects

```text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
```
```json5
{
  server: { host: "127.0.0.1", port: 8080 },
  app: { debug: true }
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
    port: 8080,
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
    { host: "a.example", port: 1080 },
    { host: "b.example", port: 1080 }
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
## top-level comment
port: 8080

items: [
    ## this comment does not break the array
    a
    b
]
```

Comments are full lines starting with `#`. Inline comments are not
supported — they get confused with the value too easily.

### 9. Multi-line strings: `( ... )` and `(( ... ))`

Values that span multiple lines go inside parentheses. The opening and
closing lines are NOT part of the value.

`(` ... `)` — common leading whitespace is stripped and, as of 0.7,
trailing whitespace is stripped from each line, so you can indent
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

Serialization: a string is emitted on a single line only when it has
no `\n`, no leading/trailing whitespace, and no control byte other
than `TAB`. Anything else — including a plain string with a stray edge
space — takes a multi-line form: verbatim `(( ... ))` when the content
has edge whitespace that stripped would alter, stripped `( ... )`
otherwise; whichever the
writer picks, the round-trip is byte-for-byte lossless.

Which block form you get depends on *where* the whitespace is. As of
0.7 the stripped form strips trailing whitespace from every content
line (§ 5.6), so a trailing space forces the verbatim form, which
preserves it byte-for-byte:

```json5
{ password: "hunter2 " }
```
```text
password: ((
hunter2 
))
```

Leading whitespace — and, since 0.7, trailing whitespace — is what
forces the verbatim form: stripping would eat the leading indent:

```json5
{ indent: "  padded" }
```
```text
indent: ((
  padded
))
```

Either way, reading it back gives you the original bytes.

Limitation: a body containing a line whose trimmed content is exactly
`))` cannot use the verbatim form. It falls back to stripped instead —
unless the body also has a sole-`)` line, a whitespace-only line, a
line with trailing whitespace, or
every line indented (nothing to anchor the dedent at zero), in which
case no form can hold it and serialization returns an error rather
than emit a document that fails to round-trip.

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
## unit variant — just the name
mode: fast

## newtype variant — single-entry object
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

## Formatting — canonical spelling, comments kept

`ktav::format_str` rewrites a document into the structural spelling
`emit_canonical` produces, but keeps the trivia the canonical writer
drops. Every comment survives verbatim.

```rust
let tidied = ktav::format_str("## the server\nserver: {host: a, port: 80}\n")?;
assert_eq!(tidied, "## the server\nserver: {\n    host: a\n    port: 80\n}\n");
```

That is, the inline compound expands to the canonical multi-line
form and the comment stays exactly where it was:

```ktav
## the server
server: {
    host: a
    port: 80
}
```

Blank lines survive as a grouping hint, but a run of two or more
collapses to exactly one, and blank padding just inside a bracket is
dropped. That is what makes the transform a fixed point: formatting
already-formatted output never changes it again.

Key order is never changed. Canonical form has no sorting rule
(§ 5.9), and reordering keys would make review diffs worse, not
better — this is a spelling normaliser, not a refactoring tool.

For a document with no comments **and no blank lines**,
`format_str` equals `emit_canonical` of its parse. The stronger
condition is deliberate: blank lines are no more part of the `Value`
model than comments are, so `emit_canonical` drops them and
`format_str` does not.

### `ktav-fmt` — the optional command-line formatter

Behind the `cli` feature, off by default. In a project that already
has a toolchain the library call above plus a build hook is usually the
better answer, and editors format through `ktav-lsp`; the binary is
for the case where neither is at hand.

```text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
```

`--check` writes nothing and prints the path of every file that is
not already formatted, so it drops straight into CI next to
`cargo fmt --check`.

## Source editing (unreleased)

This API is under development and is **not included in the published 0.8.1 crate**.
Unlike `format_str`, `ktav::edit` does not reformat the whole document: it replaces an existing value in the original text. Bytes outside the effective edit range remain unchanged, including comments, blank lines, key spelling, ordering, indentation, BOM and existing line endings. The range normally contains only the target; when readability requires expanding inline ancestors, it includes those wrappers. A raw marker is adjusted only when the new value requires it. The input is borrowed; success returns a new `String`, and an error leaves the input untouched.

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;

let source = "## top\n\na: 1\n\n## end\n";
let replacement = Value::Integer("2".into());
let expected = "## top\n\na: 2\n\n## end\n";
assert_eq!(edit_value(source, "a", &replacement)?, expected);
assert_eq!(edit_value_segments(source, &["a"], &replacement)?, expected);
let path = vec![String::from("a")];
assert_eq!(edit_value_segments(source, &path, &replacement)?, expected);
```

The textual path `key1.key2.0.key4` uses `.` as a separator. Escape a literal dot as `\.` and a backslash as `\\`; other escapes, an unfinished escape and empty textual segments are errors. For a literal key `a.b`, the Rust path argument is `r"a\.b"` or `"a\\.b"`. The slice/vector API takes decoded names (`&["a.b"]`), without path escaping. In an object, `0` means the key `"0"`; in an array, it means index zero. Array indexes contain only ASCII digits; leading zeroes are accepted, signs are not.

### Preserving the source form

Dotted keys stay dotted, inline objects/arrays stay inline, and block objects/arrays stay blocks, including when emptied. This applies to deep edits inside arrays too. Replacing a scalar with a new container uses a block outside an inline parent; replacing a whole inline container preserves its inline form when readable. A type change does not by itself reformat ancestors. No-op edits bypass all readability rules.

An edited inline line is expanded only when the replacement contains a multiline string, the line grows beyond `max(100, original line length)`, or a replacement string needs at least four inline escapes with at least 25% extra encoded characters. Length means Unicode scalar values, not UTF-8 bytes or terminal columns; the escape overhead is compared with the decoded string length. Existing long lines are not expanded unless they grow.

Expansion opens the containing inline wrapper and only the affected nested wrappers still needing expansion; untouched nested branches may stay inline. Key spelling and order are preserved. Moved scalar spellings and raw markers change only when standalone syntax needs a different encoding. New block levels use four spaces and the source EOL style; no final newline is added. If a lossless standalone encoding is unavailable, for example because a string contains CR or a line equal to `))`, the valid inline representation is retained instead. Readability never authorizes comment loss or semantic changes. These fixed rules add no configuration mode.

A short paired benchmark lives in `benches/source_edit.rs`. Build it separately with `cargo bench --bench source_edit --no-run --message-format=json`, then run the reported executable with `baseline` or `candidate` as its argument. Apply an external 15-second process timeout (for example, `caps 15 <benchmark-executable> candidate` on Windows). Each case uses a 20 ms warmup and a 300 ms sample; setup and exact-output validation are outside the measured interval. Compare alternating A/B and B/A runs on the same toolchain; short-run wall-clock results are not a CI regression gate.

Replacement is joined by `insert_value`, `remove_value` and `rename_key` (see the section below): a new final Object key, deletion of a leaf/subtree/Array element, and an in-place member rename. An unchanged value returns the original text, including its original numeric spelling. New values are supplied as `Value`; a numeric-looking `Value::String` stays a string, using a raw marker where necessary.

`EditError::InvalidPath` reports a malformed textual path and byte offset; `NotFound` covers missing keys, invalid/out-of-range array indexes and descent through a scalar. `Unsupported` rejects an invalid source, an unrepresentable replacement or a replacement for which this implementation cannot preserve the surrounding text. In particular, a whole object assembled from separate dotted/reopened definitions has no single replacement range, and replacing a container must not delete its internal comments. Its individual leaves remain editable. Inline values use escapes for CR and LF. Outside inline compounds, CR-containing strings and some context-dependent multiline/container replacements are rejected rather than silently rewritten. Every returned patch is parsed and checked against the intended value tree; errors never return a partially edited document.

### Insertion, removal and renaming (unreleased)

`insert_value(source, path, value)` adds a new final Object key under an existing Object parent. The pair is appended as a whole line: at the end of the parent for block and implicit-root containers, as an inline segment before the closer for inline containers; the source EOL and the presence or absence of the final newline are preserved. An existing key is `EditError::AlreadyExists` — insertion never overwrites; insertion into an Array is an explicit `Unsupported`; missing intermediates and descent through a scalar are `NotFound`. Inline strings use the inline escaper (commas, brackets, backslash, line breaks, numeric-looking bodies); when the readability rules require it, the inline parent is expanded to block form; when a lossless block form is unavailable (a CR, a `))` line), the verified inline form is kept.

`remove_value(source, path)` deletes an existing leaf, a whole subtree, or one Array element (the tail shifts). Empty ancestors are not pruned: removing the only dotted leaf materializes the emptied dotted chain as a `k: {}` line at the same position, while a real block keeps its braces and an inline container keeps its inline form. Comment lines inside a removed container are refused with `Unsupported`; comments adjacent to the target are preserved. The BOM is kept as source metadata and never enters an edit range.

`rename_key(source, path, new_key)` renames a member in place, replacing only the last addressed key segment: dotted prefixes, quotes and escapes of the source key are kept byte-for-byte, and the new key is spelled with the standard bare/quoted encoding. An identical key is a byte-identical no-op; an existing sibling target is `AlreadyExists`; an Array index or an unrepresentable key is `Unsupported`. Order, value, separator bytes and comments are untouched.

All three operations have `*_segments` variants taking decoded literal segments, share the textual path grammar of `edit_value`, and run the same parse–splice–reparse–verify pipeline; an invalid source fails closed with `Unsupported`.

### Source-edit cookbook

These examples describe the unreleased 0.9 API, not the published 0.8.1 package. They replace existing values only: keys are not renamed, missing paths are not created, and entries are not removed. Each snippet is independent; Rust `?` assumes an enclosing `Result`-returning function. Go examples panic only for these fixed, known-valid inputs; applications should handle the returned error.

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;
```

#### 1. Dotted deployment settings

Only the address changes; the comment, dotted form and neighbouring user entry remain byte-for-byte unchanged.

```rust
let source = "## deployment\ntargets.storagehost.address: old\ntargets.storagehost.user: mcp-agent\n";
let updated = edit_value(source, "targets.storagehost.address", &Value::String("192.168.0.88:22".into()))?;
```

Result:

```ktav
## deployment
targets.storagehost.address: 192.168.0.88:22
targets.storagehost.user: mcp-agent
```

#### 2. Literal dots: textual path or segments

The quoted source key is literally `a.b`. The textual path escapes its dot; the segments call passes the decoded key directly. Both calls return the same result.

```rust
let source = "\"a.b\": {port: 80}\n";
let updated = edit_value(source, "a\\.b.port", &Value::Integer("90".into()))?;
let same = edit_value_segments(source, &["a.b", "port"], &Value::Integer("90".into()))?;
```

Result:

```ktav
"a.b": {port: 90}
```

#### 3. A leaf inside a mixed-form array

`0` is an array index here. Inside an Object, the same segment would address the key named `0`; untouched containers remain inline.

```rust
let source = "servers: [{host: old, keep: yes}, standby]\n";
let updated = edit_value_segments(source, &["servers", "0", "host"], &Value::String("new".into()))?;
```

Result:

```ktav
servers: [{host: new, keep: yes}, standby]
```

#### 4. A numeric-looking String stays a String

The replacement is a String, not an Integer. The raw marker preserves that distinction. Supply a typed Integer when a numeric value is intended.

```rust
let source = "port: 8080\n";
let updated = edit_value(source, "port", &Value::String("9090".into()))?;
```

Result:

```ktav
port:: 9090
```

#### 5. Multiline replacement: expand only what is necessary

The changed array expands with four-space indentation. The untouched object stays inline, and verbatim string contents are not reindented.

```rust
let source = "a: [old, {keep: yes}]\n";
let updated = edit_value(source, "a.0", &Value::String("x\ny".into()))?;
```

Result:

```ktav
a: [
    ((
x
y
    ))
    {keep: yes}
]
```

#### 6. No-op preserves noncanonical spelling

The Integer value is already 1. Equality returns the original bytes, including `+1` and the original separator spacing; readability thresholds are bypassed.

```rust
let source = "n  : +1\n";
let updated = edit_value(source, "n", &Value::Integer("1".into()))?;
assert_eq!(updated, source);
```

Result:

```ktav
n  : +1
```

#### 7. BOM, CRLF and an unterminated final line

The returned string keeps the BOM and CRLF, and does not gain a final newline. Its escaped representation is shown to make the bytes visible.

```rust
let source = "\u{FEFF}port: 8080\r\nhost: db";
let updated = edit_value(source, "port", &Value::Integer("9090".into()))?;
```

Result:

```text
"\uFEFFport: 9090\r\nhost: db"
```

Expansion is deterministic: a multiline replacement, growth beyond `max(100, original line length)`, or at least four escapes with at least 25% encoding overhead for an individual replacement string. Width counts Unicode scalar values, not UTF-8 bytes or terminal columns. An already-long line without growth is kept. New levels use four spaces and the source EOL. If CR or a `))` data line prevents lossless standalone encoding, a valid inline candidate is retained; these characters do not automatically make every inline edit unsupported.

#### 8. Exact Integer replacement

The source Integer domain is signed 64-bit. This value exceeds JavaScript's safe-Number range but fits i64; use an exact host representation, not a rounded floating-point value. A typed Integer outside i64 cannot be preserved as an Integer in source and is refused instead of truncated. Use a String when a large identifier is text. Float identity distinguishes negative and positive zero; Object no-op equality ignores key order, while Array order remains significant.

```rust
let updated = edit_value("n: 1\n", "n", &Value::Integer("9007199254740993".into()))?;
```

```ktav
n: 9007199254740993
```

### Diagnostics and safe failures in 0.9

Source coordinates are UTF-8 byte offsets, including a BOM; spans are half-open `[start, end)`, and lines are 1-based. LF, lone CR and CRLF are handled, with CRLF counted as one terminator. A Unicode character may occupy several bytes.

An unfinished `a: [\n` reports `UnclosedCompound`, line 1 and span `3..5`: the span begins at the innermost unmatched opener, not the start of its key or line. For raw bytes `61 3A 20 FF` (`a: ` followed by invalid UTF-8), validation reports `InvalidUtf8`, line 1 and a covering span `3..4`; `line_text` is null rather than fabricated by lossy decoding. UTF-8 validation precedes grammar processing.

A malformed textual path, including an empty textual path or empty textual segment, is `EditInvalidPath`; its offset is measured in the path's UTF-8 bytes. An empty segments list, missing key, out-of-range index or scalar descent is `EditNotFound`. Object numeric segments remain keys; array indexes use ASCII digits, allow leading zeros and reject signs.

Whole dotted/reopened intermediate objects may occupy several physical ranges and are `EditUnsupported`; their existing leaves can still be edited. Replacing a container must not discard interior comments. No failed call returns a partial patch or changes the supplied source. The returned text is re-parsed and semantically checked: the target must equal the replacement and the rest of the value tree must remain equal.

For bindings, invalid source is reported with the ordinary parser envelope before path validation. Malformed wire payloads are `Message` errors without invented source coordinates. The wire envelope has `error`, `reason`, `line`, `line_text`, `span`, `path`, `body`, `canonical`, `spec_section` and `message`; host accessor names are documented in the errors section. `body` carries the decimal path-byte offset for `EditInvalidPath`, null for `EditNotFound`, and the refusal reason for `EditUnsupported`.

The coordinate examples describe ordinary parse errors (`Error` / `ErrorEnvelope`). Rust source editing itself returns `EditError::{InvalidPath, NotFound, Unsupported}`: `InvalidPath { offset }` refers to the path bytes, and an invalid source is wrapped as `Unsupported`. The C ABI validates the source first and exposes the ordinary source envelope to bindings. These Rust variants are not the wire envelope field layout.

## The C ABI for other languages

The Go, Java, PHP, C#, JS and Python bindings share the Rust core. The off-by-default `cabi` feature provides wire decoding, document operations and error encoding; one macro call in a binding's `cdylib` exports the C symbols. Direct Rust bridges such as PyO3 and N-API can use the same safe operations without another FFI layer.

```text
// crates/cabi/src/lib.rs of a binding:
ktav::declare_cabi!();
```

The **unreleased 0.9 preparation uses ABI 2**, with twelve symbols: nine document functions (`ktav_loads`, `ktav_loads_strict`, `ktav_dumps`, `ktav_dumps_force_strings`, `ktav_emit_canonical`, `ktav_format`, `ktav_canonical_from_source`, `ktav_edit_value`, `ktav_edit_value_segments`), `ktav_free`, `ktav_version` and `ktav_abi_version`. Hosts must reject an ABI-1 binary rather than silently falling back to a different backend. This does not publish or change a binding package version.

Both edit functions take three length-delimited byte inputs: Ktav source, path and replacement JSON wire value. The first path is UTF-8 dotted text; the second is a JSON array of literal decoded strings. Tagged numeric values (`$i` / `$f`) preserve scalar types and precision; the replacement may itself be a scalar. Success returns UTF-8 Ktav source, not a JSON string, using the same lossless editing and readability rules as `ktav::edit`.

Every error uses the existing ten-field JSON envelope. Source diagnostics retain the 0.9 raw-byte locations. Edit failures use `EditInvalidPath`, `EditNotFound` or `EditUnsupported`, without invented source positions: `body` carries the decimal path byte offset for `EditInvalidPath`, the reason for `EditUnsupported`, or null for `EditNotFound`. Transport JSON/UTF-8 errors outside the source use `Message`; the host surfaces `message` as its error text.

The full signatures, ownership contract and artifact naming convention (`ktav_cabi-windows-amd64.dll`, `libktav_cabi-darwin-arm64.dylib`, `libktav_cabi-linux-amd64.so`, `$KTAV_LIB_PATH`) are specified in [docs/CABI.md](docs/CABI.md).

## Architecture

```
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── thin/             — arena-backed borrowed parse (parse_events)
├── render/           — pretty-printer, canonical writer, formatter
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error, ErrorKind, ErrorEnvelope, serde::Error
├── bin/ktav-fmt.rs   — the ktav-fmt command-line formatter
└── lib.rs            — glue: from_str / to_string / format_str / …
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

The published dependency below is 0.8.1; it does not include the new path
editing or 0.9 error model. Version 0.9.0 currently requires this source checkout.

```toml
[dependencies]
ktav = "0.8.1"
serde = { version = "1", features = ["derive"] }
```

The formatter is also available as a binary, behind an off-by-default
feature:

```sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
```

## Support the project

The author has many ideas that could be broadly useful to IT worldwide —
not limited to Ktav. Realizing them requires funding. If you'd like to
help, please reach out at **phpcraftdream@gmail.com**.

## License

Dual-licensed under **MIT OR Apache-2.0** at your option. See
[LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

## Other Ktav implementations

- [`spec`](https://github.com/ktav-lang/spec) — specification + conformance suite
- [`csharp`](https://github.com/ktav-lang/csharp) — C# / .NET (`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) — Go (`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) — Java / JVM (`io.github.ktav-lang:ktav` on Maven Central)
- [`js`](https://github.com/ktav-lang/js) — JS / TS (`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) — PHP (`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) — Python (`pip install ktav`)
