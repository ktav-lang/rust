>>>>> lang=en
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

>>>>> lang=ru
```json5
{ indent: "  padded" }
```
```text
indent: ((
  padded
))
```

В обоих случаях при обратном чтении получите исходные байты.

Ограничение: тело, где есть строка с trimmed-содержимым ровно `))`,
не может использовать дословную форму. Вместо неё берётся форма со
снятым отступом — если только в теле нет ещё и строки ровно `)`,
строки из одних пробелов, строки с замыкающим пробельным хвостом,
или все строки с отступом (не от чего
отсчитывать выравнивание к нулю) — тогда ни одна форма не подходит, и
сериализация возвращает ошибку вместо документа, который не
восстановится обратно.

### 10. Пустые compound-ы

```text
meta: {}
tags: []
```

Inline-пустой разрешён. Всё с содержимым обязано занимать несколько
строк, и закрывающий `}` / `]` обязан стоять на отдельной строке.

>>>>> lang=zh
```json5
{ indent: "  padded" }
```
```text
indent: ((
  padded
))
```

两种情况读回来都能得到原始字节。

限制:若正文中有一行 trim 后恰好是 `))`,则不能使用逐字形式,会
改用去缩进形式——除非正文中还含有恰好是 `)` 的一行、只含空白的
一行、末尾带空白的一行,或所有行都有缩进(没有可用于将去缩进基准定为零的行),此时
没有任何形式能容纳该正文,序列化会返回错误,而不是输出一个无法
还原的文档。

### 10. 空复合值

```text
meta: {}
tags: []
```

允许内联空。带内容的值必须跨越多行,且闭合的 `}` / `]` 必须独占
一行。

