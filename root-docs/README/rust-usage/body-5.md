>>>>> lang=en
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

>>>>> lang=ru
Варианты: `MissingSeparatorSpace`, `InvalidTypedScalar`, `DuplicateKey`,
`KeyPathConflict`, `EmptyKey`, `InvalidKey`, `UnclosedCompound`,
`UnbalancedBracket`, `InlineNonEmptyCompound`, `MissingSeparator`,
`LossyScalar` (только в строгом режиме, см. ниже), `Other`. Enum помечен
`#[non_exhaustive]` — обязателен arm `_ =>`. `Error::line()` /
`Error::span()` — convenience accessors для случаев когда вариант
неважен. `Display` выдаёт те же читаемые строки, что и legacy
`Error::Syntax(_)`, поэтому существующие string-based вызывающие
работают без изменений.

Полный запускаемый пример проходит по всем вариантам:
[`examples/errors.rs`](examples/errors.rs) — `cargo run --example errors`.

### Один JSON-конверт для любой структурированной ошибки

Аксессоры выше существуют только в Rust. `ErrorEnvelope` — контракт
для всех остальных: один JSON-объект, десять полей, всегда все
десять, в фиксированном порядке — `error`, `reason`, `line`,
`line_text`, `span`, `path`, `body`, `canonical`,
`spec_section`, `message`.

```rust
use ktav::{parse, ErrorEnvelope};

let src = "a: 1.10\n";
if let Err(e) = ktav::parse_strict(src) {
    println!("{}", ErrorEnvelope::from_error(&e, src).to_json());
}
```

>>>>> lang=zh
变体:`MissingSeparatorSpace`、`InvalidTypedScalar`、`DuplicateKey`、
`KeyPathConflict`、`EmptyKey`、`InvalidKey`、`UnclosedCompound`、
`UnbalancedBracket`、`InlineNonEmptyCompound`、`MissingSeparator`、
`LossyScalar`（仅严格模式，见下文）、`Other`。该枚举标注
`#[non_exhaustive]` —— 必须包含 `_ =>` 分支。变体不重要时可使用便捷
访问器 `Error::line()` / `Error::span()`。`Display` 输出的字符串与
遗留 `Error::Syntax(_)` 完全一致,因此原有基于字符串的调用方无需修改
即可继续工作。

完整可运行示例遍历所有变体:
[`examples/errors.rs`](examples/errors.rs) —— `cargo run --example errors`。

### 任何结构化错误共用一个 JSON 信封

上面的访问器只存在于 Rust。`ErrorEnvelope` 是给其余各方的传输契约:
一个 JSON 对象,十个字段,永远是全部十个,顺序固定 —— `error`、
`reason`、`line`、`line_text`、`span`、`path`、`body`、
`canonical`、`spec_section`、`message`。

```rust
use ktav::{parse, ErrorEnvelope};

let src = "a: 1.10\n";
if let Err(e) = ktav::parse_strict(src) {
    println!("{}", ErrorEnvelope::from_error(&e, src).to_json());
}
```

