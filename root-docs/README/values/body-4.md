>>>>> lang=en
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

>>>>> lang=ru
**Единственные** allowed inline compound-значения — разделять нечего,
запятые не нужны.

```text
## пустой объект
meta: {}
## пустой массив
tags: []
```

### Ключеподобные строки требуют `::`

Если содержимое строки совпадает с ключевым словом (`true`, `false`,
`null`) или начинается с `{` или `[`, **сериализатор автоматически
эмитит `::`**, чтобы round-trip был без потерь. На стороне записи
поступайте так же:

```text
## строка "true", а не булево
flag:: true
## строка "null", а не Null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
```

>>>>> lang=zh
**唯一**允许的内联复合值——没什么要分隔,不需要逗号。

```text
## 空对象
meta: {}
## 空数组
tags: []
```

### 与关键字同形的字符串需要 `::`

若字符串内容恰好等于关键字(`true`、`false`、`null`),或以 `{`、
`[` 开头,**序列化器会自动输出 `::`**,以保证 round-trip 无损。
写入端请用同样的方式:

```text
## 字符串 "true",而非 Bool
flag:: true
## 字符串 "null",而非 Null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
```

