>>>>> lang=en
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

>>>>> lang=ru
`((` ... `))` — побайтово: каждый символ между маркерами попадает в
значение, включая ведущие пробелы:

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

Внутри блока `{` / `[` / `#` — просто содержимое, **никакого разбора
compound, никакого пропуска комментариев**. Единственная особая
последовательность — терминатор на отдельной строке.

Пустая инлайн-форма: `key: ()` или `key: (())` — обе дают пустую
строку (то же, что `key:`).

>>>>> lang=zh
`((` ... `))` —— 逐字节保留:开始与结束标记之间的每一个字符都
进入值,包括前导空白:

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

在块内,`{` / `[` / `#` 只是内容——**不做复合解析,不跳过注释**。
唯一的特殊序列就是独占一行的终止符。

空的内联形式:`key: ()` 或 `key: (())`——都会产生空字符串(等同
于 `key:`)。

