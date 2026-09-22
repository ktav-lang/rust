>>>>> lang=en
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

>>>>> lang=ru
### 11. Enum-ы

Ktav использует дефолтное *externally tagged* представление enum-ов
serde.

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

>>>>> lang=zh
### 11. 枚举

Ktav 使用 serde 默认的 *externally tagged* 枚举表示形式。

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

