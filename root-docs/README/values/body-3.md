>>>>> lang=en
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

>>>>> lang=ru
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

Строго нижний регистр. На стороне Rust соответствует `Option::None`,
а также `()` для unit.

```text
## Value::Null
label: null
## Value::String("Null")
capitalized: Null
## Value::String("null")
literal:: null
```

При сериализации `Option::None` эмитится как `null`. Подавить можно
через `#[serde(skip_serializing_if = "Option::is_none")]`, если вы
предпочитаете, чтобы поля не было вовсе.

### Пустой объект / пустой массив

>>>>> lang=zh
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

### Null:`null`

严格小写。对应 Rust 侧的 `Option::None`,以及 unit 类型 `()`。

```text
## Value::Null
label: null
## Value::String("Null")
capitalized: Null
## Value::String("null")
literal:: null
```

序列化时,`Option::None` 会输出为 `null`。若你希望该字段干脆缺省,
可以加上 `#[serde(skip_serializing_if = "Option::is_none")]`。

### 空对象 / 空数组

