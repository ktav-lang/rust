// Source of truth for README.md, README.ru.md, README.zh.md.
//
// GENERATED FROM THIS FILE — never edit those artifacts by hand. Change
// the units here, then run
//   node tools/docs-gen/src/cli.mjs
// and verify with `--check`, which is what CI runs.
//
// One unit is one granular meaning: a heading, a sentence, a bullet, a
// table row. Its `join` records how it attaches to the unit before it —
// a blank line by default, a newline for `tight`, a space for `flow`,
// nothing for `none`. The map form exists because the three languages
// wrap their lines in different places. A unit is either a full
// translation set or a single `common` block for text that is identical
// in every language (code, tables, version headings).

export default [
  {
    id: 'intro-title',
    common: `# Ktav (כְּתָב)`,
  },
  {
    id: 'badge-row',
    join: 'block',
    common: `[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)`,
  },
  {
    id: 'tagline-quote-1',
    join: 'block',
    en: `> A plain configuration format.`,
    ru: `> Простой формат конфигурации.`,
    zh: `> 一种朴素的配置格式。`,
  },
  {
    id: 'tagline-quote-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `JSON5-shaped, but without the quotes,
> without the commas, with dotted keys for nesting.`,
    ru: `Формы JSON5, но без кавычек, без
> запятых, с точечными ключами для вложенности.`,
    zh: `形态上接近 JSON5,但不带引号、不用逗号,
> 以点分键表达嵌套。`,
  },
  {
    id: 'tagline-quote-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Native \`serde\`
> integration.`,
    ru: `Нативная интеграция
> с \`serde\`.`,
    zh: `原生 \`serde\` 集成。`,
  },
  {
    id: 'languages-row',
    join: 'block',
    en: `**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)`,
    ru: `**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)`,
    zh: `**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**`,
  },
  {
    id: 'playground-row',
    join: 'block',
    en: `**Playground:** convert JSON / YAML / TOML / INI ⇄ Ktav in your browser at **[ktav-lang.github.io](https://ktav-lang.github.io/)**.`,
    ru: `**Песочница:** конвертация JSON / YAML / TOML / INI ⇄ Ktav прямо в браузере — **[ktav-lang.github.io](https://ktav-lang.github.io/)**.`,
    zh: `**演练场：** 在浏览器中互转 JSON / YAML / TOML / INI ⇄ Ktav — **[ktav-lang.github.io](https://ktav-lang.github.io/)**。`,
  },
  {
    id: 'spec-pointer',
    join: 'block',
    en: `**Specification:** this crate implements **Ktav 0.7.1**, the version
named by \`[package.metadata.ktav] spec-version\` in \`Cargo.toml\`. The
format is versioned and maintained independently of this crate — the two
numbers move apart on purpose, since a crate release that changes no
format behaviour leaves \`spec-version\` where it was. See
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec) for the formal
document, and [\`CHANGELOG.md\`](CHANGELOG.md) for this crate's history.`,
    ru: `**Спецификация:** этот crate реализует **Ktav 0.7.1** — версию,
указанную в \`[package.metadata.ktav] spec-version\` в \`Cargo.toml\`.
Формат версионируется и поддерживается независимо от crate-а, и два
номера намеренно расходятся: выпуск crate-а, не меняющий поведение
формата, оставляет \`spec-version\` на месте. См.
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec) для канонического
документа и [\`CHANGELOG.ru.md\`](CHANGELOG.ru.md) — историю crate-а.`,
    zh: `**规范:** 本 crate 实现 **Ktav 0.7.1**,即 \`Cargo.toml\` 中
\`[package.metadata.ktav] spec-version\` 所指定的版本。格式与 crate 彼此
独立地版本化与维护,两个号码会有意分开——不改变格式行为的 crate 发布会
让 \`spec-version\` 保持原样。规范正文见
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec),crate 的历史见
[\`CHANGELOG.zh.md\`](CHANGELOG.zh.md)。`,
  },
  {
    id: 'intro-rule',
    join: 'block',
    common: `---`,
  },
  {
    id: 'name-heading',
    join: 'block',
    en: `## Name`,
    ru: `## Название`,
    zh: `## 名称`,
  },
  {
    id: 'name-meaning-1',
    join: 'block',
    en: `*Ktav* (Hebrew: **כְּתָב**) means "writing, that which is written" — a
thing recorded in a form fixed enough that its meaning does not depend
on who passes it along.`,
    ru: `*Ktav* (иврит: **כְּתָב**) означает «письмо, то, что записано» —
нечто зафиксированное в форме, достаточно устойчивой, чтобы смысл не
зависел от того, кто это передаёт дальше.`,
    zh: `*Ktav*(希伯来语:**כְּתָב**)意为「书写、被写下来的东西」——一种
以足够稳固的形态被记录下来的东西,其意义不依赖于传递者。`,
  },
  {
    id: 'name-meaning-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The name fits literally: a config file *is*
ktav on disk, and the library reads it and hands you back a live
structure without making anything up along the way.`,
    ru: `Название подходит буквально:
конфиг-файл *и есть* ktav на диске, а библиотека его читает и отдаёт
живую структуру, ничего не выдумывая по дороге.`,
    zh: `名字用得
很字面:一份配置文件*就是*磁盘上的 ktav,而本库把它读进来,原样
交出一个活生生的结构,不会在途中擅自脑补。`,
  },
  {
    id: 'motto-heading',
    join: 'block',
    en: `## Motto`,
    ru: `## Девиз`,
    zh: `## 格言`,
  },
  {
    id: 'motto-quote-1',
    join: 'block',
    en: `> **Be the config's friend, not its examiner.`,
    ru: `> **Будь другом конфига, а не его экзаменатором.`,
    zh: `> **做配置的朋友,别做它的考官。`,
  },
  {
    id: 'motto-quote-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The config isn't perfect —
> but it's the best one.**`,
    ru: `Конфиг неидеален —
> но он лучший из возможных.**`,
    zh: `配置并不完美——但已是最好的那一份。**`,
  },
  {
    id: 'motto-locality-1',
    join: 'block',
    en: `Every rule is local.`,
    ru: `Каждое правило локально.`,
    zh: `每条规则都是局部的。`,
  },
  {
    id: 'motto-locality-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Every line either stands on its own or depends only
on visible brackets.`,
    ru: `Каждая строка либо стоит сама по себе, либо
зависит только от видимых скобок.`,
    zh: `每一行要么独立成立,要么只依赖于可见的括号。`,
  },
  {
    id: 'motto-locality-3',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `No indentation pitfalls, no forgotten quotes, no
trailing-comma arithmetic.`,
    ru: `Никаких ловушек с отступами, никаких
забытых кавычек, никакой арифметики замыкающих запятых.`,
    zh: `没有缩进陷阱,没有忘记的引号,没有尾随逗号的算术。`,
  },
  {
    id: 'rules-heading',
    join: 'block',
    en: `## The rules`,
    ru: `## Правила`,
    zh: `## 规则`,
  },
  {
    id: 'rules-intro-1',
    join: 'block',
    en: `A Ktav document is an implicit top-level object.`,
    ru: `Документ Ktav — это неявный объект верхнего уровня.`,
    zh: `Ktav 文档是一个隐式的顶层对象。`,
  },
  {
    id: 'rules-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Inside any object you
have pairs; inside any array you have items.`,
    ru: `Внутри любого
объекта — пары; внутри любого массива — элементы.`,
    zh: `任何对象里是键值对,任何数组里是
元素。`,
  },
  {
    id: 'rules-reference-card',
    join: 'block',
    common: `\`\`\`text
## comment              — any line starting with '##'
key: value             — scalar pair; key may be a dotted path (a.b.c)
key:: value            — scalar pair; value is ALWAYS a literal string
key: { ... }           — multi-line object; \`}\` closes on its own line
key: [ ... ]           — multi-line array; \`]\` closes on its own line
key: {}   /   key: []  — empty compound, inline
key: ( ... )           — multi-line string; common indent stripped
key: (( ... ))         — multi-line string; verbatim (no stripping)
:: value               — inside an array: literal-string item
\`\`\``,
  },
  {
    id: 'rules-summary-1',
    join: 'block',
    en: `That's the whole language.`,
    ru: `Это весь язык.`,
    zh: `整个语言就这些。`,
  },
  {
    id: 'rules-summary-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `No commas, no quotes, no escape inside the
value itself — the only "escape" is the \`::\` marker, and it lives in the
separator (for pairs) or as a line prefix (for array items).`,
    ru: `Никаких запятых, никаких кавычек, никаких escape
внутри самого значения — единственный «escape» это маркер \`::\`, и он
живёт в разделителе (для пар) или в префиксе строки (для элементов
массива).`,
    zh: `没有逗号、没有引号、没有值内部的转义——唯一的
「转义」是 \`::\` 标记,它出现在分隔符里(用于键值对)或行首前缀里
(用于数组元素)。`,
  },
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
  {
    id: 'compounds-multiline-heading',
    join: 'block',
    en: `## Compound values are multi-line`,
    ru: `## Составные значения — многострочные`,
    zh: `## 复合值是多行的`,
  },
  {
    id: 'compounds-multiline-rule-1',
    join: 'block',
    en: `Non-empty \`{ ... }\` / \`[ ... ]\` **must** span multiple lines, with the
closing bracket on its own line.`,
    ru: `Непустые \`{ ... }\` / \`[ ... ]\` **обязаны** занимать несколько строк,
с закрывающей скобкой на отдельной строке.`,
    zh: `非空的 \`{ ... }\` / \`[ ... ]\` **必须**跨越多行,闭合括号独占一行。`,
  },
  {
    id: 'compounds-multiline-rule-2',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `\`x: { a: 1 }\` and \`x: [1, 2, 3]\` are
rejected with a clear error — Ktav has no comma-separation rules and
no escape mechanism for them.`,
    ru: `\`x: { a: 1 }\` и
\`x: [1, 2, 3]\` отклоняются с ясной ошибкой — в Ktav нет правил
разделения запятыми и нет механизма escape для них.`,
    zh: `\`x: { a: 1 }\` 与 \`x: [1, 2, 3]\` 会被以清晰的错误拒绝——Ktav 没有
逗号分隔的规则,也没有针对它们的转义机制。`,
  },
  {
    id: 'preamble-055',
    join: 'block',
    common: `\`\`\`text
## rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]

## accepted — multi-line form
server: {
    host: 127.0.0.1
    port: 8080
}

tags: [
    primary
    eu
    prod
]
\`\`\``,
  },
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
  {
    id: 'rust-usage-heading',
    join: 'block',
    en: `## Using it from Rust`,
    ru: `## Использование из Rust`,
    zh: `## 在 Rust 中使用`,
  },
  {
    id: 'rust-usage-serde-intro-1',
    join: 'block',
    en: `Ktav is serde-native.`,
    ru: `Ktav — serde-нативный.`,
    zh: `Ktav 原生支持 serde。`,
  },
  {
    id: 'rust-usage-serde-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Any type implementing \`Serialize\` / \`Deserialize\`
(including \`#[derive]\`-generated ones) round-trips through Ktav out of
the box.`,
    ru: `Любой тип, реализующий \`Serialize\` /
\`Deserialize\` (включая сгенерированные через \`#[derive]\`),
round-trip-ится через Ktav из коробки.`,
    zh: `任何实现了 \`Serialize\` / \`Deserialize\` 的类型
(包括 \`#[derive]\` 生成的)都可以开箱即用地通过 Ktav 完成
round-trip。`,
  },
  {
    id: 'parse-heading',
    join: 'block',
    en: `### Parse — decode straight into a typed struct`,
    ru: `### Парсинг — декод сразу в типизированную структуру`,
    zh: `### 解析 —— 直接解码到类型化结构体`,
  },
  {
    id: 'preamble-060',
    join: 'block',
    common: `\`\`\`rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Db { host: String, timeout: u32 }

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port:    u16,
    ratio:   f64,
    tls:     bool,
    tags:    Vec<String>,
    db:      Db,
}

const SRC: &str = "\\
service: web
port: 8080
ratio: 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout: 30
";

let cfg: Config = ktav::from_str(SRC)?;
println!("port={} db.host={}", cfg.port, cfg.db.host);
\`\`\``,
  },
  {
    id: 'walk-heading',
    join: 'block',
    en: `### Walk — match on the dynamic \`Value\` enum`,
    ru: `### Обход — match по динамическому enum \`Value\``,
    zh: `### 遍历 —— 在动态 \`Value\` 枚举上 match`,
  },
  {
    id: 'preamble-062',
    join: 'block',
    en: `\`\`\`rust
use ktav::value::Value;

let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("top is always an object") };

for (k, v) in top {
    let kind = match v {
        Value::Null         => "null".into(),
        Value::Bool(b)      => format!("bool={b}"),
        Value::Integer(s)   => format!("int={s}"),
        Value::Float(s)     => format!("float={s}"),
        Value::String(s)    => format!("str={s:?}"),
        Value::Array(a)     => format!("array({})", a.len()),
        Value::Object(o)    => format!("object({})", o.len()),
    };
    println!("{k} -> {kind}");
}
\`\`\``,
    ru: `\`\`\`rust
use ktav::value::Value;

let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("top — всегда object") };

for (k, v) in top {
    let kind = match v {
        Value::Null         => "null".into(),
        Value::Bool(b)      => format!("bool={b}"),
        Value::Integer(s)   => format!("int={s}"),
        Value::Float(s)     => format!("float={s}"),
        Value::String(s)    => format!("str={s:?}"),
        Value::Array(a)     => format!("array({})", a.len()),
        Value::Object(o)    => format!("object({})", o.len()),
    };
    println!("{k} -> {kind}");
}
\`\`\``,
    zh: `\`\`\`rust
use ktav::value::Value;

let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("顶层始终是 object") };

for (k, v) in top {
    let kind = match v {
        Value::Null         => "null".into(),
        Value::Bool(b)      => format!("bool={b}"),
        Value::Integer(s)   => format!("int={s}"),
        Value::Float(s)     => format!("float={s}"),
        Value::String(s)    => format!("str={s:?}"),
        Value::Array(a)     => format!("array({})", a.len()),
        Value::Object(o)    => format!("object({})", o.len()),
    };
    println!("{k} -> {kind}");
}
\`\`\``,
  },
  {
    id: 'build-render-heading',
    join: 'block',
    en: `### Build & render — construct a document in code`,
    ru: `### Билд + рендер — собираем документ в коде`,
    zh: `### 构建并渲染 —— 用代码搭建文档`,
  },
  {
    id: 'preamble-064',
    join: 'block',
    common: `\`\`\`rust
use ktav::value::{ObjectMap, Value};

let mut top = ObjectMap::default();
top.insert("name".into(),  Value::String("frontend".into()));
top.insert("port".into(),  Value::Integer("8443".into()));
top.insert("tls".into(),   Value::Bool(true));
top.insert("ratio".into(), Value::Float("0.95".into()));
top.insert("notes".into(), Value::Null);

let text = ktav::render::render(&Value::Object(top))?;
\`\`\``,
  },
  {
    id: 'build-render-prefer-serde',
    join: 'block',
    en: `For typical app use prefer the serde path — \`ktav::to_string(&cfg)\` —
and reach for \`Value\` only when the schema is dynamic.`,
    ru: `В обычных сценариях используйте serde-путь — \`ktav::to_string(&cfg)\`.
К \`Value\` обращайтесь только когда схема динамическая.`,
    zh: `通常请走 serde 路径(\`ktav::to_string(&cfg)\`);仅在 schema
是动态的时候才需要直接操作 \`Value\`。`,
  },
  {
    id: 'entry-points-1',
    join: 'block',
    en: `Four public entry points: [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav) for reading, [\`to_string\`](https://docs.rs/ktav) /
[\`to_file\`](https://docs.rs/ktav) for writing.`,
    ru: `Четыре публичных entry point-а: [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav) — для чтения,
[\`to_string\`](https://docs.rs/ktav) / [\`to_file\`](https://docs.rs/ktav) —
для записи.`,
    zh: `四个公共入口:读取用 [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav),写入用
[\`to_string\`](https://docs.rs/ktav) / [\`to_file\`](https://docs.rs/ktav)。`,
  },
  {
    id: 'entry-points-2',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `A complete runnable
example lives in [\`examples/basic.rs\`](examples/basic.rs).`,
    ru: `Полный запускаемый пример — в
[\`examples/basic.rs\`](examples/basic.rs).`,
    zh: `完整可运行示例:[\`examples/basic.rs\`](examples/basic.rs)。`,
  },
  {
    id: 'errors-heading',
    join: 'block',
    en: `### Inspect errors — structured variants for tooling`,
    ru: `### Анализ ошибок — структурированные варианты для тулинга`,
    zh: `### 检视错误 —— 给工具链使用的结构化变体`,
  },
  {
    id: 'errors-structured-intro',
    join: 'block',
    en: `\`Error::Structured(ErrorKind)\` carries a typed category plus a
byte-offset \`Span\` for every parse failure, so editors and linters
can highlight the exact offending range instead of the whole line.`,
    ru: `\`Error::Structured(ErrorKind)\` несёт типизированную категорию плюс
byte-offset \`Span\` для каждого parse-фейла, так что редакторы и
линтеры могут подсветить именно offending диапазон, а не всю строку.`,
    zh: `\`Error::Structured(ErrorKind)\` 为每次解析失败携带类型化的类别加字节
偏移 \`Span\`,因此编辑器和 linter 可以高亮恰好出错的范围,而不是
整行。`,
  },
  {
    id: 'preamble-070',
    join: 'block',
    en: `\`\`\`rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\\nport: 443\\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("line {line}: duplicate key {key:?}");
        println!("offending bytes: {:?}", span.slice(src));   // -> Some("port")
        let (l, c) = span.line_col(src);                      // 1-based / 0-based byte col
        println!("highlight at {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
\`\`\``,
    ru: `\`\`\`rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\\nport: 443\\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("строка {line}: дубликат ключа {key:?}");
        println!("offending байты: {:?}", span.slice(src));    // -> Some("port")
        let (l, c) = span.line_col(src);                       // line 1-based, col 0-based по байтам
        println!("подсветить в {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
\`\`\``,
    zh: `\`\`\`rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\\nport: 443\\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("第 {line} 行:重复键 {key:?}");
        println!("出错字节: {:?}", span.slice(src));   // -> Some("port")
        let (l, c) = span.line_col(src);                // 行 1 起,列 0 起按字节
        println!("高亮 {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
\`\`\``,
  },
  {
    id: 'errors-variants-list-1',
    join: 'block',
    en: `Variants: \`MissingSeparatorSpace\`, \`InvalidTypedScalar\`, \`DuplicateKey\`,
\`KeyPathConflict\`, \`EmptyKey\`, \`InvalidKey\`, \`UnclosedCompound\`,
\`UnbalancedBracket\`, \`InlineNonEmptyCompound\`, \`MissingSeparator\`,
\`LossyScalar\` (strict mode only, see below), \`Other\`.`,
    ru: `Варианты: \`MissingSeparatorSpace\`, \`InvalidTypedScalar\`, \`DuplicateKey\`,
\`KeyPathConflict\`, \`EmptyKey\`, \`InvalidKey\`, \`UnclosedCompound\`,
\`UnbalancedBracket\`, \`InlineNonEmptyCompound\`, \`MissingSeparator\`,
\`LossyScalar\` (только в строгом режиме, см. ниже), \`Other\`.`,
    zh: `变体:\`MissingSeparatorSpace\`、\`InvalidTypedScalar\`、\`DuplicateKey\`、
\`KeyPathConflict\`、\`EmptyKey\`、\`InvalidKey\`、\`UnclosedCompound\`、
\`UnbalancedBracket\`、\`InlineNonEmptyCompound\`、\`MissingSeparator\`、
\`LossyScalar\`（仅严格模式，见下文）、\`Other\`。`,
  },
  {
    id: 'errors-variants-list-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The enum is
\`#[non_exhaustive]\` — always include a \`_ =>\` arm.`,
    ru: `Enum помечен
\`#[non_exhaustive]\` — обязателен arm \`_ =>\`.`,
    zh: `该枚举标注
\`#[non_exhaustive]\` —— 必须包含 \`_ =>\` 分支。`,
  },
  {
    id: 'errors-variants-list-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`Error::line()\` /
\`Error::span()\` are convenience accessors when the variant doesn't
matter.`,
    ru: `\`Error::line()\` /
\`Error::span()\` — convenience accessors для случаев когда вариант
неважен.`,
    zh: `变体不重要时可使用便捷
访问器 \`Error::line()\` / \`Error::span()\`。`,
  },
  {
    id: 'errors-variants-list-4',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The \`Display\` impl produces the same human-readable string the
legacy \`Error::Syntax(_)\` did, so existing string-based callers keep
working.`,
    ru: `\`Display\` выдаёт те же читаемые строки, что и legacy
\`Error::Syntax(_)\`, поэтому существующие string-based вызывающие
работают без изменений.`,
    zh: `\`Display\` 输出的字符串与
遗留 \`Error::Syntax(_)\` 完全一致,因此原有基于字符串的调用方无需修改
即可继续工作。`,
  },
  {
    id: 'errors-example-pointer',
    join: 'block',
    en: `A complete runnable example walks all variants:
[\`examples/errors.rs\`](examples/errors.rs) — \`cargo run --example errors\`.`,
    ru: `Полный запускаемый пример проходит по всем вариантам:
[\`examples/errors.rs\`](examples/errors.rs) — \`cargo run --example errors\`.`,
    zh: `完整可运行示例遍历所有变体:
[\`examples/errors.rs\`](examples/errors.rs) —— \`cargo run --example errors\`。`,
  },
  {
    id: 'envelope-heading',
    join: 'block',
    en: `### One JSON envelope for every structured error`,
    ru: `### Один JSON-конверт для любой структурированной ошибки`,
    zh: `### 任何结构化错误共用一个 JSON 信封`,
  },
  {
    id: 'envelope-intro',
    join: 'block',
    en: `The accessors above are Rust-only. \`ErrorEnvelope\` is the wire
contract for everyone else: one JSON object, nine fields, always all
nine, in a fixed order — \`error\`, \`reason\`, \`line\`, \`line_text\`,
\`span\`, \`path\`, \`body\`, \`canonical\`, \`spec_section\`.`,
    ru: `Аксессоры выше существуют только в Rust. \`ErrorEnvelope\` — контракт
для всех остальных: один JSON-объект, девять полей, всегда все
девять, в фиксированном порядке — \`error\`, \`reason\`, \`line\`,
\`line_text\`, \`span\`, \`path\`, \`body\`, \`canonical\`,
\`spec_section\`.`,
    zh: `上面的访问器只存在于 Rust。\`ErrorEnvelope\` 是给其余各方的传输契约:
一个 JSON 对象,九个字段,永远是全部九个,顺序固定 —— \`error\`、
\`reason\`、\`line\`、\`line_text\`、\`span\`、\`path\`、\`body\`、
\`canonical\`、\`spec_section\`。`,
  },
  {
    id: 'envelope-snippet',
    join: 'block',
    common: `\`\`\`rust
use ktav::{parse, ErrorEnvelope};

let src = "a: 1.10\\n";
if let Err(e) = ktav::parse_strict(src) {
    println!("{}", ErrorEnvelope::from_error(&e, src).to_json());
}
\`\`\``,
  },
  {
    id: 'envelope-snippet-output',
    join: 'block',
    common: `\`\`\`json
{"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
 "span":{"start":0,"end":7},"path":null,"body":"1.10",
 "canonical":"1.1","spec_section":"§3.6/§5.2"}
\`\`\``,
  },
  {
    id: 'envelope-nulls',
    join: 'block',
    en: `Absent information is an explicit \`null\`, never an omitted key, so a
consumer can read every field positionally without negotiating a
schema first.`,
    ru: `Отсутствующие сведения — явный \`null\`, а не пропущенный ключ, поэтому
потребитель читает каждое поле позиционно, без предварительного
согласования схемы.`,
    zh: `缺失的信息是显式的 \`null\`,而不是省略键,因此使用方无需事先协商模式
即可按位置读取每个字段。`,
  },
  {
    id: 'envelope-span-unit',
    join: 'block',
    en: `\`span\` is \`{"start":N,"end":M}\` in **byte offsets into the UTF-8
source**, not UTF-16 code units — the same unit [\`Span\`](https://docs.rs/ktav)
itself uses. An LSP consumer either converts, or negotiates
\`positionEncoding: "utf-8"\`.`,
    ru: `\`span\` — это \`{"start":N,"end":M}\` в **байтовых смещениях по
UTF-8-исходнику**, а не в code units UTF-16: та же единица, что у самого
[\`Span\`](https://docs.rs/ktav). Потребителю LSP нужно либо
конвертировать, либо договариваться о \`positionEncoding: "utf-8"\`.`,
    zh: `\`span\` 是 \`{"start":N,"end":M}\`,单位为 **UTF-8 源文本中的字节
偏移**,而不是 UTF-16 code unit —— 与 [\`Span\`](https://docs.rs/ktav)
本身一致。LSP 使用方要么自行转换,要么协商
\`positionEncoding: "utf-8"\`。`,
  },
  {
    id: 'envelope-path-rule',
    join: 'block',
    en: `\`path\` is an **array of exact decoded key segments, never a joined
string**. A key literally named \`a.b\` is one segment and cannot be
confused with a two-segment path — there is no separator in the wire
contract to be ambiguous about.`,
    ru: `\`path\` — **массив точных декодированных сегментов ключа, а не
склеенная строка**. Ключ, буквально названный \`a.b\`, — это один
сегмент, и спутать его с путём из двух нельзя: в контракте просто нет
разделителя, вокруг которого возникла бы двусмысленность.`,
    zh: `\`path\` 是**精确解码后的键段数组,绝不是拼接字符串**。字面名为
\`a.b\` 的键是一个段,不会与两段路径混淆 —— 传输契约里根本没有可供
产生歧义的分隔符。`,
  },
  {
    id: 'envelope-writer-note',
    join: 'block',
    en: `Writer rejections use the same envelope: \`reason\` carries the § 5.9.0
reason code (\`NonFiniteFloat\`, \`EmptyKeyName\`, …) and the two
rejections are named apart — \`UnrepresentableAt\` when the writer can
say where the offending node is (it fills \`path\` too),
\`Unrepresentable\` when it cannot.`,
    ru: `Отказы писателя используют тот же конверт: \`reason\` несёт код причины
из § 5.9.0 (\`NonFiniteFloat\`, \`EmptyKeyName\`, …), а сами отказы
названы раздельно — \`UnrepresentableAt\`, когда писатель может указать
узел (тогда он заполняет и \`path\`), и \`Unrepresentable\`, когда не
может.`,
    zh: `写入器的拒绝使用同一个信封:\`reason\` 携带 § 5.9.0 的原因码
(\`NonFiniteFloat\`、\`EmptyKeyName\` 等),两种拒绝分别命名 ——
写入器能指出出错节点时为 \`UnrepresentableAt\`(此时也会填充
\`path\`),不能指出时为 \`Unrepresentable\`。`,
  },
  {
    id: 'envelope-rendering',
    join: 'block',
    en: `Rendering is \`to_json()\` (or \`push_json(&mut String)\` to append into
a buffer you own). It is valid JSON for any payload — every string is
escaped per RFC 8259 — and no serde dependency is involved.`,
    ru: `Печать — \`to_json()\` (или \`push_json(&mut String)\`, чтобы дописать в
собственный буфер). Результат — валидный JSON для любого содержимого:
каждая строка экранируется по RFC 8259, зависимость от serde не
задействована.`,
    zh: `渲染用 \`to_json()\`(或 \`push_json(&mut String)\` 追加进你自己的
缓冲区)。对任何载荷它都是合法 JSON —— 每个字符串都按 RFC 8259
转义 —— 且不涉及 serde 依赖。`,
  },
  {
    id: 'strict-mode-heading',
    join: 'block',
    en: `### Strict mode — catch silently canonicalised numbers`,
    ru: `### Строгий режим — ловим молча канонизированные числа`,
    zh: `### 严格模式 —— 捕获被静默规范化的数字`,
  },
  {
    id: 'strict-mode-intro-1',
    join: 'block',
    en: `Types are inferred from a scalar's lexical form, and inferred numbers
are canonicalised: \`version: 1.10\` parses as \`Float(1.1)\` and
\`zip: 01234\` as \`Integer(1234)\`.`,
    ru: `Типы выводятся по лексической форме скаляра, а выведенные числа
канонизируются: \`version: 1.10\` разбирается как \`Float(1.1)\`, а
\`zip: 01234\` — как \`Integer(1234)\`.`,
    zh: `类型由标量的词法形式推断，且推断出的数字会被规范化：\`version: 1.10\`
解析为 \`Float(1.1)\`，\`zip: 01234\` 解析为 \`Integer(1234)\`。`,
  },
  {
    id: 'strict-mode-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The default \`parse()\` does this
silently, so writing the document back out rewrites it.`,
    ru: `Обычный \`parse()\` делает это молча,
поэтому обратная запись документа его переписывает.`,
    zh: `默认的
\`parse()\` 会静默完成这一过程，因此把文档写回去就会改写它。`,
  },
  {
    id: 'strict-mode-rejects',
    join: 'block',
    en: `\`parse_strict()\` rejects such **lossy scalars** instead:`,
    ru: `\`parse_strict()\` вместо этого отвергает такие **скаляры с потерей**:`,
    zh: `\`parse_strict()\` 则拒绝这类**有损标量**：`,
  },
  {
    id: 'preamble-080',
    join: 'block',
    en: `\`\`\`rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "zip: 01234\\n";

assert!(parse(src).is_ok());                 // Integer(1234) — leading zero gone

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("expected LossyScalar, got {other:?}"),
}
\`\`\``,
    ru: `\`\`\`rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "zip: 01234\\n";

assert!(parse(src).is_ok());                 // Integer(1234) — ведущий ноль потерян

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("ожидался LossyScalar, получено {other:?}"),
}
\`\`\``,
    zh: `\`\`\`rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "zip: 01234\\n";

assert!(parse(src).is_ok());                 // Integer(1234) —— 前导零丢失

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("期望 LossyScalar，实际为 {other:?}"),
}
\`\`\``,
  },
  {
    id: 'strict-mode-fix-options-1',
    join: 'block',
    en: `Fix either by appending \`::\` to keep the value a String
(\`zip:: 01234\`) or by writing the canonical number.`,
    ru: `Исправить можно либо дописав \`::\`, чтобы значение осталось строкой
(\`zip:: 01234\`), либо записав число в канонической форме.`,
    zh: `修复方式有两种：追加 \`::\` 让该值保持字符串（\`zip:: 01234\`），或直接
写成规范数字。`,
  },
  {
    id: 'strict-mode-fix-options-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Any document
\`parse_strict()\` accepts yields exactly the same \`Value\` tree as
\`parse()\`, so strict mode is a validation gate, not a different
dialect.`,
    ru: `Любой
документ, принятый \`parse_strict()\`, даёт ровно то же дерево \`Value\`,
что и \`parse()\` — строгий режим это проверочный шлюз, а не другой
диалект.`,
    zh: `凡是 \`parse_strict()\` 接受的文档，其产生的 \`Value\` 树
与 \`parse()\` 完全相同 —— 严格模式是一道校验闸门，而非另一种方言。`,
  },
  {
    id: 'strict-mode-fix-options-3',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `The serde path (\`from_str\`) has no strict variant yet.`,
    ru: `У serde-пути (\`from_str\`) строгого варианта пока нет.`,
    zh: `serde 路径（\`from_str\`）目前尚无严格模式变体。`,
  },
  {
    id: 'stream-parse-heading',
    join: 'block',
    en: `### Stream parse — events without an intermediate tree`,
    ru: `### Stream-парсинг — события без промежуточного дерева`,
    zh: `### 流式解析 —— 不构建中间树的事件流`,
  },
  {
    id: 'stream-parse-intro-1',
    join: 'block',
    en: `\`parse_events\` invokes a callback for each parse event, with strings
borrowed directly into the input buffer — no allocation per event, no
intermediate \`Value\` tree.`,
    ru: `\`parse_events\` вызывает callback на каждое событие парсинга, со
строками заимствующимися напрямую из input-буфера — без аллокации на
событие, без промежуточного \`Value\`-дерева.`,
    zh: `\`parse_events\` 对每个解析事件调用回调,字符串直接从 input 缓冲区
借用 —— 不在每个事件上分配,也不构建中间 \`Value\` 树。`,
  },
  {
    id: 'stream-parse-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Useful when you don't need the full document:
counting keys, streaming to another format, building a custom shape.`,
    ru: `Полезно когда полный
документ не нужен: подсчёт ключей, стриминг в другой формат, построение
custom-shape-а.`,
    zh: `在不需要完整
文档时很有用:统计键、流式转换到其他格式、构建自定义形状。`,
  },
  {
    id: 'preamble-087',
    join: 'block',
    common: `\`\`\`rust
use ktav::{parse_events, ParseEvent};

let src = "port: 8080\\nhost: example.com\\n";
let mut keys = Vec::new();
parse_events(src, |ev| {
    if let ParseEvent::Key(k) = ev {
        keys.push(k.to_string());
    }
})?;
assert_eq!(keys, ["port", "host"]);
\`\`\``,
  },
  {
    id: 'stream-parse-root-events-1',
    join: 'block',
    en: `The root is \`BeginObject\`/\`EndObject\` or \`BeginArray\`/\`EndArray\`
depending on the document's first content line (an Object here, since
\`port: 8080\` is a pair); nested compounds bracket their contents the
same way.`,
    ru: `Корень — \`BeginObject\`/\`EndObject\` или \`BeginArray\`/\`EndArray\`, в
зависимости от первой содержательной строки документа (здесь Object,
так как \`port: 8080\` — это пара); вложенные компаунды обрамляют своё
содержимое так же.`,
    zh: `根据文档第一条内容行的形状,根节点是 \`BeginObject\`/\`EndObject\` 或
\`BeginArray\`/\`EndArray\`(此处因为 \`port: 8080\` 是一个 pair,所以是
Object);嵌套复合值以同样方式括起自身内容。`,
  },
  {
    id: 'stream-parse-root-events-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`ParseEvent\` is \`#[non_exhaustive]\`.`,
    ru: `\`ParseEvent\` помечен
\`#[non_exhaustive]\`.`,
    zh: `\`ParseEvent\` 标注
\`#[non_exhaustive]\`。`,
  },
  {
    id: 'stream-parse-root-events-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `A complete runnable
example with depth tracking
and a pretty-printer:
[\`examples/events.rs\`](examples/events.rs) — \`cargo run --example events\`.`,
    ru: `Полный запускаемый пример с tracking-ом глубины
и pretty-print-ом:
[\`examples/events.rs\`](examples/events.rs) — \`cargo run --example events\`.`,
    zh: `完整的、带深度跟踪
和漂亮打印的可运行示例:
[\`examples/events.rs\`](examples/events.rs) —— \`cargo run --example events\`。`,
  },
  {
    id: 'preamble-091',
    join: 'block',
    en: `### Numbers`,
    ru: `### Числа`,
    zh: `### 数字`,
  },
  {
    id: 'stream-numbers-notes-1',
    join: 'block',
    en: `Rust numeric types (\`u8\`..\`u128\`, \`i8\`..\`i128\`, \`usize\`, \`isize\`, \`f32\`,
\`f64\`) serialize to Ktav as bare numbers: \`port: 8080\`, \`ratio: 0.5\`.`,
    ru: `Числовые Rust-типы (\`u8\`..\`u128\`, \`i8\`..\`i128\`, \`usize\`, \`isize\`, \`f32\`,
\`f64\`) сериализуются в Ktav как голые числа: \`port: 8080\`,
\`ratio: 0.5\`.`,
    zh: `Rust 数值类型(\`u8\`..\`u128\`、\`i8\`..\`i128\`、\`usize\`、\`isize\`、\`f32\`、
\`f64\`)会以裸数字序列化到 Ktav:\`port: 8080\`、\`ratio: 0.5\`。`,
  },
  {
    id: 'stream-numbers-notes-2',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `Coming back, a bare integer/decimal body deserializes straight into
the target numeric type; a value that arrived as a string (e.g. forced
with \`::\`) is still accepted via \`FromStr\`.`,
    ru: `На обратном пути голое целое/десятичное тело
десериализуется прямо в нужный числовой тип; значение, пришедшее
строкой (например, форсированное через \`::\`), по-прежнему принимается
через \`FromStr\`.`,
    zh: `回程时,
裸整数/小数 body 直接反序列化为目标数值类型;以字符串形式到达的值
(例如用 \`::\` 强制的)仍通过 \`FromStr\` 接受。`,
  },
  {
    id: 'stream-numbers-notes-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`NaN\` / \`±Infinity\` are
rejected by the serializer (Ktav does not represent them).`,
    ru: `\`NaN\` / \`±Infinity\` отвергаются сериализатором
(Ktav их не представляет).`,
    zh: `\`NaN\` / \`±Infinity\` 会被
序列化器拒绝(Ktav 不表示这些值)。`,
  },
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
  {
    id: 'round-trip-heading',
    join: 'block',
    common: `## Round-trip`,
  },
  {
    id: 'round-trip-snippet',
    join: 'block',
    common: `\`\`\`rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
\`\`\``,
  },
  {
    id: 'preamble-163',
    join: 'block',
    en: `Serialization preserves:`,
    ru: `Сериализация сохраняет:`,
    zh: `序列化会保持:`,
  },
  {
    id: 'preamble-164',
    join: 'tight',
    en: `- **Field order** — \`Value::Object\` is backed by an \`IndexMap\`, so the
  order is whatever serde emits (for structs: declaration order).`,
    ru: `- **Порядок полей** — \`Value::Object\` лежит на \`IndexMap\`, так что
  порядок — тот, что эмитит serde (для struct-ов: порядок объявления).`,
    zh: `- **字段顺序** —— \`Value::Object\` 底层是 \`IndexMap\`,顺序由 serde
  输出决定(对结构体而言:就是声明顺序)。`,
  },
  {
    id: 'preamble-165',
    join: 'tight',
    en: `- **Literal strings** — values starting with \`{\` or \`[\` are emitted
  with the \`::\` marker.`,
    ru: `- **Литеральные строки** — значения, начинающиеся с \`{\` или \`[\`,
  эмитятся с маркером \`::\`.`,
    zh: `- **字面量字符串** —— 以 \`{\` 或 \`[\` 开头的值会带 \`::\` 标记输出。`,
  },
  {
    id: 'preamble-166',
    join: 'tight',
    en: `- **\`None\` fields** — skipped on output; reappear as \`None\` on input
  (via serde's \`Option\` handling).`,
    ru: `- **Поля \`None\`** — пропускаются на выходе; восстанавливаются как
  \`None\` на входе (через обработку \`Option\` в serde).`,
    zh: `- **\`None\` 字段** —— 输出时跳过;输入时通过 serde 的 \`Option\`
  处理重新出现为 \`None\`。`,
  },
  {
    id: 'formatting-heading',
    join: 'block',
    en: `## Formatting — canonical spelling, comments kept`,
    ru: `## Форматирование — каноническое написание, комментарии на месте`,
    zh: `## 格式化 —— 规范写法,注释保留`,
  },
  {
    id: 'formatting-intro',
    join: 'block',
    en: `\`ktav::format_str\` rewrites a document into the structural spelling
\`emit_canonical\` produces, but keeps the trivia the canonical writer
drops. Every comment survives verbatim.`,
    ru: `\`ktav::format_str\` переписывает документ в то структурное написание,
которое выдаёт \`emit_canonical\`, но сохраняет оформление, выбрасываемое
каноническим писателем. Каждый комментарий сохраняется дословно.`,
    zh: `\`ktav::format_str\` 把文档改写为 \`emit_canonical\` 所产出的结构写法,
但保留规范写入器会丢弃的附属内容。每条注释都逐字保留。`,
  },
  {
    id: 'formatting-snippet',
    join: 'block',
    common: `\`\`\`rust
let tidied = ktav::format_str("## the server\\nserver: {host: a, port: 80}\\n")?;
assert_eq!(tidied, "## the server\\nserver: {\\n    host: a\\n    port: 80\\n}\\n");
\`\`\``,
  },
  {
    id: 'formatting-snippet-rendered',
    join: 'block',
    en: `That is, the inline compound expands to the canonical multi-line
form and the comment stays exactly where it was:`,
    ru: `То есть inline-компаунд разворачивается в каноническую многострочную
форму, а комментарий остаётся ровно там, где был:`,
    zh: `也就是说,行内复合值展开为规范的多行形式,而注释仍停留在原处:`,
  },
  {
    id: 'formatting-snippet-ktav',
    join: 'block',
    common: `\`\`\`ktav
## the server
server: {
    host: a
    port: 80
}
\`\`\``,
  },
  {
    id: 'formatting-blank-lines',
    join: 'block',
    en: `Blank lines survive as a grouping hint, but a run of two or more
collapses to exactly one, and blank padding just inside a bracket is
dropped. That is what makes the transform a fixed point: formatting
already-formatted output never changes it again.`,
    ru: `Пустые строки сохраняются как подсказка группировки, но серия из двух
и более схлопывается ровно в одну, а пустой отступ сразу внутри скобки
выбрасывается. Именно это делает преобразование неподвижной точкой:
форматирование уже отформатированного вывода больше ничего не меняет.`,
    zh: `空行作为分组提示保留,但连续两行及以上会折叠为恰好一行,紧贴括号内侧
的空行填充会被丢弃。正是这一点让该变换成为不动点:对已格式化的输出再
次格式化不会再有任何改变。`,
  },
  {
    id: 'formatting-key-order',
    join: 'block',
    en: `Key order is never changed. Canonical form has no sorting rule
(§ 5.9), and reordering keys would make review diffs worse, not
better — this is a spelling normaliser, not a refactoring tool.`,
    ru: `Порядок ключей не меняется никогда. У канонической формы нет правила
сортировки (§ 5.9), а перестановка ключей ухудшила бы диффы на ревью,
а не улучшила: это нормализатор написания, а не инструмент
рефакторинга.`,
    zh: `键序永不改变。规范形式没有排序规则(§ 5.9),而重排键只会让评审差异
更糟,而非更好 —— 这是写法规范化工具,不是重构工具。`,
  },
  {
    id: 'formatting-canonical-relation',
    join: 'block',
    en: `For a document with no comments **and no blank lines**,
\`format_str\` equals \`emit_canonical\` of its parse. The stronger
condition is deliberate: blank lines are no more part of the \`Value\`
model than comments are, so \`emit_canonical\` drops them and
\`format_str\` does not.`,
    ru: `Для документа без комментариев **и без пустых строк** \`format_str\`
совпадает с \`emit_canonical\` его разбора. Условие намеренно сильнее
очевидного: пустые строки входят в модель \`Value\` ровно настолько же,
насколько комментарии, — то есть никак, поэтому \`emit_canonical\` их
выбрасывает, а \`format_str\` нет.`,
    zh: `对于既无注释**也无空行**的文档,\`format_str\` 等于其解析结果的
\`emit_canonical\`。这个更强的条件是刻意的:空行与注释一样都不属于
\`Value\` 模型,因此 \`emit_canonical\` 会丢弃它们,而 \`format_str\`
不会。`,
  },
  {
    id: 'formatting-cli-heading',
    join: 'block',
    en: `### \`ktav-fmt\` — the optional command-line formatter`,
    ru: `### \`ktav-fmt\` — необязательный форматтер командной строки`,
    zh: `### \`ktav-fmt\` —— 可选的命令行格式化器`,
  },
  {
    id: 'formatting-cli-optional',
    join: 'block',
    en: `Behind the \`cli\` feature, off by default. In a project that already
has a toolchain the library call above plus a build hook is usually the
better answer, and editors format through \`ktav-lsp\`; the binary is
for the case where neither is at hand.`,
    ru: `Живёт за feature-флагом \`cli\` и по умолчанию выключен. В проекте, где
тулчейн уже есть, библиотечный вызов выше плюс хук сборки обычно
уместнее, а редакторы форматируют через \`ktav-lsp\`; бинарь — для
случая, когда ни того, ни другого под рукой нет.`,
    zh: `位于 \`cli\` feature 之后,默认关闭。在已有工具链的项目里,上面的库
调用加一个构建钩子通常更合适,编辑器则经由 \`ktav-lsp\` 格式化;二进制
是留给两者都不可用的场景。`,
  },
  {
    id: 'formatting-cli-usage',
    join: 'block',
    common: `\`\`\`text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
\`\`\``,
  },
  {
    id: 'formatting-cli-check',
    join: 'block',
    en: `\`--check\` writes nothing and prints the path of every file that is
not already formatted, so it drops straight into CI next to
\`cargo fmt --check\`.`,
    ru: `\`--check\` ничего не пишет и печатает путь каждого файла, который ещё
не отформатирован, — так что он встаёт в CI прямо рядом с
\`cargo fmt --check\`.`,
    zh: `\`--check\` 不写入任何内容,只打印每个尚未格式化的文件路径,因此可以
直接放进 CI,紧挨着 \`cargo fmt --check\`。`,
  },
  {
    id: 'cabi-heading',
    join: 'block',
    en: `## The C ABI for other languages`,
    ru: `## C ABI для других языков`,
    zh: `## 面向其他语言的 C ABI`,
  },
  {
    id: 'cabi-intro',
    join: 'block',
    en: `Six language bindings — Go, Java, PHP, C#, JS and Python — load a
small native library built on this crate. The shareable half of that
shim lives behind the off-by-default \`cabi\` feature: the
\`ktav::cabi\` module carries the wire decoding, the six document
operations and the error encoding, and one macro call in a binding's
\`cdylib\` expands the exported symbols.`,
    ru: `Шесть языковых биндингов — Go, Java, PHP, C#, JS и Python — загружают
небольшую нативную библиотеку, собранную на этом crate. Разделяемая
половина шима живёт за выключенной по умолчанию фичей \`cabi\`: модуль
\`ktav::cabi\` несёт декодирование wire-формата, шесть операций над
документами и кодирование ошибок, а один макровызов в \`cdylib\`
биндинга разворачивает экспортируемые символы.`,
    zh: `六个语言绑定——Go、Java、PHP、C#、JS 与 Python——加载一个构建在
本 crate 之上的小型原生库。垫片中可共享的一半位于默认关闭的
\`cabi\` feature 之后:\`ktav::cabi\` 模块承载 wire 解码、六项文档
操作与错误编码,而绑定 cdylib 中的一次宏调用即可展开全部导出符号。`,
  },
  {
    id: 'cabi-usage',
    join: 'block',
    common: `\`\`\`text
// crates/cabi/src/lib.rs of a binding — the whole body:
ktav::declare_cabi!();
\`\`\``,
  },
  {
    id: 'cabi-symbols',
    join: 'block',
    en: `The expansion exports nine symbols: the six document functions
(\`ktav_loads\`, \`ktav_loads_strict\`, \`ktav_dumps\`,
\`ktav_dumps_force_strings\`, \`ktav_emit_canonical\`,
\`ktav_format\`), \`ktav_free\`, \`ktav_version\` and
\`ktav_abi_version\`. Every error leaves as the nine-field JSON
envelope, so a host never has to sniff plain text against JSON, and
\`ktav_abi_version()\` lets a host refuse a stale native library
instead of corrupting memory. The full contract — signatures,
ownership, the error encoding, the artifact naming convention
(\`ktav_cabi-windows-amd64.dll\`, \`libktav_cabi-darwin-arm64.dylib\`,
\`libktav_cabi-linux-amd64.so\`, the \`$KTAV_LIB_PATH\` override) — is
specified in [docs/CABI.md](docs/CABI.md).`,
    ru: `Развёртка экспортирует девять символов: шесть функций-операций
(\`ktav_loads\`, \`ktav_loads_strict\`, \`ktav_dumps\`,
\`ktav_dumps_force_strings\`, \`ktav_emit_canonical\`,
\`ktav_format\`), \`ktav_free\`, \`ktav_version\` и
\`ktav_abi_version\`. Каждая ошибка уходит девятиполевым
JSON-конвертом, поэтому хосту не приходится гадать, текст перед ним
или JSON, а \`ktav_abi_version()\` позволяет хосту отказаться от
устаревшей нативной библиотеки вместо порчи памяти. Полный контракт —
сигнатуры, владение, кодирование ошибок, конвенция имён артефактов
(\`ktav_cabi-windows-amd64.dll\`, \`libktav_cabi-darwin-arm64.dylib\`,
\`libktav_cabi-linux-amd64.so\`, переопределение \`$KTAV_LIB_PATH\`) —
описан в [docs/CABI.md](docs/CABI.md).`,
    zh: `展开导出九个符号:六个文档操作函数(\`ktav_loads\`、
\`ktav_loads_strict\`、\`ktav_dumps\`、\`ktav_dumps_force_strings\`、
\`ktav_emit_canonical\`、\`ktav_format\`)、\`ktav_free\`、
\`ktav_version\` 与 \`ktav_abi_version\`。每个错误都以九字段 JSON
信封传递,宿主无需嗅探它是纯文本还是 JSON;\`ktav_abi_version()\`
让宿主拒绝过期的原生库而不是破坏内存。完整契约——签名、所有权、
错误编码、产物命名约定(\`ktav_cabi-windows-amd64.dll\`、
\`libktav_cabi-darwin-arm64.dylib\`、\`libktav_cabi-linux-amd64.so\`、
\`$KTAV_LIB_PATH\` 覆盖)——见
[docs/CABI.md](docs/CABI.md)。`,
  },
  {
    id: 'architecture-heading',
    join: 'block',
    en: `## Architecture`,
    ru: `## Архитектура`,
    zh: `## 架构`,
  },
  {
    id: 'architecture-tree',
    join: 'block',
    common: `\`\`\`
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── thin/             — arena-backed borrowed parse (parse_events)
├── render/           — pretty-printer, canonical writer, formatter
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error, ErrorKind, ErrorEnvelope, serde::Error
├── bin/ktav-fmt.rs   — the ktav-fmt command-line formatter
└── lib.rs            — glue: from_str / to_string / format_str / …
\`\`\``,
  },
  {
    id: 'architecture-file-conventions',
    join: 'block',
    en: `Each file holds one exported item; implementation details are private to
their parent module.`,
    ru: `В каждом файле — один экспортируемый элемент; детали реализации
приватны внутри родительского модуля.`,
    zh: `每个文件持有一个导出项;实现细节相对其父模块私有。`,
  },
  {
    id: 'not-do-heading',
    join: 'block',
    en: `## What Ktav does NOT do — and never will`,
    ru: `## Чего Ktav НЕ делает — и никогда не будет`,
    zh: `## Ktav **不**做、也永远不会做的事`,
  },
  {
    id: 'preamble-171',
    join: 'block',
    en: `- **Inline non-empty compounds** like \`x: { a: 1, b: 2 }\`.`,
    ru: `- **Inline непустые compound-ы** вроде \`x: { a: 1, b: 2 }\`.`,
    zh: `- **内联的非空复合值**,比如 \`x: { a: 1, b: 2 }\`。`,
  },
  {
    id: 'preamble-172',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `They'd bring
  commas, and commas would bring escaping.`,
    ru: `Они
  притащили бы запятые, а запятые притащили бы escape.`,
    zh: `它们会带来逗号,
  逗号又会带来转义。`,
  },
  {
    id: 'preamble-173',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Compound values are
  multiline.`,
    ru: `Compound-
  значения многострочны.`,
    zh: `复合值保持多行。`,
  },
  {
    id: 'preamble-174',
    join: 'tight',
    en: `- **Anchors / aliases / merge keys** (\`&anchor\`, \`*ref\`, \`<<:\`).`,
    ru: `- **Якоря / алиасы / merge-ключи** (\`&anchor\`, \`*ref\`, \`<<:\`).`,
    zh: `- **锚点 / 别名 / 合并键**(\`&anchor\`、\`*ref\`、\`<<:\`)。`,
  },
  {
    id: 'preamble-175',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Any
  line whose meaning depends on a declaration far away stops being
  self-sufficient.`,
    ru: `Любая
  строка, смысл которой зависит от декларации в отдалённом месте,
  перестаёт быть самодостаточной.`,
    zh: `任何一行
  若其含义依赖远处的声明,就不再自洽。`,
  },
  {
    id: 'preamble-176',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `If you want DRY, compose defaults in code.`,
    ru: `Если нужен DRY — композируйте
  defaults в коде.`,
    zh: `若需要 DRY,请在代码里
  组合默认值。`,
  },
  {
    id: 'preamble-177',
    join: 'tight',
    en: `- **File includes** (\`@include\`, \`!import\`).`,
    ru: `- **Инклюды файлов** (\`@include\`, \`!import\`).`,
    zh: `- **文件包含**(\`@include\`、\`!import\`)。`,
  },
  {
    id: 'preamble-178',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Write a wrapper in code
  for large configs.`,
    ru: `Для больших конфигов
  напишите обёртку в коде.`,
    zh: `大型配置请在代码里包一层
  封装。`,
  },
  {
    id: 'preamble-179',
    join: 'tight',
    en: `- **Top-level arrays.** The document is always an object.`,
    ru: `- **Массивы на верхнем уровне.** Документ всегда — объект.`,
    zh: `- **顶层数组。** 文档始终是对象。`,
  },
  {
    id: 'installation-heading',
    join: 'block',
    en: `## Installation`,
    ru: `## Установка`,
    zh: `## 安装`,
  },
  {
    id: 'installation-toml',
    join: 'block',
    common: `\`\`\`toml
[dependencies]
ktav = "0.7.1"
serde = { version = "1", features = ["derive"] }
\`\`\``,
  },
  {
    id: 'installation-cli',
    join: 'block',
    en: `The formatter is also available as a binary, behind an off-by-default
feature:`,
    ru: `Форматтер доступен и как исполняемый файл — за выключенным по
умолчанию feature-флагом:`,
    zh: `格式化器也可作为二进制取用,位于一个默认关闭的 feature 之后:`,
  },
  {
    id: 'installation-cli-cmd',
    join: 'block',
    common: `\`\`\`sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
\`\`\``,
  },
  {
    id: 'support-heading',
    join: 'block',
    en: `## Support the project`,
    ru: `## Поддержите проект`,
    zh: `## 支持本项目`,
  },
  {
    id: 'support-appeal-1',
    join: 'block',
    en: `The author has many ideas that could be broadly useful to IT worldwide —
not limited to Ktav.`,
    ru: `У автора много идей, которые могут быть полезны IT во всём мире, — и
далеко не только для Ktav.`,
    zh: `作者有许多构想,可能对全球 IT 广泛有益——不局限于 Ktav。`,
  },
  {
    id: 'support-appeal-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Realizing them requires funding.`,
    ru: `Их реализация требует финансирования.`,
    zh: `实现这些
构想需要资金支持。`,
  },
  {
    id: 'support-appeal-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `If you'd like to
help, please reach out at **phpcraftdream@gmail.com**.`,
    ru: `Если
вы хотите помочь — пишите на **phpcraftdream@gmail.com**.`,
    zh: `如果您愿意提供帮助,请联系
**phpcraftdream@gmail.com**。`,
  },
  {
    id: 'license-heading',
    join: 'block',
    en: `## License`,
    ru: `## Лицензия`,
    zh: `## 许可证`,
  },
  {
    id: 'license-text',
    join: 'block',
    en: `Dual-licensed under **MIT OR Apache-2.0** at your option. See
[LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).`,
    ru: `Двойное лицензирование: **MIT OR Apache-2.0** — на ваш выбор. См.
[LICENSE-MIT](LICENSE-MIT) и [LICENSE-APACHE](LICENSE-APACHE).`,
    zh: `双许可：**MIT OR Apache-2.0**，由你选择其一。详见
[LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE)。`,
  },
  {
    id: 'implementations-heading',
    join: 'block',
    en: `## Other Ktav implementations`,
    ru: `## Другие реализации Ktav`,
    zh: `## 其他 Ktav 实现`,
  },
  {
    id: 'preamble-189',
    join: 'block',
    en: `- [\`spec\`](https://github.com/ktav-lang/spec) — specification + conformance suite`,
    ru: `- [\`spec\`](https://github.com/ktav-lang/spec) — спецификация + conformance-тесты`,
    zh: `- [\`spec\`](https://github.com/ktav-lang/spec) —— 规范 + 一致性测试套件`,
  },
  {
    id: 'preamble-190',
    join: 'tight',
    en: `- [\`csharp\`](https://github.com/ktav-lang/csharp) — C# / .NET (\`dotnet add package Ktav\`)`,
    ru: `- [\`csharp\`](https://github.com/ktav-lang/csharp) — C# / .NET (\`dotnet add package Ktav\`)`,
    zh: `- [\`csharp\`](https://github.com/ktav-lang/csharp) —— C# / .NET(\`dotnet add package Ktav\`)`,
  },
  {
    id: 'preamble-191',
    join: 'tight',
    en: `- [\`golang\`](https://github.com/ktav-lang/golang) — Go (\`go get github.com/ktav-lang/golang\`)`,
    ru: `- [\`golang\`](https://github.com/ktav-lang/golang) — Go (\`go get github.com/ktav-lang/golang\`)`,
    zh: `- [\`golang\`](https://github.com/ktav-lang/golang) —— Go(\`go get github.com/ktav-lang/golang\`)`,
  },
  {
    id: 'preamble-192',
    join: 'tight',
    en: `- [\`java\`](https://github.com/ktav-lang/java) — Java / JVM (\`io.github.ktav-lang:ktav\` on Maven Central)`,
    ru: `- [\`java\`](https://github.com/ktav-lang/java) — Java / JVM (\`io.github.ktav-lang:ktav\` на Maven Central)`,
    zh: `- [\`java\`](https://github.com/ktav-lang/java) —— Java / JVM(\`io.github.ktav-lang:ktav\`,Maven Central)`,
  },
  {
    id: 'preamble-193',
    join: 'tight',
    en: `- [\`js\`](https://github.com/ktav-lang/js) — JS / TS (\`npm install @ktav-lang/ktav\`)`,
    ru: `- [\`js\`](https://github.com/ktav-lang/js) — JS / TS (\`npm install @ktav-lang/ktav\`)`,
    zh: `- [\`js\`](https://github.com/ktav-lang/js) —— JS / TS(\`npm install @ktav-lang/ktav\`)`,
  },
  {
    id: 'preamble-194',
    join: 'tight',
    en: `- [\`php\`](https://github.com/ktav-lang/php) — PHP (\`composer require ktav-lang/ktav\`)`,
    ru: `- [\`php\`](https://github.com/ktav-lang/php) — PHP (\`composer require ktav-lang/ktav\`)`,
    zh: `- [\`php\`](https://github.com/ktav-lang/php) —— PHP(\`composer require ktav-lang/ktav\`)`,
  },
  {
    id: 'preamble-195',
    join: 'tight',
    en: `- [\`python\`](https://github.com/ktav-lang/python) — Python (\`pip install ktav\`)`,
    ru: `- [\`python\`](https://github.com/ktav-lang/python) — Python (\`pip install ktav\`)`,
    zh: `- [\`python\`](https://github.com/ktav-lang/python) —— Python(\`pip install ktav\`)`,
  },
];
