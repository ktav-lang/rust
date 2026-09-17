// README units: corners section.
export const corners = [
  {
    id: 'corners-heading',
    join: 'block',
    en: `## Corners worth knowing`,
    ru: `## Углы, о которых стоит знать`,
    zh: `## 值得知道的边角`,
  },
  {
    id: 'corners-intro',
    join: 'block',
    en: `Four rules the examples above never reach. Each one is settled by the
specification rather than by this implementation, so every conforming
parser behaves the same way.`,
    ru: `Четыре правила, до которых примеры выше не доходят. Каждое решено
спецификацией, а не этой реализацией, поэтому любой соответствующий
парсер ведёт себя так же.`,
    zh: `以上示例未触及的四条规则。每一条都由规范而非本实现决定,因此任何
符合规范的解析器行为都一致。`,
  },
  {
    id: 'corners-quoted-keys-heading',
    join: 'block',
    en: `### Quoted keys — when a key contains a dot or a space`,
    ru: `### Кавычные ключи — когда в ключе точка или пробел`,
    zh: `### 引号键 —— 当键里有点号或空格时`,
  },
  {
    id: 'corners-quoted-keys-body',
    join: 'block',
    en: `A dot in a bare key means nesting: \`db.host: primary\` builds
\`db\` → \`host\`. Quoting the key turns off that reading, so the dot is
part of the name (spec § 5.3.3):`,
    ru: `Точка в голом ключе означает вложенность: \`db.host: primary\`
строит \`db\` → \`host\`. Кавычки отключают это прочтение, и точка
становится частью имени (§ 5.3.3):`,
    zh: `裸键中的点号表示嵌套:\`db.host: primary\` 会构建 \`db\` → \`host\`。
给键加引号会关闭这一解读,点号因此成为名字的一部分(§ 5.3.3):`,
  },
  {
    id: 'corners-quoted-keys-snippet',
    join: 'block',
    common: `\`\`\`ktav
db.host: primary
"db.host": literal
"a b": spaces are fine too
\`\`\``,
  },
  {
    id: 'corners-quoted-keys-note',
    join: 'block',
    en: `The first line nests. The second is a single key literally named
\`db.host\`. This is why the error envelope reports \`path\` as an array
of segments rather than a joined string — a joined string could not tell
those two apart.`,
    ru: `Первая строка вкладывает. Вторая — один ключ, буквально названный
\`db.host\`. Именно поэтому конверт ошибок отдаёт \`path\` массивом
сегментов, а не склеенной строкой: склеенная строка не смогла бы
различить эти два случая.`,
    zh: `第一行产生嵌套。第二行是一个字面名为 \`db.host\` 的键。这正是错误
信封把 \`path\` 表示为段数组而非拼接字符串的原因 —— 拼接字符串无法区分
这两者。`,
  },
  {
    id: 'corners-escapes-heading',
    join: 'block',
    en: `### Escapes apply inside inline compounds and quoted keys`,
    ru: `### Экранирование действует внутри inline-компаундов и кавычных ключей`,
    zh: `### 转义在行内复合值与引号键中生效`,
  },
  {
    id: 'corners-escapes-body',
    join: 'block',
    en: `\`\\uXXXX\` and the named escapes (spec § 3.7, § 3.7.1) are read where
a delimiter would otherwise be structural — inside \`{ }\` and \`[ ]\`,
and inside a quoted key. A bare block-level value has no delimiters to
escape, so a backslash there is ordinary text:`,
    ru: `\`\\uXXXX\` и именованные escape-последовательности (§ 3.7, § 3.7.1)
читаются там, где разделитель иначе был бы структурным — внутри
\`{ }\` и \`[ ]\`, а также внутри кавычного ключа. У голого блочного
значения экранировать нечего, поэтому обратный слэш там — обычный
текст:`,
    zh: `\`\\uXXXX\` 与具名转义(§ 3.7、§ 3.7.1)在分隔符本应具有结构含义的
位置被解读 —— 即 \`{ }\` 与 \`[ ]\` 内部,以及引号键内部。裸的块级值没有
需要转义的分隔符,因此其中的反斜杠是普通文本:`,
  },
  {
    id: 'corners-escapes-snippet',
    join: 'block',
    common: `\`\`\`ktav
inline: {greek: \\u03b1, csv: a\\,b}
"\\u00e9": 1
literal: \\u0041
\`\`\``,
  },
  {
    id: 'corners-escapes-note',
    join: 'block',
    en: `\`greek\` is \`α\`, \`csv\` is the single string \`a,b\` — the escaped
comma is not a separator — and the quoted key is \`é\`. But \`literal\` is
the eight characters \`\\u0041\`, unchanged. A recognised escape also
forces String classification: a value written \`\\u0031\` is the string
\`1\`, not the integer.`,
    ru: `\`greek\` — это \`α\`, \`csv\` — одна строка \`a,b\` (экранированная
запятая не разделитель), а кавычный ключ — \`é\`. Зато \`literal\` — это
восемь символов \`\\u0041\` без изменений. Распознанное экранирование
также принудительно делает значение строкой: значение \`\\u0031\` — это
строка \`1\`, а не целое.`,
    zh: `\`greek\` 是 \`α\`,\`csv\` 是单个字符串 \`a,b\`(被转义的逗号不是
分隔符),引号键是 \`é\`。但 \`literal\` 是原封不动的八个字符
\`\\u0041\`。被识别的转义还会强制归类为字符串:写作 \`\\u0031\` 的值是
字符串 \`1\`,而不是整数。`,
  },
  {
    id: 'corners-bom-heading',
    join: 'block',
    en: `### Exactly one leading byte-order mark is skipped`,
    ru: `### Ровно один ведущий BOM пропускается`,
    zh: `### 恰好跳过一个前导 BOM`,
  },
  {
    id: 'corners-bom-body',
    join: 'block',
    en: `A U+FEFF at the very start of the document is skipped before any
other byte is examined (spec § 3.1). Anywhere else it is ordinary
content — including the start of a later line, where it becomes part of
that key's name. Editors that add a BOM on save therefore do not break
a document, and a stray one further in does not silently disappear.`,
    ru: `U+FEFF в самом начале документа пропускается до того, как будет
рассмотрен любой другой байт (§ 3.1). В любом другом месте это обычное
содержимое — в том числе в начале следующей строки, где он становится
частью имени ключа. Поэтому редактор, дописывающий BOM при сохранении,
документ не ломает, а случайный BOM в середине не исчезает молча.`,
    zh: `文档最开头的 U+FEFF 会在检查任何其他字节之前被跳过(§ 3.1)。出现
在其他位置时它是普通内容 —— 包括后续行的开头,在那里它会成为该键名字的
一部分。因此保存时添加 BOM 的编辑器不会破坏文档,而中间位置的多余 BOM
也不会悄然消失。`,
  },
  {
    id: 'corners-utf8-heading',
    join: 'block',
    en: `### Invalid UTF-8 is its own error, not an I/O failure`,
    ru: `### Невалидный UTF-8 — отдельная ошибка, а не сбой ввода-вывода`,
    zh: `### 非法 UTF-8 是独立的错误,而非 I/O 失败`,
  },
  {
    id: 'corners-utf8-body',
    join: 'block',
    en: `[\`from_file\`](https://docs.rs/ktav) validates the file's bytes as
UTF-8 before parsing and reports [\`Error::InvalidUtf8\`] with the byte
offset of the first bad sequence (spec § 6.15). A missing file or a
permission problem stays [\`Error::Io\`] — the two are worth
distinguishing, because one means "fix the file" and the other means
"fix the path". The bytes are never repaired or replaced before the
parser sees them.`,
    ru: `[\`from_file\`](https://docs.rs/ktav) проверяет байты файла на
UTF-8 до разбора и сообщает [\`Error::InvalidUtf8\`] со смещением первой
некорректной последовательности (§ 6.15). Отсутствующий файл или
проблема с правами остаются [\`Error::Io\`] — различать их стоит, потому
что одно означает «почини файл», а другое — «почини путь». Байты никогда
не чинятся и не подменяются до того, как их увидит парсер.`,
    zh: `[\`from_file\`](https://docs.rs/ktav) 在解析前校验文件字节是否为
UTF-8,并以首个非法序列的字节偏移报告 [\`Error::InvalidUtf8\`](§ 6.15)。
文件缺失或权限问题仍然是 [\`Error::Io\`] —— 二者值得区分,因为一个意味着
「修文件」,另一个意味着「修路径」。字节在解析器看到之前绝不会被修复或
替换。`,
  },
];
