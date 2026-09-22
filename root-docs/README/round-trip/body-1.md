>>>>> lang=en
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

>>>>> lang=ru
## Round-trip

```rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
```

Сериализация сохраняет:
- **Порядок полей** — `Value::Object` лежит на `IndexMap`, так что
  порядок — тот, что эмитит serde (для struct-ов: порядок объявления).
- **Литеральные строки** — значения, начинающиеся с `{` или `[`,
  эмитятся с маркером `::`.
- **Поля `None`** — пропускаются на выходе; восстанавливаются как
  `None` на входе (через обработку `Option` в serde).

>>>>> lang=zh
## Round-trip

```rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
```

序列化会保持:
- **字段顺序** —— `Value::Object` 底层是 `IndexMap`,顺序由 serde
  输出决定(对结构体而言:就是声明顺序)。
- **字面量字符串** —— 以 `{` 或 `[` 开头的值会带 `::` 标记输出。
- **`None` 字段** —— 输出时跳过;输入时通过 serde 的 `Option`
  处理重新出现为 `None`。

