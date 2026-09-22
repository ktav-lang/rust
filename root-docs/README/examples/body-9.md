>>>>> lang=en
Serialization: a string is emitted on a single line only when it has
no `\n`, no leading/trailing whitespace, and no control byte other
than `TAB`. Anything else — including a plain string with a stray edge
space — takes a multi-line form: verbatim `(( ... ))` when the content
has edge whitespace that stripped would alter, stripped `( ... )`
otherwise; whichever the
writer picks, the round-trip is byte-for-byte lossless.

Which block form you get depends on *where* the whitespace is. As of
0.7 the stripped form strips trailing whitespace from every content
line (§ 5.6), so a trailing space forces the verbatim form, which
preserves it byte-for-byte:

```json5
{ password: "hunter2 " }
```
```text
password: ((
hunter2 
))
```

Leading whitespace — and, since 0.7, trailing whitespace — is what
forces the verbatim form: stripping would eat the leading indent:

>>>>> lang=ru
Сериализация: строка выходит в одну строку, только если в ней нет
`\n`, нет ведущих/замыкающих пробельных символов и нет управляющих
байтов кроме `TAB`. Всё остальное — включая обычную строку со
случайным пробелом на краю — уходит в многострочную форму: дословную
`(( ... ))`, если у содержимого есть крайние пробельные символы,
которые снятие отступа исказит, иначе форму со снятым отступом
`( ... )`; какую бы форму writer ни выбрал, round-trip
байт-в-байт без потерь.

Какая именно блочная форма — зависит от того, *где* пробелы. С 0.7
форма со снятым отступом срезает замыкающий пробельный хвост с каждой
строки содержимого (§ 5.6), так что замыкающий пробел вынуждает взять
дословную форму, которая сохраняет его байт-в-байт:

```json5
{ password: "hunter2 " }
```
```text
password: ((
hunter2 
))
```

Ведущие пробелы — а с 0.7 ещё и замыкающие — вынуждают взять дословную
форму: снятие отступа съело бы ведущий отступ:

>>>>> lang=zh
序列化:字符串只有在不含 `\n`、没有首尾空白、且不含除 `TAB`
以外的控制字节时才会以单行形式输出。其他情况——哪怕只是普通字符串
边缘多了一个空格——都会采用多行形式:当内容含有会被去缩进破坏的
边缘空白时采用逐字形式 `(( ... ))`,否则采用去缩进形式
`( ... )`;无论 writer 选择哪种形式,
round-trip 都是字节级无损的。

具体采用哪种块形式,取决于空白出现在**哪一侧**。从 0.7 起,去缩进
形式会剥除每行内容末尾的空白(§ 5.6),因此末尾空格必须使用逐字
形式,由它逐字节保留:

```json5
{ password: "hunter2 " }
```
```text
password: ((
hunter2 
))
```

前导空白——从 0.7 起还包括末尾空白——必须使用逐字形式:去缩进会
把前导缩进吃掉:

