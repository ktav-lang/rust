// README units: values section.
export const values = [
  {
    id: 'values-heading',
    join: 'block',
    en: `## Values and special tokens`,
    ru: `## Значения и специальные токены`,
    zh: `## 值与特殊记号`,
  },
  {
    id: 'strings-heading',
    join: 'block',
    en: `### Strings`,
    ru: `### Строки`,
    zh: `### 字符串`,
  },
  {
    id: 'strings-intro-1',
    join: 'block',
    en: `Default for any scalar.`,
    ru: `Дефолт для любого скаляра.`,
    zh: `任何标量的默认类型。`,
  },
  {
    id: 'strings-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Stored internally as \`Value::String\`.`,
    ru: `Внутри хранятся как \`Value::String\`.`,
    zh: `在内部以 \`Value::String\` 存放。`,
  },
  {
    id: 'strings-intro-3',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `The value
is whatever follows \`:\` after trimming.`,
    ru: `Значение — это всё, что идёт после \`:\`, после обрезки пробелов.`,
    zh: `值就是 \`:\` 之后
经过空白修剪的内容。`,
  },
  {
    id: 'strings-snippet',
    join: 'block',
    en: `\`\`\`text
name: Russia
path: /etc/hosts
greeting: hello world
## \`::\` forces a literal string
pattern:: [a-z]+
\`\`\``,
    ru: `\`\`\`text
name: Russia
path: /etc/hosts
greeting: hello world
## \`::\` принудительно задаёт литеральную строку
pattern:: [a-z]+
\`\`\``,
    zh: `\`\`\`text
name: Russia
path: /etc/hosts
greeting: hello world
## \`::\` 强制将值解释为字面量字符串
pattern:: [a-z]+
\`\`\``,
  },
  {
    id: 'preamble-031',
    join: 'block',
    en: `### Numbers`,
    ru: `### Числа`,
    zh: `### 数字`,
  },
  {
    id: 'numbers-typing',
    join: 'block',
    en: `Numbers are written bare (no quotes) and typed by lexical form: a bare
integer body parses to \`Value::Integer\`, a bare decimal to
\`Value::Float\`. Each stores a *normalized* payload, not the original
spelling: \`Integer\` holds the canonical base-10 form (no underscores,
no leading zeros/\`+\`), \`Float\` holds the shortest decimal form that
round-trips the exact \`f64\` bits — \`+1_000\` becomes \`Integer("1000")\`,
\`1.0e+2\` becomes \`Float("100.0")\`. \`Value\`-level \`Integer\` covers the
i64 range; a native Rust integer type wider than i64 (\`u64\`, \`i128\`,
\`u128\`) that doesn't fit is stored as \`Value::String\` instead when
going through \`ser::to_value\`, matching what parsing that same decimal
text back would produce. serde deserializes numbers into the target
Rust type (\`u16\`, \`i64\`, \`i128\`, \`f64\`, …) via direct parsing, and
formats them with the same canonicalization on serialization; a value
forced to a string with \`::\` is still accepted.`,
    ru: `Числа пишутся без кавычек и типизируются по лексической форме: голое
целое тело разбирается в \`Value::Integer\`, голая десятичная запись —
в \`Value::Float\`. Каждый хранит *нормализованный* payload, а не
исходное написание: \`Integer\` хранит канонический десятичный вид (без
подчёркиваний, без ведущих нулей/\`+\`), \`Float\` — кратчайшую десятичную
форму, восстанавливающую точные биты \`f64\` — \`+1_000\` становится
\`Integer("1000")\`, \`1.0e+2\` становится \`Float("100.0")\`. \`Integer\` на
уровне \`Value\` покрывает диапазон i64; более широкий native Rust
целочисленный тип (\`u64\`, \`i128\`, \`u128\`), не помещающийся в i64, при
проходе через \`ser::to_value\` сохраняется как \`Value::String\` — точно
так же, как если бы этот же десятичный текст был разобран парсером.
serde десериализует числа в целевой Rust-тип (\`u16\`, \`i64\`, \`i128\`,
\`f64\`, …) прямым разбором и форматирует их с той же канонизацией при
сериализации.`,
    zh: `数字不加引号,并根据字面形式分类:裸整数 body 解析为
\`Value::Integer\`,裸小数解析为 \`Value::Float\`。两者存放的都是
*规范化*后的 payload,而非原始写法:\`Integer\` 保存规范十进制形式
(无下划线、无前导零/\`+\`),\`Float\` 保存能还原精确 \`f64\` 位模式的最短
十进制形式——\`+1_000\` 变为 \`Integer("1000")\`,\`1.0e+2\` 变为
\`Float("100.0")\`。\`Value\` 层的 \`Integer\` 覆盖 i64 范围;比 i64 更宽的
native Rust 整数类型(\`u64\`、\`i128\`、\`u128\`)若超出该范围,经过
\`ser::to_value\` 时会存为 \`Value::String\`——效果与直接解析同一段十进制
文本完全一致。serde 在反序列化时直接将数字解析为目标 Rust 类型
(\`u16\`、\`i64\`、\`i128\`、\`f64\`……),并在序列化时用相同的规范化格式化。`,
  },
  {
    id: 'numbers-snippet',
    join: 'block',
    common: `\`\`\`text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
\`\`\``,
  },
  {
    id: 'numbers-lossy-note',
    join: 'block',
    en: `A value like \`port: abc\` parses fine *at the Ktav level* (string
\`"abc"\`), but \`serde::deserialize\` into \`u16\` will return a clear
\`ParseError\`.`,
    ru: `Значение вида \`port: abc\` парсится нормально *на уровне Ktav* (строка
\`"abc"\`), но \`serde::deserialize\` в \`u16\` вернёт понятный
\`ParseError\`.`,
    zh: `像 \`port: abc\` 这样的值在 *Ktav 层*可以正常解析(即字符串 \`"abc"\`),
但 \`serde::deserialize\` 到 \`u16\` 时会返回一个清晰的 \`ParseError\`。`,
  },
  {
    id: 'booleans-heading',
    join: 'block',
    en: `### Booleans: \`true\` / \`false\``,
    ru: `### Булевы: \`true\` / \`false\``,
    zh: `### 布尔:\`true\` / \`false\``,
  },
  {
    id: 'booleans-strictness-1',
    join: 'block',
    en: `Strict lowercase.`,
    ru: `Строго нижний регистр.`,
    zh: `严格小写。`,
  },
  {
    id: 'booleans-strictness-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Anything else is a string.`,
    ru: `Всё остальное — строка.`,
    zh: `其它写法都是字符串。`,
  },
  {
    id: 'booleans-snippet',
    join: 'block',
    common: `\`\`\`text
## Value::Bool(true)
on: true
## Value::Bool(false)
off: false
## Value::String("True")
capitalized: True
## Value::String("FALSE")
yelling:    FALSE
## Value::String("true")
literal:: true
\`\`\``,
  },
  {
    id: 'null-heading',
    join: 'block',
    en: `### Null: \`null\``,
    ru: `### Null: \`null\``,
    zh: `### Null:\`null\``,
  },
  {
    id: 'null-mapping-1',
    join: 'block',
    en: `Strict lowercase.`,
    ru: `Строго нижний регистр.`,
    zh: `严格小写。`,
  },
  {
    id: 'null-mapping-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Matches \`Option::None\` on the Rust side, as well as
\`()\` for unit.`,
    ru: `На стороне Rust соответствует \`Option::None\`,
а также \`()\` для unit.`,
    zh: `对应 Rust 侧的 \`Option::None\`,以及 unit 类型 \`()\`。`,
  },
  {
    id: 'null-snippet',
    join: 'block',
    common: `\`\`\`text
## Value::Null
label: null
## Value::String("Null")
capitalized: Null
## Value::String("null")
literal:: null
\`\`\``,
  },
  {
    id: 'null-serialization-1',
    join: 'block',
    en: `When serializing, \`Option::None\` is emitted as \`null\`.`,
    ru: `При сериализации \`Option::None\` эмитится как \`null\`.`,
    zh: `序列化时,\`Option::None\` 会输出为 \`null\`。`,
  },
  {
    id: 'null-serialization-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Suppress with
\`#[serde(skip_serializing_if = "Option::is_none")]\` if you prefer the
field absent.`,
    ru: `Подавить можно
через \`#[serde(skip_serializing_if = "Option::is_none")]\`, если вы
предпочитаете, чтобы поля не было вовсе.`,
    zh: `若你希望该字段干脆缺省,
可以加上 \`#[serde(skip_serializing_if = "Option::is_none")]\`。`,
  },
  {
    id: 'empty-compounds-heading',
    join: 'block',
    en: `### Empty object / empty array`,
    ru: `### Пустой объект / пустой массив`,
    zh: `### 空对象 / 空数组`,
  },
  {
    id: 'empty-compounds-only-inline',
    join: 'block',
    en: `The **only** inline compound values allowed — nothing to separate, no
commas needed.`,
    ru: `**Единственные** allowed inline compound-значения — разделять нечего,
запятые не нужны.`,
    zh: `**唯一**允许的内联复合值——没什么要分隔,不需要逗号。`,
  },
  {
    id: 'empty-compounds-snippet',
    join: 'block',
    en: `\`\`\`text
## empty object
meta: {}
## empty array
tags: []
\`\`\``,
    ru: `\`\`\`text
## пустой объект
meta: {}
## пустой массив
tags: []
\`\`\``,
    zh: `\`\`\`text
## 空对象
meta: {}
## 空数组
tags: []
\`\`\``,
  },
  {
    id: 'literal-strings-heading',
    join: 'block',
    en: `### Keyword-like strings need \`::\``,
    ru: `### Ключеподобные строки требуют \`::\``,
    zh: `### 与关键字同形的字符串需要 \`::\``,
  },
  {
    id: 'literal-strings-rationale-1',
    join: 'block',
    en: `If a string's content happens to equal a keyword (\`true\`, \`false\`,
\`null\`) or begin with \`{\` or \`[\`, the **serializer emits \`::\`
automatically** so the round-trip is lossless.`,
    ru: `Если содержимое строки совпадает с ключевым словом (\`true\`, \`false\`,
\`null\`) или начинается с \`{\` или \`[\`, **сериализатор автоматически
эмитит \`::\`**, чтобы round-trip был без потерь.`,
    zh: `若字符串内容恰好等于关键字(\`true\`、\`false\`、\`null\`),或以 \`{\`、
\`[\` 开头,**序列化器会自动输出 \`::\`**,以保证 round-trip 无损。`,
  },
  {
    id: 'literal-strings-rationale-2',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `On the writing side you
do the same:`,
    ru: `На стороне записи
поступайте так же:`,
    zh: `写入端请用同样的方式:`,
  },
  {
    id: 'literal-strings-snippet',
    join: 'block',
    en: `\`\`\`text
## the string "true", not a bool
flag:: true
## the string "null", not a null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
\`\`\``,
    ru: `\`\`\`text
## строка "true", а не булево
flag:: true
## строка "null", а не Null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
\`\`\``,
    zh: `\`\`\`text
## 字符串 "true",而非 Bool
flag:: true
## 字符串 "null",而非 Null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
\`\`\``,
  },
];
