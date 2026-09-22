>>>>> lang=en
```ktav
inline: {greek: \u03b1, csv: a\,b}
"\u00e9": 1
literal: \u0041
```

`greek` is `α`, `csv` is the single string `a,b` — the escaped
comma is not a separator — and the quoted key is `é`. But `literal` is
the eight characters `\u0041`, unchanged. A recognised escape also
forces String classification: a value written `\u0031` is the string
`1`, not the integer.

### Exactly one leading byte-order mark is skipped

A U+FEFF at the very start of the document is skipped before any
other byte is examined (spec § 3.1). Anywhere else it is ordinary
content — including the start of a later line, where it becomes part of
that key's name. Editors that add a BOM on save therefore do not break
a document, and a stray one further in does not silently disappear.

### Invalid UTF-8 is its own error, not an I/O failure

[`from_file`](https://docs.rs/ktav) validates the file's bytes as
UTF-8 before parsing and reports [`Error::InvalidUtf8`] with the byte
offset of the first bad sequence (spec § 6.15). A missing file or a
permission problem stays [`Error::Io`] — the two are worth
distinguishing, because one means "fix the file" and the other means
"fix the path". The bytes are never repaired or replaced before the
parser sees them.

>>>>> lang=ru
```ktav
inline: {greek: \u03b1, csv: a\,b}
"\u00e9": 1
literal: \u0041
```

`greek` — это `α`, `csv` — одна строка `a,b` (экранированная
запятая не разделитель), а кавычный ключ — `é`. Зато `literal` — это
восемь символов `\u0041` без изменений. Распознанное экранирование
также принудительно делает значение строкой: значение `\u0031` — это
строка `1`, а не целое.

### Ровно один ведущий BOM пропускается

U+FEFF в самом начале документа пропускается до того, как будет
рассмотрен любой другой байт (§ 3.1). В любом другом месте это обычное
содержимое — в том числе в начале следующей строки, где он становится
частью имени ключа. Поэтому редактор, дописывающий BOM при сохранении,
документ не ломает, а случайный BOM в середине не исчезает молча.

### Невалидный UTF-8 — отдельная ошибка, а не сбой ввода-вывода

[`from_file`](https://docs.rs/ktav) проверяет байты файла на
UTF-8 до разбора и сообщает [`Error::InvalidUtf8`] со смещением первой
некорректной последовательности (§ 6.15). Отсутствующий файл или
проблема с правами остаются [`Error::Io`] — различать их стоит, потому
что одно означает «почини файл», а другое — «почини путь». Байты никогда
не чинятся и не подменяются до того, как их увидит парсер.

>>>>> lang=zh
```ktav
inline: {greek: \u03b1, csv: a\,b}
"\u00e9": 1
literal: \u0041
```

`greek` 是 `α`,`csv` 是单个字符串 `a,b`(被转义的逗号不是
分隔符),引号键是 `é`。但 `literal` 是原封不动的八个字符
`\u0041`。被识别的转义还会强制归类为字符串:写作 `\u0031` 的值是
字符串 `1`,而不是整数。

### 恰好跳过一个前导 BOM

文档最开头的 U+FEFF 会在检查任何其他字节之前被跳过(§ 3.1)。出现
在其他位置时它是普通内容 —— 包括后续行的开头,在那里它会成为该键名字的
一部分。因此保存时添加 BOM 的编辑器不会破坏文档,而中间位置的多余 BOM
也不会悄然消失。

### 非法 UTF-8 是独立的错误,而非 I/O 失败

[`from_file`](https://docs.rs/ktav) 在解析前校验文件字节是否为
UTF-8,并以首个非法序列的字节偏移报告 [`Error::InvalidUtf8`](§ 6.15)。
文件缺失或权限问题仍然是 [`Error::Io`] —— 二者值得区分,因为一个意味着
「修文件」,另一个意味着「修路径」。字节在解析器看到之前绝不会被修复或
替换。

