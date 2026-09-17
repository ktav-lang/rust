// README units: examples section.
export const examples = [
  {
    id: 'examples-heading',
    join: 'block',
    en: `## Examples: Ktav → JSON5`,
    ru: `## Примеры: Ktav → JSON5`,
    zh: `## 示例:Ktav → JSON5`,
  },
  {
    id: 'examples-json5-rationale',
    join: 'block',
    en: `JSON5 is on the right because it reads like ordinary JavaScript, allows
comments, and shows exactly what the parser produces.`,
    ru: `JSON5 справа потому, что читается как обычный JavaScript, допускает
комментарии и показывает ровно то, что производит парсер.`,
    zh: `右侧使用 JSON5,因为它读起来像普通 JavaScript,允许注释,并且
能完整展示解析器产出的结果。`,
  },
  {
    id: 'example-1-scalars-heading',
    join: 'block',
    en: `### 1. Scalars`,
    ru: `### 1. Скаляры`,
    zh: `### 1. 标量`,
  },
  {
    id: 'preamble-098',
    join: 'block',
    common: `\`\`\`text
name: Russia
port: 20082
\`\`\``,
  },
  {
    id: 'preamble-099',
    join: 'tight',
    common: `\`\`\`json5
{
  name: "Russia",
  port: 20082
}
\`\`\``,
  },
  {
    id: 'example-1-scalars-note',
    join: 'block',
    en: `Scalars are typed at the \`Value\` level from their lexical form
(\`Integer\`/\`Float\`/\`Bool\`/\`Null\`/\`String\`); force a literal string with
the \`::\` raw marker (e.g. \`port:: 20082\`) when a numeric-looking body
must stay a string.`,
    ru: `Скаляры типизируются на уровне \`Value\` по своей лексической форме
(\`Integer\`/\`Float\`/\`Bool\`/\`Null\`/\`String\`); принудить числоподобное
тело остаться строкой можно raw-маркером \`::\` (например,
\`port:: 20082\`).`,
    zh: `标量在 \`Value\` 层就已根据其字面形式被分类
(\`Integer\`/\`Float\`/\`Bool\`/\`Null\`/\`String\`);若数字形状的内容需要保持
字符串,可用 \`::\` raw 标记强制(例如 \`port:: 20082\`)。`,
  },
  {
    id: 'example-2-dotted-keys-heading',
    join: 'block',
    en: `### 2. Dotted keys = nested objects`,
    ru: `### 2. Точечные ключи = вложенные объекты`,
    zh: `### 2. 点分键 = 嵌套对象`,
  },
  {
    id: 'preamble-102',
    join: 'block',
    common: `\`\`\`text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
\`\`\``,
  },
  {
    id: 'preamble-103',
    join: 'tight',
    common: `\`\`\`json5
{
  server: { host: "127.0.0.1", port: 8080 },
  app: { debug: true }
}
\`\`\``,
  },
  {
    id: 'example-2-dotted-keys-note-1',
    join: 'block',
    en: `Any depth works.`,
    ru: `Любая глубина работает.`,
    zh: `深度任意。`,
  },
  {
    id: 'example-2-dotted-keys-note-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The full address is on every line.`,
    ru: `Полный адрес — на каждой строке.`,
    zh: `完整地址写在每一行上。`,
  },
  {
    id: 'example-3-nested-object-heading',
    join: 'block',
    en: `### 3. Nested object as a value`,
    ru: `### 3. Вложенный объект как значение`,
    zh: `### 3. 作为值的嵌套对象`,
  },
  {
    id: 'preamble-107',
    join: 'block',
    common: `\`\`\`text
server: {
    host: 127.0.0.1
    port: 8080
    endpoints.api: /v1
    endpoints.admin: /admin
}
\`\`\``,
  },
  {
    id: 'preamble-108',
    join: 'tight',
    common: `\`\`\`json5
{
  server: {
    host: "127.0.0.1",
    port: 8080,
    endpoints: { api: "/v1", admin: "/admin" }
  }
}
\`\`\``,
  },
  {
    id: 'example-4-array-of-scalars-heading',
    join: 'block',
    en: `### 4. Array of scalars`,
    ru: `### 4. Массив скаляров`,
    zh: `### 4. 标量数组`,
  },
  {
    id: 'preamble-110',
    join: 'block',
    common: `\`\`\`text
banned_patterns: [
    .*\\.onion:\\d+
    .*:25
]
\`\`\``,
  },
  {
    id: 'preamble-111',
    join: 'tight',
    common: `\`\`\`json5
{
  banned_patterns: [".*\\\\.onion:\\\\d+", ".*:25"]
}
\`\`\``,
  },
  {
    id: 'example-5-array-of-objects-heading',
    join: 'block',
    en: `### 5. Array of objects`,
    ru: `### 5. Массив объектов`,
    zh: `### 5. 对象数组`,
  },
  {
    id: 'preamble-113',
    join: 'block',
    common: `\`\`\`text
upstreams: [
    {
        host: a.example
        port: 1080
    }
    {
        host: b.example
        port: 1080
    }
]
\`\`\``,
  },
  {
    id: 'preamble-114',
    join: 'tight',
    common: `\`\`\`json5
{
  upstreams: [
    { host: "a.example", port: 1080 },
    { host: "b.example", port: 1080 }
  ]
}
\`\`\``,
  },
  {
    id: 'example-6-arbitrary-nesting-heading',
    join: 'block',
    en: `### 6. Arbitrary nesting`,
    ru: `### 6. Произвольная вложенность`,
    zh: `### 6. 任意层嵌套`,
  },
  {
    id: 'example-6-arbitrary-nesting-rule-1',
    join: 'block',
    en: `Every compound value spans multiple lines (single-line \`{ ... }\` / \`[ ... ]\`
with contents is not accepted — only the empty forms \`{}\` / \`[]\` are
inline).`,
    ru: `Каждое составное значение занимает несколько строк (однострочные
\`{ ... }\` / \`[ ... ]\` с содержимым не принимаются — инлайн разрешены
только пустые формы \`{}\` / \`[]\`).`,
    zh: `每个复合值都跨越多行(带内容的单行 \`{ ... }\` / \`[ ... ]\` 不被接受
——只有空形式 \`{}\` / \`[]\` 允许写作内联)。`,
  },
  {
    id: 'example-6-arbitrary-nesting-rule-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Nest as deep as needed:`,
    ru: `Вкладывайте сколько угодно:`,
    zh: `想嵌多深就嵌多深:`,
  },
  {
    id: 'example-6-arbitrary-nesting-snippet',
    join: 'block',
    common: `\`\`\`text
countries: [
    {
        name: Russia
        cities: [
            {
                name: Moscow
                buildings: [
                    {
                        name: Kremlin
                    }
                    {
                        name: Saint Basil's
                    }
                ]
            }
            {
                name: Saint Petersburg
            }
        ]
    }
    {
        name: France
    }
]
\`\`\``,
  },
  {
    id: 'example-7-literal-strings-heading',
    join: 'block',
    en: `### 7. Literal strings: \`::\``,
    ru: `### 7. Литеральные строки: \`::\``,
    zh: `### 7. 字面量字符串:\`::\``,
  },
  {
    id: 'example-7-literal-strings-rationale-1',
    join: 'block',
    en: `Some values would otherwise be parsed as compound (because they start
with \`{\` or \`[\`): regular expressions, IPv6 addresses, template
placeholders.`,
    ru: `Некоторые значения иначе были бы разобраны как compound (потому что
начинаются с \`{\` или \`[\`): регулярные выражения, IPv6-адреса,
placeholders шаблонов.`,
    zh: `某些值如果不加标记,会被当作复合值解析(因为以 \`{\` 或 \`[\` 开头):
正则、IPv6 地址、模板占位符。`,
  },
  {
    id: 'example-7-literal-strings-rationale-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The double-colon \`::\` flags them as "raw string, do not
parse further."`,
    ru: `Двойное двоеточие \`::\` помечает их как «сырая
строка, не разбирать дальше».`,
    zh: `双冒号 \`::\` 把它们标记为「原样字符串,
不要继续解析」。`,
  },
  {
    id: 'preamble-122',
    join: 'block',
    common: `\`\`\`text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl

hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
\`\`\``,
  },
  {
    id: 'preamble-123',
    join: 'tight',
    common: `\`\`\`json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
\`\`\``,
  },
  {
    id: 'example-7-literal-strings-note-1',
    join: 'block',
    en: `For pairs the marker sits between key and value; for array items it
stands at the start of the line.`,
    ru: `Для пар маркер стоит между ключом и значением; для элементов массива —
в начале строки.`,
    zh: `对于键值对,该标记位于键和值之间;对于数组元素,它位于行首。`,
  },
  {
    id: 'example-7-literal-strings-note-2',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `**Serialization emits \`::\`
automatically** when a string value begins with \`{\` or \`[\`, so
round-tripping regexes and IPv6 addresses just works.`,
    ru: `**Сериализация эмитит \`::\` автоматически**, когда
строковое значение начинается с \`{\` или \`[\`, так что round-trip
regex-ов и IPv6-адресов просто работает.`,
    zh: `**当字符串值以 \`{\` 或 \`[\` 开头时,序列化会自动输出 \`::\`**,所以
正则与 IPv6 地址的 round-trip 自然工作。`,
  },
  {
    id: 'example-8-comments-heading',
    join: 'block',
    en: `### 8. Comments`,
    ru: `### 8. Комментарии`,
    zh: `### 8. 注释`,
  },
  {
    id: 'preamble-127',
    join: 'block',
    common: `\`\`\`text
## top-level comment
port: 8080

items: [
    ## this comment does not break the array
    a
    b
]
\`\`\``,
  },
  {
    id: 'example-8-comments-explanation-1',
    join: 'block',
    en: `Comments are full lines starting with \`#\`.`,
    ru: `Комментарии — целые строки, начинающиеся с \`#\`.`,
    zh: `注释为整行,以 \`#\` 开头。`,
  },
  {
    id: 'example-8-comments-explanation-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Inline comments are not
supported — they get confused with the value too easily.`,
    ru: `Inline-комментарии
не поддерживаются — их слишком легко спутать со значением.`,
    zh: `不支持行内注释——太容易与值混淆。`,
  },
  {
    id: 'example-9-multiline-strings-heading',
    join: 'block',
    en: `### 9. Multi-line strings: \`( ... )\` and \`(( ... ))\``,
    ru: `### 9. Многострочные строки: \`( ... )\` и \`(( ... ))\``,
    zh: `### 9. 多行字符串:\`( ... )\` 与 \`(( ... ))\``,
  },
  {
    id: 'example-9-multiline-strings-intro-1',
    join: 'block',
    en: `Values that span multiple lines go inside parentheses.`,
    ru: `Значения, занимающие несколько строк, заключаются в круглые скобки.`,
    zh: `跨越多行的值放在圆括号里。`,
  },
  {
    id: 'example-9-multiline-strings-intro-2',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `The opening and
closing lines are NOT part of the value.`,
    ru: `Открывающая и закрывающая строки НЕ входят в значение.`,
    zh: `开起行与关闭行**不**属于值。`,
  },
  {
    id: 'example-9-parens-form',
    join: 'block',
    en: `\`(\` ... \`)\` — common leading whitespace is stripped and, as of 0.7,
trailing whitespace is stripped from each line, so you can indent
the block to match its surroundings without contaminating the content:`,
    ru: `\`(\` ... \`)\` — общий ведущий отступ срезается, а с 0.7 с каждой строки
срезается ещё и замыкающий пробельный хвост, так что можно выравнивать
блок по окружающему коду, не засоряя содержимое:`,
    zh: `\`(\` ... \`)\` —— 剥除公共前导缩进,且从 0.7 起还会剥除每行末尾的空白,
所以你可以按照周围代码的缩进
书写,而不会污染内容:`,
  },
  {
    id: 'preamble-134',
    join: 'block',
    common: `\`\`\`text
body: (
    {
      "qwe": 1
    }
)
\`\`\``,
  },
  {
    id: 'preamble-135',
    join: 'tight',
    common: `\`\`\`json5
{ body: "{\\n  \\"qwe\\": 1\\n}" }
\`\`\``,
  },
  {
    id: 'example-9-double-parens-form',
    join: 'block',
    en: `\`((\` ... \`))\` — verbatim: every character between the markers ends up in
the value, including leading whitespace:`,
    ru: `\`((\` ... \`))\` — побайтово: каждый символ между маркерами попадает в
значение, включая ведущие пробелы:`,
    zh: `\`((\` ... \`))\` —— 逐字节保留:开始与结束标记之间的每一个字符都
进入值,包括前导空白:`,
  },
  {
    id: 'preamble-137',
    join: 'block',
    common: `\`\`\`text
sig: ((
  -----BEGIN-----
  QUJDRA==
  -----END-----
))
\`\`\``,
  },
  {
    id: 'preamble-138',
    join: 'tight',
    common: `\`\`\`json5
{ sig: "  -----BEGIN-----\\n  QUJDRA==\\n  -----END-----" }
\`\`\``,
  },
  {
    id: 'example-9-block-content-rule-1',
    join: 'block',
    en: `Inside a block, \`{\` / \`[\` / \`#\` are just content — **no compound parsing,
no comment skipping**.`,
    ru: `Внутри блока \`{\` / \`[\` / \`#\` — просто содержимое, **никакого разбора
compound, никакого пропуска комментариев**.`,
    zh: `在块内,\`{\` / \`[\` / \`#\` 只是内容——**不做复合解析,不跳过注释**。`,
  },
  {
    id: 'example-9-block-content-rule-2',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `The only special sequence is the terminator on
its own line.`,
    ru: `Единственная особая
последовательность — терминатор на отдельной строке.`,
    zh: `唯一的特殊序列就是独占一行的终止符。`,
  },
  {
    id: 'example-9-empty-inline-form',
    join: 'block',
    en: `Empty inline form: \`key: ()\` or \`key: (())\` — both yield the empty
string (same as \`key:\`).`,
    ru: `Пустая инлайн-форма: \`key: ()\` или \`key: (())\` — обе дают пустую
строку (то же, что \`key:\`).`,
    zh: `空的内联形式:\`key: ()\` 或 \`key: (())\`——都会产生空字符串(等同
于 \`key:\`)。`,
  },
  {
    id: 'example-9-serialization-rules-1',
    join: 'block',
    en: `Serialization: a string is emitted on a single line only when it has
no \`\\n\`, no leading/trailing whitespace, and no control byte other
than \`TAB\`.`,
    ru: `Сериализация: строка выходит в одну строку, только если в ней нет
\`\\n\`, нет ведущих/замыкающих пробельных символов и нет управляющих
байтов кроме \`TAB\`.`,
    zh: `序列化:字符串只有在不含 \`\\n\`、没有首尾空白、且不含除 \`TAB\`
以外的控制字节时才会以单行形式输出。`,
  },
  {
    id: 'example-9-serialization-rules-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Anything else — including a plain string with a stray edge
space — takes a multi-line form: verbatim \`(( ... ))\` when the content
has edge whitespace that stripped would alter, stripped \`( ... )\`
otherwise; whichever the
writer picks, the round-trip is byte-for-byte lossless.`,
    ru: `Всё остальное — включая обычную строку со
случайным пробелом на краю — уходит в многострочную форму: дословную
\`(( ... ))\`, если у содержимого есть крайние пробельные символы,
которые снятие отступа исказит, иначе форму со снятым отступом
\`( ... )\`; какую бы форму writer ни выбрал, round-trip
байт-в-байт без потерь.`,
    zh: `其他情况——哪怕只是普通字符串
边缘多了一个空格——都会采用多行形式:当内容含有会被去缩进破坏的
边缘空白时采用逐字形式 \`(( ... ))\`,否则采用去缩进形式
\`( ... )\`;无论 writer 选择哪种形式,
round-trip 都是字节级无损的。`,
  },
  {
    id: 'example-9-which-form-1',
    join: 'block',
    en: `Which block form you get depends on *where* the whitespace is.`,
    ru: `Какая именно блочная форма — зависит от того, *где* пробелы.`,
    zh: `具体采用哪种块形式,取决于空白出现在**哪一侧**。`,
  },
  {
    id: 'example-9-which-form-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `As of
0.7 the stripped form strips trailing whitespace from every content
line (§ 5.6), so a trailing space forces the verbatim form, which
preserves it byte-for-byte:`,
    ru: `С 0.7
форма со снятым отступом срезает замыкающий пробельный хвост с каждой
строки содержимого (§ 5.6), так что замыкающий пробел вынуждает взять
дословную форму, которая сохраняет его байт-в-байт:`,
    zh: `从 0.7 起,去缩进
形式会剥除每行内容末尾的空白(§ 5.6),因此末尾空格必须使用逐字
形式,由它逐字节保留:`,
  },
  {
    id: 'preamble-146',
    join: 'block',
    common: `\`\`\`json5
{ password: "hunter2 " }
\`\`\``,
  },
  {
    id: 'preamble-147',
    join: 'tight',
    common: `\`\`\`text
password: ((
hunter2 
))
\`\`\``,
  },
  {
    id: 'example-9-leading-whitespace-note',
    join: 'block',
    en: `Leading whitespace — and, since 0.7, trailing whitespace — is what
forces the verbatim form: stripping would eat the leading indent:`,
    ru: `Ведущие пробелы — а с 0.7 ещё и замыкающие — вынуждают взять дословную
форму: снятие отступа съело бы ведущий отступ:`,
    zh: `前导空白——从 0.7 起还包括末尾空白——必须使用逐字形式:去缩进会
把前导缩进吃掉:`,
  },
  {
    id: 'preamble-149',
    join: 'block',
    common: `\`\`\`json5
{ indent: "  padded" }
\`\`\``,
  },
  {
    id: 'preamble-150',
    join: 'tight',
    common: `\`\`\`text
indent: ((
  padded
))
\`\`\``,
  },
  {
    id: 'example-9-roundtrip-note',
    join: 'block',
    en: `Either way, reading it back gives you the original bytes.`,
    ru: `В обоих случаях при обратном чтении получите исходные байты.`,
    zh: `两种情况读回来都能得到原始字节。`,
  },
  {
    id: 'example-9-limitation',
    join: 'block',
    en: `Limitation: a body containing a line whose trimmed content is exactly
\`))\` cannot use the verbatim form. It falls back to stripped instead —
unless the body also has a sole-\`)\` line, a whitespace-only line, a
line with trailing whitespace, or
every line indented (nothing to anchor the dedent at zero), in which
case no form can hold it and serialization returns an error rather
than emit a document that fails to round-trip.`,
    ru: `Ограничение: тело, где есть строка с trimmed-содержимым ровно \`))\`,
не может использовать дословную форму. Вместо неё берётся форма со
снятым отступом — если только в теле нет ещё и строки ровно \`)\`,
строки из одних пробелов, строки с замыкающим пробельным хвостом,
или все строки с отступом (не от чего
отсчитывать выравнивание к нулю) — тогда ни одна форма не подходит, и
сериализация возвращает ошибку вместо документа, который не
восстановится обратно.`,
    zh: `限制:若正文中有一行 trim 后恰好是 \`))\`,则不能使用逐字形式,会
改用去缩进形式——除非正文中还含有恰好是 \`)\` 的一行、只含空白的
一行、末尾带空白的一行,或所有行都有缩进(没有可用于将去缩进基准定为零的行),此时
没有任何形式能容纳该正文,序列化会返回错误,而不是输出一个无法
还原的文档。`,
  },
  {
    id: 'example-10-empty-compounds-heading',
    join: 'block',
    en: `### 10. Empty compounds`,
    ru: `### 10. Пустые compound-ы`,
    zh: `### 10. 空复合值`,
  },
  {
    id: 'example-10-empty-compounds-snippet',
    join: 'block',
    common: `\`\`\`text
meta: {}
tags: []
\`\`\``,
  },
  {
    id: 'example-10-empty-compounds-note-1',
    join: 'block',
    en: `Inline empty is allowed.`,
    ru: `Inline-пустой разрешён.`,
    zh: `允许内联空。`,
  },
  {
    id: 'example-10-empty-compounds-note-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Anything with contents must span multiple
lines, and the closing \`}\` / \`]\` must sit on its own line.`,
    ru: `Всё с содержимым обязано занимать несколько
строк, и закрывающий \`}\` / \`]\` обязан стоять на отдельной строке.`,
    zh: `带内容的值必须跨越多行,且闭合的 \`}\` / \`]\` 必须独占
一行。`,
  },
  {
    id: 'example-11-enums-heading',
    join: 'block',
    en: `### 11. Enums`,
    ru: `### 11. Enum-ы`,
    zh: `### 11. 枚举`,
  },
  {
    id: 'example-11-enums-intro',
    join: 'block',
    en: `Ktav uses serde's default *externally tagged* enum representation.`,
    ru: `Ktav использует дефолтное *externally tagged* представление enum-ов
serde.`,
    zh: `Ktav 使用 serde 默认的 *externally tagged* 枚举表示形式。`,
  },
  {
    id: 'preamble-159',
    join: 'block',
    common: `\`\`\`rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode { Fast, Slow }

#[derive(Serialize, Deserialize)]
enum Action {
    Log(String),
    Count(u32),
}
\`\`\``,
  },
  {
    id: 'preamble-160',
    join: 'block',
    common: `\`\`\`text
## unit variant — just the name
mode: fast

## newtype variant — single-entry object
action: {
    Log: hello
}
\`\`\``,
  },
];
