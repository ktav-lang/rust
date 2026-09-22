>>>>> lang=en
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

>>>>> lang=ru
```json
{"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
 "span":{"start":0,"end":7},"path":null,"body":"1.10",
 "canonical":"1.1","spec_section":"§3.6/§5.2",
 "message":"Syntax error: Line 1: LossyScalar: '1.10' would be …"}
```

`message` — последнее поле и единственное, которое никогда не
`null`: в нём дословно лежит `Display`-представление этой ошибки.
Биндинг показывает его пользователю как есть, а не собирает текст сам
из структурированных полей, поэтому один и тот же документ даёт
одинаковый текст ошибки на любом языке. Девять прежних полей сохранили
свои позиции — `message` дописано в конец, а не вставлено в середину.

Отсутствующие сведения — явный `null`, а не пропущенный ключ, поэтому
потребитель читает каждое поле позиционно, без предварительного
согласования схемы.

`span` — это `{"start":N,"end":M}` в **байтовых смещениях по
UTF-8-исходнику**, а не в code units UTF-16: та же единица, что у самого
[`Span`](https://docs.rs/ktav). Потребителю LSP нужно либо
конвертировать, либо договариваться о `positionEncoding: "utf-8"`.

`path` — **массив точных декодированных сегментов ключа, а не
склеенная строка**. Ключ, буквально названный `a.b`, — это один
сегмент, и спутать его с путём из двух нельзя: в контракте просто нет
разделителя, вокруг которого возникла бы двусмысленность.

>>>>> lang=zh
```json
{"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
 "span":{"start":0,"end":7},"path":null,"body":"1.10",
 "canonical":"1.1","spec_section":"§3.6/§5.2",
 "message":"Syntax error: Line 1: LossyScalar: '1.10' would be …"}
```

`message` 是最后一个字段,也是唯一永远不为 `null` 的字段:它逐字
携带该错误的 `Display` 渲染结果。绑定层直接把它展示给用户,而不是自己
从结构化字段拼装文字,因此同一份文档在任何语言下都会给出相同的错误文本。
原有的九个字段保持各自的位置 —— `message` 是追加的,而非插入的。

缺失的信息是显式的 `null`,而不是省略键,因此使用方无需事先协商模式
即可按位置读取每个字段。

`span` 是 `{"start":N,"end":M}`,单位为 **UTF-8 源文本中的字节
偏移**,而不是 UTF-16 code unit —— 与 [`Span`](https://docs.rs/ktav)
本身一致。LSP 使用方要么自行转换,要么协商
`positionEncoding: "utf-8"`。

`path` 是**精确解码后的键段数组,绝不是拼接字符串**。字面名为
`a.b` 的键是一个段,不会与两段路径混淆 —— 传输契约里根本没有可供
产生歧义的分隔符。

