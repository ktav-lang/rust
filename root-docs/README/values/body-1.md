>>>>> lang=en
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

>>>>> lang=ru
## Значения и специальные токены

### Строки

Дефолт для любого скаляра. Внутри хранятся как `Value::String`.
Значение — это всё, что идёт после `:`, после обрезки пробелов.

```text
name: Russia
path: /etc/hosts
greeting: hello world
## `::` принудительно задаёт литеральную строку
pattern:: [a-z]+
```

### Числа

Числа пишутся без кавычек и типизируются по лексической форме: голое
целое тело разбирается в `Value::Integer`, голая десятичная запись —
в `Value::Float`. Каждый хранит *нормализованный* payload, а не
исходное написание: `Integer` хранит канонический десятичный вид (без
подчёркиваний, без `+`), `Float` — кратчайшую десятичную
форму, восстанавливающую точные биты `f64` — `+1_000` становится
`Integer("1000")`, `1.0e+2` становится `Float("100.0")`. Десятичное
число, чьи цифры начинаются с избыточного `0`, вовсе не число:
`zip: 01234` — это `String("01234")` (§ 5.2), поэтому дополненный
нулями идентификатор сохраняется дословно. `Integer` на
уровне `Value` покрывает диапазон i64; более широкий native Rust
целочисленный тип (`u64`, `i128`, `u128`), не помещающийся в i64, при
проходе через `ser::to_value` сохраняется как `Value::String` — точно
так же, как если бы этот же десятичный текст был разобран парсером.
serde десериализует числа в целевой Rust-тип (`u16`, `i64`, `i128`,
`f64`, …) прямым разбором и форматирует их с той же канонизацией при
сериализации.

>>>>> lang=zh
## 值与特殊记号

### 字符串

任何标量的默认类型。在内部以 `Value::String` 存放。值就是 `:` 之后
经过空白修剪的内容。

```text
name: Russia
path: /etc/hosts
greeting: hello world
## `::` 强制将值解释为字面量字符串
pattern:: [a-z]+
```

### 数字

数字不加引号,并根据字面形式分类:裸整数 body 解析为
`Value::Integer`,裸小数解析为 `Value::Float`。两者存放的都是
*规范化*后的 payload,而非原始写法:`Integer` 保存规范十进制形式
(无下划线、无 `+`),`Float` 保存能还原精确 `f64` 位模式的最短
十进制形式——`+1_000` 变为 `Integer("1000")`,`1.0e+2` 变为
`Float("100.0")`。数字以冗余的 `0` 开头的十进制根本不是数字:
`zip: 01234` 是 `String("01234")`(§ 5.2),因此补零的标识符会被
逐字保留。`Value` 层的 `Integer` 覆盖 i64 范围;比 i64 更宽的
native Rust 整数类型(`u64`、`i128`、`u128`)若超出该范围,经过
`ser::to_value` 时会存为 `Value::String`——效果与直接解析同一段十进制
文本完全一致。serde 在反序列化时直接将数字解析为目标 Rust 类型
(`u16`、`i64`、`i128`、`f64`……),并在序列化时用相同的规范化格式化。

