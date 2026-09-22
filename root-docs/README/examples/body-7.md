>>>>> lang=en
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

>>>>> lang=ru
```text
## top-level comment
port: 8080

items: [
    ## this comment does not break the array
    a
    b
]
```

Комментарии — целые строки, начинающиеся с `#`. Inline-комментарии
не поддерживаются — их слишком легко спутать со значением.

### 9. Многострочные строки: `( ... )` и `(( ... ))`

Значения, занимающие несколько строк, заключаются в круглые скобки.
Открывающая и закрывающая строки НЕ входят в значение.

`(` ... `)` — общий ведущий отступ срезается, а с 0.7 с каждой строки
срезается ещё и замыкающий пробельный хвост, так что можно выравнивать
блок по окружающему коду, не засоряя содержимое:

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

>>>>> lang=zh
```text
## top-level comment
port: 8080

items: [
    ## this comment does not break the array
    a
    b
]
```

注释为整行,以 `#` 开头。不支持行内注释——太容易与值混淆。

### 9. 多行字符串:`( ... )` 与 `(( ... ))`

跨越多行的值放在圆括号里。开起行与关闭行**不**属于值。

`(` ... `)` —— 剥除公共前导缩进,且从 0.7 起还会剥除每行末尾的空白,
所以你可以按照周围代码的缩进
书写,而不会污染内容:

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

