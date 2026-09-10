export default [
  {
    id: 'intro-title',
    common: `# Ktav (כְּתָב)`,
  },
  {
    id: 'badge-row',
    common: `[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)`,
  },
  {
    id: 'tagline-quote',
    en: `> A plain configuration format. JSON5-shaped, but without the quotes,
> without the commas, with dotted keys for nesting. Native \`serde\`
> integration.`,
    ru: `> Простой формат конфигурации. Формы JSON5, но без кавычек, без
> запятых, с точечными ключами для вложенности. Нативная интеграция
> с \`serde\`.`,
    zh: `> 一种朴素的配置格式。形态上接近 JSON5,但不带引号、不用逗号,
> 以点分键表达嵌套。原生 \`serde\` 集成。`,
  },
  {
    id: 'languages-row',
    en: `**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)`,
    ru: `**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)`,
    zh: `**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**`,
  },
  {
    id: 'playground-row',
    en: `**Playground:** convert JSON / YAML / TOML / INI ⇄ Ktav in your browser at **[ktav-lang.github.io](https://ktav-lang.github.io/)**.`,
    ru: `**Песочница:** конвертация JSON / YAML / TOML / INI ⇄ Ktav прямо в браузере — **[ktav-lang.github.io](https://ktav-lang.github.io/)**.`,
    zh: `**演练场：** 在浏览器中互转 JSON / YAML / TOML / INI ⇄ Ktav — **[ktav-lang.github.io](https://ktav-lang.github.io/)**。`,
  },
  {
    id: 'spec-pointer',
    en: `**Specification:** this crate implements **Ktav**. The format is
versioned and maintained independently of this crate — see
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec) for the formal
document.`,
    ru: `**Спецификация:** этот crate реализует **Ktav**. Формат версионируется
и поддерживается независимо от crate-а — см.
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec) для
канонического документа.`,
    zh: `**规范:** 本 crate 实现 **Ktav**。格式与 crate 彼此独立地
版本化与维护——规范正文见
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec)。`,
  },
  // R15-F4: merges the English-only "**Spec 0.7 status:** ..." paragraph with
  // the following `---` rule, because every unit renders into all three
  // languages and the ru/zh files have no counterpart for that paragraph
  // (docs parity gap).
  // TODO: split into a full triple plus a separate common `---` unit once
  // ru/zh translations are written.
  {
    id: 'spec-0-7-status-with-rule',
    en: `**Spec 0.7 status:** the 0.7 draft (still unreleased; see the spec
repository's \`versions/0.7\`) is implemented in this tree and covered by
its conformance suite — BOM handling (§ 3.1), \`(...)\` trailing-whitespace
stripping (§ 5.6), escape-forces-String (§ 3.7 / § 5.2), the Float domain
floor, \`\\uXXXX\` escapes (§ 3.7.1), quoted keys (§ 5.3.3) with the § 5.9.10
canonical key rules, writer-side representable-value rejection (§ 5.9.0),
and the \`UnterminatedQuotedKey\` (§ 6.16) / \`InvalidUtf8\` (§ 6.15) error
categories. The crate version and its \`spec-version\` metadata remain
pinned to \`0.6.4\` pending the 0.7 release decision.

---`,
    ru: `---`,
    zh: `---`,
  },
  {
    id: 'name-heading',
    en: `## Name`,
    ru: `## Название`,
    zh: `## 名称`,
  },
  {
    id: 'name-meaning',
    en: `*Ktav* (Hebrew: **כְּתָב**) means "writing, that which is written" — a
thing recorded in a form fixed enough that its meaning does not depend
on who passes it along. The name fits literally: a config file *is*
ktav on disk, and the library reads it and hands you back a live
structure without making anything up along the way.`,
    ru: `*Ktav* (иврит: **כְּתָב**) означает «письмо, то, что записано» —
нечто зафиксированное в форме, достаточно устойчивой, чтобы смысл не
зависел от того, кто это передаёт дальше. Название подходит буквально:
конфиг-файл *и есть* ktav на диске, а библиотека его читает и отдаёт
живую структуру, ничего не выдумывая по дороге.`,
    zh: `*Ktav*(希伯来语:**כְּתָב**)意为「书写、被写下来的东西」——一种
以足够稳固的形态被记录下来的东西,其意义不依赖于传递者。名字用得
很字面:一份配置文件*就是*磁盘上的 ktav,而本库把它读进来,原样
交出一个活生生的结构,不会在途中擅自脑补。`,
  },
  {
    id: 'motto-heading',
    en: `## Motto`,
    ru: `## Девиз`,
    zh: `## 格言`,
  },
  {
    id: 'motto-quote',
    en: `> **Be the config's friend, not its examiner. The config isn't perfect —
> but it's the best one.**`,
    ru: `> **Будь другом конфига, а не его экзаменатором. Конфиг неидеален —
> но он лучший из возможных.**`,
    zh: `> **做配置的朋友,别做它的考官。配置并不完美——但已是最好的那一份。**`,
  },
  {
    id: 'motto-locality',
    en: `Every rule is local. Every line either stands on its own or depends only
on visible brackets. No indentation pitfalls, no forgotten quotes, no
trailing-comma arithmetic.`,
    ru: `Каждое правило локально. Каждая строка либо стоит сама по себе, либо
зависит только от видимых скобок. Никаких ловушек с отступами, никаких
забытых кавычек, никакой арифметики замыкающих запятых.`,
    zh: `每条规则都是局部的。每一行要么独立成立,要么只依赖于可见的括号。
没有缩进陷阱,没有忘记的引号,没有尾随逗号的算术。`,
  },
  {
    id: 'rules-heading',
    en: `## The rules`,
    ru: `## Правила`,
    zh: `## 规则`,
  },
  {
    id: 'rules-intro',
    en: `A Ktav document is an implicit top-level object. Inside any object you
have pairs; inside any array you have items.`,
    ru: `Документ Ktav — это неявный объект верхнего уровня. Внутри любого
объекта — пары; внутри любого массива — элементы.`,
    zh: `Ktav 文档是一个隐式的顶层对象。任何对象里是键值对,任何数组里是
元素。`,
  },
  {
    id: 'rules-reference-card',
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
    id: 'rules-summary',
    en: `That's the whole language. No commas, no quotes, no escape inside the
value itself — the only "escape" is the \`::\` marker, and it lives in the
separator (for pairs) or as a line prefix (for array items).`,
    ru: `Это весь язык. Никаких запятых, никаких кавычек, никаких escape
внутри самого значения — единственный «escape» это маркер \`::\`, и он
живёт в разделителе (для пар) или в префиксе строки (для элементов
массива).`,
    zh: `整个语言就这些。没有逗号、没有引号、没有值内部的转义——唯一的
「转义」是 \`::\` 标记,它出现在分隔符里(用于键值对)或行首前缀里
(用于数组元素)。`,
  },
  {
    id: 'values-heading',
    en: `## Values and special tokens`,
    ru: `## Значения и специальные токены`,
    zh: `## 值与特殊记号`,
  },
  {
    id: 'strings-heading',
    en: `### Strings`,
    ru: `### Строки`,
    zh: `### 字符串`,
  },
  {
    id: 'strings-intro',
    en: `Default for any scalar. Stored internally as \`Value::String\`. The value
is whatever follows \`:\` after trimming.`,
    ru: `Дефолт для любого скаляра. Внутри хранятся как \`Value::String\`.
Значение — это всё, что идёт после \`:\`, после обрезки пробелов.`,
    zh: `任何标量的默认类型。在内部以 \`Value::String\` 存放。值就是 \`:\` 之后
经过空白修剪的内容。`,
  },
  {
    id: 'strings-snippet',
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
    id: 'numbers-heading',
    en: `### Numbers`,
    ru: `### Числа`,
    zh: `### 数字`,
  },
  {
    id: 'numbers-typing',
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
    common: `\`\`\`text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
\`\`\``,
  },
  {
    id: 'numbers-lossy-note',
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
    en: `### Booleans: \`true\` / \`false\``,
    ru: `### Булевы: \`true\` / \`false\``,
    zh: `### 布尔:\`true\` / \`false\``,
  },
  {
    id: 'booleans-strictness',
    en: `Strict lowercase. Anything else is a string.`,
    ru: `Строго нижний регистр. Всё остальное — строка.`,
    zh: `严格小写。其它写法都是字符串。`,
  },
  {
    id: 'booleans-snippet',
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
    en: `### Null: \`null\``,
    ru: `### Null: \`null\``,
    zh: `### Null:\`null\``,
  },
  {
    id: 'null-mapping',
    en: `Strict lowercase. Matches \`Option::None\` on the Rust side, as well as
\`()\` for unit.`,
    ru: `Строго нижний регистр. На стороне Rust соответствует \`Option::None\`,
а также \`()\` для unit.`,
    zh: `严格小写。对应 Rust 侧的 \`Option::None\`,以及 unit 类型 \`()\`。`,
  },
  {
    id: 'null-snippet',
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
    id: 'null-serialization',
    en: `When serializing, \`Option::None\` is emitted as \`null\`. Suppress with
\`#[serde(skip_serializing_if = "Option::is_none")]\` if you prefer the
field absent.`,
    ru: `При сериализации \`Option::None\` эмитится как \`null\`. Подавить можно
через \`#[serde(skip_serializing_if = "Option::is_none")]\`, если вы
предпочитаете, чтобы поля не было вовсе.`,
    zh: `序列化时,\`Option::None\` 会输出为 \`null\`。若你希望该字段干脆缺省,
可以加上 \`#[serde(skip_serializing_if = "Option::is_none")]\`。`,
  },
  {
    id: 'empty-compounds-heading',
    en: `### Empty object / empty array`,
    ru: `### Пустой объект / пустой массив`,
    zh: `### 空对象 / 空数组`,
  },
  {
    id: 'empty-compounds-only-inline',
    en: `The **only** inline compound values allowed — nothing to separate, no
commas needed.`,
    ru: `**Единственные** allowed inline compound-значения — разделять нечего,
запятые не нужны.`,
    zh: `**唯一**允许的内联复合值——没什么要分隔,不需要逗号。`,
  },
  {
    id: 'empty-compounds-snippet',
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
    en: `### Keyword-like strings need \`::\``,
    ru: `### Ключеподобные строки требуют \`::\``,
    zh: `### 与关键字同形的字符串需要 \`::\``,
  },
  {
    id: 'literal-strings-rationale',
    en: `If a string's content happens to equal a keyword (\`true\`, \`false\`,
\`null\`) or begin with \`{\` or \`[\`, the **serializer emits \`::\`
automatically** so the round-trip is lossless. On the writing side you
do the same:`,
    ru: `Если содержимое строки совпадает с ключевым словом (\`true\`, \`false\`,
\`null\`) или начинается с \`{\` или \`[\`, **сериализатор автоматически
эмитит \`::\`**, чтобы round-trip был без потерь. На стороне записи
поступайте так же:`,
    zh: `若字符串内容恰好等于关键字(\`true\`、\`false\`、\`null\`),或以 \`{\`、
\`[\` 开头,**序列化器会自动输出 \`::\`**,以保证 round-trip 无损。
写入端请用同样的方式:`,
  },
  {
    id: 'literal-strings-snippet',
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
    en: `## Compound values are multi-line`,
    ru: `## Составные значения — многострочные`,
    zh: `## 复合值是多行的`,
  },
  {
    id: 'compounds-multiline-rule',
    en: `Non-empty \`{ ... }\` / \`[ ... ]\` **must** span multiple lines, with the
closing bracket on its own line. \`x: { a: 1 }\` and \`x: [1, 2, 3]\` are
rejected with a clear error — Ktav has no comma-separation rules and
no escape mechanism for them.`,
    ru: `Непустые \`{ ... }\` / \`[ ... ]\` **обязаны** занимать несколько строк,
с закрывающей скобкой на отдельной строке. \`x: { a: 1 }\` и
\`x: [1, 2, 3]\` отклоняются с ясной ошибкой — в Ktav нет правил
разделения запятыми и нет механизма escape для них.`,
    zh: `非空的 \`{ ... }\` / \`[ ... ]\` **必须**跨越多行,闭合括号独占一行。
\`x: { a: 1 }\` 与 \`x: [1, 2, 3]\` 会被以清晰的错误拒绝——Ktav 没有
逗号分隔的规则,也没有针对它们的转义机制。`,
  },
  {
    id: 'compounds-rejected-snippet-part1',
    common: `\`\`\`text
## rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]`,
  },
  {
    id: 'compounds-accepted-snippet-part2',
    common: `## accepted — multi-line form
server: {
    host: 127.0.0.1
    port: 8080
}`,
  },
  {
    id: 'compounds-accepted-snippet-part3',
    common: `tags: [
    primary
    eu
    prod
]
\`\`\``,
  },
  {
    id: 'rust-usage-heading',
    en: `## Using it from Rust`,
    ru: `## Использование из Rust`,
    zh: `## 在 Rust 中使用`,
  },
  {
    id: 'rust-usage-serde-intro',
    en: `Ktav is serde-native. Any type implementing \`Serialize\` / \`Deserialize\`
(including \`#[derive]\`-generated ones) round-trips through Ktav out of
the box.`,
    ru: `Ktav — serde-нативный. Любой тип, реализующий \`Serialize\` /
\`Deserialize\` (включая сгенерированные через \`#[derive]\`),
round-trip-ится через Ktav из коробки.`,
    zh: `Ktav 原生支持 serde。任何实现了 \`Serialize\` / \`Deserialize\` 的类型
(包括 \`#[derive]\` 生成的)都可以开箱即用地通过 Ktav 完成
round-trip。`,
  },
  {
    id: 'parse-heading',
    en: `### Parse — decode straight into a typed struct`,
    ru: `### Парсинг — декод сразу в типизированную структуру`,
    zh: `### 解析 —— 直接解码到类型化结构体`,
  },
  {
    id: 'parse-snippet-imports',
    common: `\`\`\`rust
use serde::{Deserialize, Serialize};`,
  },
  {
    id: 'parse-snippet-db-struct',
    common: `#[derive(Debug, Deserialize, Serialize)]
struct Db { host: String, timeout: u32 }`,
  },
  {
    id: 'parse-snippet-config-struct',
    common: `#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port:    u16,
    ratio:   f64,
    tls:     bool,
    tags:    Vec<String>,
    db:      Db,
}`,
  },
  {
    id: 'parse-snippet-src-const',
    common: `const SRC: &str = "\\
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
";`,
  },
  {
    id: 'parse-snippet-decode',
    en: `let cfg: Config = ktav::from_str(SRC)?;
println!("port={} db.host={}", cfg.port, cfg.db.host);
\`\`\``,
    ru: `let cfg: Config = ktav::from_str(SRC)?;
println!("port={} db.host={}", cfg.port, cfg.db.host);
\`\`\``,
    zh: `let cfg: Config = ktav::from_str(SRC)?;
println!("port={} db.host={}", cfg.port, cfg.db.host);
\`\`\``,
  },
  {
    id: 'walk-heading',
    en: `### Walk — match on the dynamic \`Value\` enum`,
    ru: `### Обход — match по динамическому enum \`Value\``,
    zh: `### 遍历 —— 在动态 \`Value\` 枚举上 match`,
  },
  {
    id: 'walk-snippet-import',
    common: `\`\`\`rust
use ktav::value::Value;`,
  },
  {
    id: 'walk-snippet-parse-top',
    en: `let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("top is always an object") };`,
    ru: `let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("top — всегда object") };`,
    zh: `let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("顶层始终是 object") };`,
  },
  {
    id: 'walk-snippet-loop',
    common: `for (k, v) in top {
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
    en: `### Build & render — construct a document in code`,
    ru: `### Билд + рендер — собираем документ в коде`,
    zh: `### 构建并渲染 —— 用代码搭建文档`,
  },
  {
    id: 'build-render-snippet-import',
    common: `\`\`\`rust
use ktav::value::{ObjectMap, Value};`,
  },
  {
    id: 'build-render-snippet-inserts',
    common: `let mut top = ObjectMap::default();
top.insert("name".into(),  Value::String("frontend".into()));
top.insert("port".into(),  Value::Integer("8443".into()));
top.insert("tls".into(),   Value::Bool(true));
top.insert("ratio".into(), Value::Float("0.95".into()));
top.insert("notes".into(), Value::Null);`,
  },
  {
    id: 'build-render-snippet-render',
    common: `let text = ktav::render::render(&Value::Object(top))?;
\`\`\``,
  },
  {
    id: 'build-render-prefer-serde',
    en: `For typical app use prefer the serde path — \`ktav::to_string(&cfg)\` —
and reach for \`Value\` only when the schema is dynamic.`,
    ru: `В обычных сценариях используйте serde-путь — \`ktav::to_string(&cfg)\`.
К \`Value\` обращайтесь только когда схема динамическая.`,
    zh: `通常请走 serde 路径(\`ktav::to_string(&cfg)\`);仅在 schema
是动态的时候才需要直接操作 \`Value\`。`,
  },
  {
    id: 'entry-points',
    en: `Four public entry points: [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav) for reading, [\`to_string\`](https://docs.rs/ktav) /
[\`to_file\`](https://docs.rs/ktav) for writing. A complete runnable
example lives in [\`examples/basic.rs\`](examples/basic.rs).`,
    ru: `Четыре публичных entry point-а: [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav) — для чтения,
[\`to_string\`](https://docs.rs/ktav) / [\`to_file\`](https://docs.rs/ktav) —
для записи. Полный запускаемый пример — в
[\`examples/basic.rs\`](examples/basic.rs).`,
    zh: `四个公共入口:读取用 [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav),写入用
[\`to_string\`](https://docs.rs/ktav) / [\`to_file\`](https://docs.rs/ktav)。
完整可运行示例:[\`examples/basic.rs\`](examples/basic.rs)。`,
  },
  {
    id: 'errors-heading',
    en: `### Inspect errors — structured variants for tooling`,
    ru: `### Анализ ошибок — структурированные варианты для тулинга`,
    zh: `### 检视错误 —— 给工具链使用的结构化变体`,
  },
  {
    id: 'errors-structured-intro',
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
    id: 'errors-snippet-import',
    common: `\`\`\`rust
use ktav::{parse, Error, ErrorKind};`,
  },
  {
    id: 'errors-snippet-match',
    en: `let src = "port: 80\\nport: 443\\n";
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
    ru: `let src = "port: 80\\nport: 443\\n";
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
    zh: `let src = "port: 80\\nport: 443\\n";
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
    id: 'errors-variants-list',
    en: `Variants: \`MissingSeparatorSpace\`, \`InvalidTypedScalar\`, \`DuplicateKey\`,
\`KeyPathConflict\`, \`EmptyKey\`, \`InvalidKey\`, \`UnclosedCompound\`,
\`UnbalancedBracket\`, \`InlineNonEmptyCompound\`, \`MissingSeparator\`,
\`LossyScalar\` (strict mode only, see below), \`Other\`. The enum is
\`#[non_exhaustive]\` — always include a \`_ =>\` arm. \`Error::line()\` /
\`Error::span()\` are convenience accessors when the variant doesn't
matter. The \`Display\` impl produces the same human-readable string the
legacy \`Error::Syntax(_)\` did, so existing string-based callers keep
working.`,
    ru: `Варианты: \`MissingSeparatorSpace\`, \`InvalidTypedScalar\`, \`DuplicateKey\`,
\`KeyPathConflict\`, \`EmptyKey\`, \`InvalidKey\`, \`UnclosedCompound\`,
\`UnbalancedBracket\`, \`InlineNonEmptyCompound\`, \`MissingSeparator\`,
\`LossyScalar\` (только в строгом режиме, см. ниже), \`Other\`. Enum помечен
\`#[non_exhaustive]\` — обязателен arm \`_ =>\`. \`Error::line()\` /
\`Error::span()\` — convenience accessors для случаев когда вариант
неважен. \`Display\` выдаёт те же читаемые строки, что и legacy
\`Error::Syntax(_)\`, поэтому существующие string-based вызывающие
работают без изменений.`,
    zh: `变体:\`MissingSeparatorSpace\`、\`InvalidTypedScalar\`、\`DuplicateKey\`、
\`KeyPathConflict\`、\`EmptyKey\`、\`InvalidKey\`、\`UnclosedCompound\`、
\`UnbalancedBracket\`、\`InlineNonEmptyCompound\`、\`MissingSeparator\`、
\`LossyScalar\`（仅严格模式，见下文）、\`Other\`。该枚举标注
\`#[non_exhaustive]\` —— 必须包含 \`_ =>\` 分支。变体不重要时可使用便捷
访问器 \`Error::line()\` / \`Error::span()\`。\`Display\` 输出的字符串与
遗留 \`Error::Syntax(_)\` 完全一致,因此原有基于字符串的调用方无需修改
即可继续工作。`,
  },
  {
    id: 'errors-example-pointer',
    en: `A complete runnable example walks all variants:
[\`examples/errors.rs\`](examples/errors.rs) — \`cargo run --example errors\`.`,
    ru: `Полный запускаемый пример проходит по всем вариантам:
[\`examples/errors.rs\`](examples/errors.rs) — \`cargo run --example errors\`.`,
    zh: `完整可运行示例遍历所有变体:
[\`examples/errors.rs\`](examples/errors.rs) —— \`cargo run --example errors\`。`,
  },
  {
    id: 'strict-mode-heading',
    en: `### Strict mode — catch silently canonicalised numbers`,
    ru: `### Строгий режим — ловим молча канонизированные числа`,
    zh: `### 严格模式 —— 捕获被静默规范化的数字`,
  },
  {
    id: 'strict-mode-intro',
    en: `Types are inferred from a scalar's lexical form, and inferred numbers
are canonicalised: \`version: 1.10\` parses as \`Float(1.1)\` and
\`zip: 01234\` as \`Integer(1234)\`. The default \`parse()\` does this
silently, so writing the document back out rewrites it.`,
    ru: `Типы выводятся по лексической форме скаляра, а выведенные числа
канонизируются: \`version: 1.10\` разбирается как \`Float(1.1)\`, а
\`zip: 01234\` — как \`Integer(1234)\`. Обычный \`parse()\` делает это молча,
поэтому обратная запись документа его переписывает.`,
    zh: `类型由标量的词法形式推断，且推断出的数字会被规范化：\`version: 1.10\`
解析为 \`Float(1.1)\`，\`zip: 01234\` 解析为 \`Integer(1234)\`。默认的
\`parse()\` 会静默完成这一过程，因此把文档写回去就会改写它。`,
  },
  {
    id: 'strict-mode-rejects',
    en: `\`parse_strict()\` rejects such **lossy scalars** instead:`,
    ru: `\`parse_strict()\` вместо этого отвергает такие **скаляры с потерей**:`,
    zh: `\`parse_strict()\` 则拒绝这类**有损标量**：`,
  },
  {
    id: 'strict-mode-snippet-imports',
    common: `\`\`\`rust
use ktav::{parse, parse_strict, Error, ErrorKind};`,
  },
  {
    id: 'strict-mode-snippet-src',
    common: `let src = "zip: 01234\\n";`,
  },
  {
    id: 'strict-mode-snippet-parse-ok',
    en: `assert!(parse(src).is_ok());                 // Integer(1234) — leading zero gone`,
    ru: `assert!(parse(src).is_ok());                 // Integer(1234) — ведущий ноль потерян`,
    zh: `assert!(parse(src).is_ok());                 // Integer(1234) —— 前导零丢失`,
  },
  {
    id: 'strict-mode-snippet-match',
    en: `match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("expected LossyScalar, got {other:?}"),
}
\`\`\``,
    ru: `match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("ожидался LossyScalar, получено {other:?}"),
}
\`\`\``,
    zh: `match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("期望 LossyScalar，实际为 {other:?}"),
}
\`\`\``,
  },
  {
    id: 'strict-mode-fix-options',
    en: `Fix either by appending \`::\` to keep the value a String
(\`zip:: 01234\`) or by writing the canonical number. Any document
\`parse_strict()\` accepts yields exactly the same \`Value\` tree as
\`parse()\`, so strict mode is a validation gate, not a different
dialect. The serde path (\`from_str\`) has no strict variant yet.`,
    ru: `Исправить можно либо дописав \`::\`, чтобы значение осталось строкой
(\`zip:: 01234\`), либо записав число в канонической форме. Любой
документ, принятый \`parse_strict()\`, даёт ровно то же дерево \`Value\`,
что и \`parse()\` — строгий режим это проверочный шлюз, а не другой
диалект. У serde-пути (\`from_str\`) строгого варианта пока нет.`,
    zh: `修复方式有两种：追加 \`::\` 让该值保持字符串（\`zip:: 01234\`），或直接
写成规范数字。凡是 \`parse_strict()\` 接受的文档，其产生的 \`Value\` 树
与 \`parse()\` 完全相同 —— 严格模式是一道校验闸门，而非另一种方言。
serde 路径（\`from_str\`）目前尚无严格模式变体。`,
  },
  {
    id: 'stream-parse-heading',
    en: `### Stream parse — events without an intermediate tree`,
    ru: `### Stream-парсинг — события без промежуточного дерева`,
    zh: `### 流式解析 —— 不构建中间树的事件流`,
  },
  {
    id: 'stream-parse-intro',
    en: `\`parse_events\` invokes a callback for each parse event, with strings
borrowed directly into the input buffer — no allocation per event, no
intermediate \`Value\` tree. Useful when you don't need the full document:
counting keys, streaming to another format, building a custom shape.`,
    ru: `\`parse_events\` вызывает callback на каждое событие парсинга, со
строками заимствующимися напрямую из input-буфера — без аллокации на
событие, без промежуточного \`Value\`-дерева. Полезно когда полный
документ не нужен: подсчёт ключей, стриминг в другой формат, построение
custom-shape-а.`,
    zh: `\`parse_events\` 对每个解析事件调用回调,字符串直接从 input 缓冲区
借用 —— 不在每个事件上分配,也不构建中间 \`Value\` 树。在不需要完整
文档时很有用:统计键、流式转换到其他格式、构建自定义形状。`,
  },
  {
    id: 'stream-parse-snippet-imports',
    common: `\`\`\`rust
use ktav::{parse_events, ParseEvent};`,
  },
  {
    id: 'stream-parse-snippet-keys',
    common: `let src = "port: 8080\\nhost: example.com\\n";
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
    id: 'stream-parse-root-events',
    en: `The root is \`BeginObject\`/\`EndObject\` or \`BeginArray\`/\`EndArray\`
depending on the document's first content line (an Object here, since
\`port: 8080\` is a pair); nested compounds bracket their contents the
same way. \`ParseEvent\` is \`#[non_exhaustive]\`. A complete runnable
example with depth tracking
and a pretty-printer:
[\`examples/events.rs\`](examples/events.rs) — \`cargo run --example events\`.`,
    ru: `Корень — \`BeginObject\`/\`EndObject\` или \`BeginArray\`/\`EndArray\`, в
зависимости от первой содержательной строки документа (здесь Object,
так как \`port: 8080\` — это пара); вложенные компаунды обрамляют своё
содержимое так же. \`ParseEvent\` помечен
\`#[non_exhaustive]\`. Полный запускаемый пример с tracking-ом глубины
и pretty-print-ом:
[\`examples/events.rs\`](examples/events.rs) — \`cargo run --example events\`.`,
    zh: `根据文档第一条内容行的形状,根节点是 \`BeginObject\`/\`EndObject\` 或
\`BeginArray\`/\`EndArray\`(此处因为 \`port: 8080\` 是一个 pair,所以是
Object);嵌套复合值以同样方式括起自身内容。\`ParseEvent\` 标注
\`#[non_exhaustive]\`。完整的、带深度跟踪
和漂亮打印的可运行示例:
[\`examples/events.rs\`](examples/events.rs) —— \`cargo run --example events\`。`,
  },
  {
    id: 'stream-numbers-heading',
    en: `### Numbers`,
    ru: `### Числа`,
    zh: `### 数字`,
  },
  {
    id: 'stream-numbers-notes',
    en: `Rust numeric types (\`u8\`..\`u128\`, \`i8\`..\`i128\`, \`usize\`, \`isize\`, \`f32\`,
\`f64\`) serialize to Ktav as bare numbers: \`port: 8080\`, \`ratio: 0.5\`.
Coming back, a bare integer/decimal body deserializes straight into
the target numeric type; a value that arrived as a string (e.g. forced
with \`::\`) is still accepted via \`FromStr\`. \`NaN\` / \`±Infinity\` are
rejected by the serializer (Ktav does not represent them).`,
    ru: `Числовые Rust-типы (\`u8\`..\`u128\`, \`i8\`..\`i128\`, \`usize\`, \`isize\`, \`f32\`,
\`f64\`) сериализуются в Ktav как голые числа: \`port: 8080\`,
\`ratio: 0.5\`. На обратном пути голое целое/десятичное тело
десериализуется прямо в нужный числовой тип; значение, пришедшее
строкой (например, форсированное через \`::\`), по-прежнему принимается
через \`FromStr\`. \`NaN\` / \`±Infinity\` отвергаются сериализатором
(Ktav их не представляет).`,
    zh: `Rust 数值类型(\`u8\`..\`u128\`、\`i8\`..\`i128\`、\`usize\`、\`isize\`、\`f32\`、
\`f64\`)会以裸数字序列化到 Ktav:\`port: 8080\`、\`ratio: 0.5\`。回程时,
裸整数/小数 body 直接反序列化为目标数值类型;以字符串形式到达的值
(例如用 \`::\` 强制的)仍通过 \`FromStr\` 接受。\`NaN\` / \`±Infinity\` 会被
序列化器拒绝(Ktav 不表示这些值)。`,
  },
  {
    id: 'examples-heading',
    en: `## Examples: Ktav → JSON5`,
    ru: `## Примеры: Ktav → JSON5`,
    zh: `## 示例:Ktav → JSON5`,
  },
  {
    id: 'examples-json5-rationale',
    en: `JSON5 is on the right because it reads like ordinary JavaScript, allows
comments, and shows exactly what the parser produces.`,
    ru: `JSON5 справа потому, что читается как обычный JavaScript, допускает
комментарии и показывает ровно то, что производит парсер.`,
    zh: `右侧使用 JSON5,因为它读起来像普通 JavaScript,允许注释,并且
能完整展示解析器产出的结果。`,
  },
  {
    id: 'example-1-scalars-heading',
    en: `### 1. Scalars`,
    ru: `### 1. Скаляры`,
    zh: `### 1. 标量`,
  },
  {
    id: 'example-1-scalars-snippet',
    common: `\`\`\`text
name: Russia
port: 20082
\`\`\`
\`\`\`json5
{
  name: "Russia",
  port: 20082
}
\`\`\``,
  },
  {
    id: 'example-1-scalars-note',
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
    en: `### 2. Dotted keys = nested objects`,
    ru: `### 2. Точечные ключи = вложенные объекты`,
    zh: `### 2. 点分键 = 嵌套对象`,
  },
  {
    id: 'example-2-dotted-keys-snippet',
    common: `\`\`\`text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
\`\`\`
\`\`\`json5
{
  server: { host: "127.0.0.1", port: 8080 },
  app: { debug: true }
}
\`\`\``,
  },
  {
    id: 'example-2-dotted-keys-note',
    en: `Any depth works. The full address is on every line.`,
    ru: `Любая глубина работает. Полный адрес — на каждой строке.`,
    zh: `深度任意。完整地址写在每一行上。`,
  },
  {
    id: 'example-3-nested-object-heading',
    en: `### 3. Nested object as a value`,
    ru: `### 3. Вложенный объект как значение`,
    zh: `### 3. 作为值的嵌套对象`,
  },
  {
    id: 'example-3-nested-object-snippet',
    common: `\`\`\`text
server: {
    host: 127.0.0.1
    port: 8080
    endpoints.api: /v1
    endpoints.admin: /admin
}
\`\`\`
\`\`\`json5
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
    en: `### 4. Array of scalars`,
    ru: `### 4. Массив скаляров`,
    zh: `### 4. 标量数组`,
  },
  {
    id: 'example-4-array-of-scalars-snippet',
    common: `\`\`\`text
banned_patterns: [
    .*\\.onion:\\d+
    .*:25
]
\`\`\`
\`\`\`json5
{
  banned_patterns: [".*\\\\.onion:\\\\d+", ".*:25"]
}
\`\`\``,
  },
  {
    id: 'example-5-array-of-objects-heading',
    en: `### 5. Array of objects`,
    ru: `### 5. Массив объектов`,
    zh: `### 5. 对象数组`,
  },
  {
    id: 'example-5-array-of-objects-snippet',
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
\`\`\`
\`\`\`json5
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
    en: `### 6. Arbitrary nesting`,
    ru: `### 6. Произвольная вложенность`,
    zh: `### 6. 任意层嵌套`,
  },
  {
    id: 'example-6-arbitrary-nesting-rule',
    en: `Every compound value spans multiple lines (single-line \`{ ... }\` / \`[ ... ]\`
with contents is not accepted — only the empty forms \`{}\` / \`[]\` are
inline). Nest as deep as needed:`,
    ru: `Каждое составное значение занимает несколько строк (однострочные
\`{ ... }\` / \`[ ... ]\` с содержимым не принимаются — инлайн разрешены
только пустые формы \`{}\` / \`[]\`). Вкладывайте сколько угодно:`,
    zh: `每个复合值都跨越多行(带内容的单行 \`{ ... }\` / \`[ ... ]\` 不被接受
——只有空形式 \`{}\` / \`[]\` 允许写作内联)。想嵌多深就嵌多深:`,
  },
  {
    id: 'example-6-arbitrary-nesting-snippet',
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
    en: `### 7. Literal strings: \`::\``,
    ru: `### 7. Литеральные строки: \`::\``,
    zh: `### 7. 字面量字符串:\`::\``,
  },
  {
    id: 'example-7-literal-strings-rationale',
    en: `Some values would otherwise be parsed as compound (because they start
with \`{\` or \`[\`): regular expressions, IPv6 addresses, template
placeholders. The double-colon \`::\` flags them as "raw string, do not
parse further."`,
    ru: `Некоторые значения иначе были бы разобраны как compound (потому что
начинаются с \`{\` или \`[\`): регулярные выражения, IPv6-адреса,
placeholders шаблонов. Двойное двоеточие \`::\` помечает их как «сырая
строка, не разбирать дальше».`,
    zh: `某些值如果不加标记,会被当作复合值解析(因为以 \`{\` 或 \`[\` 开头):
正则、IPv6 地址、模板占位符。双冒号 \`::\` 把它们标记为「原样字符串,
不要继续解析」。`,
  },
  {
    id: 'example-7-literal-strings-snippet',
    common: `\`\`\`text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl`,
  },
  {
    id: 'example-7-literal-strings-array-snippet',
    common: `hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
\`\`\`
\`\`\`json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
\`\`\``,
  },
  {
    id: 'example-7-literal-strings-note',
    en: `For pairs the marker sits between key and value; for array items it
stands at the start of the line. **Serialization emits \`::\`
automatically** when a string value begins with \`{\` or \`[\`, so
round-tripping regexes and IPv6 addresses just works.`,
    ru: `Для пар маркер стоит между ключом и значением; для элементов массива —
в начале строки. **Сериализация эмитит \`::\` автоматически**, когда
строковое значение начинается с \`{\` или \`[\`, так что round-trip
regex-ов и IPv6-адресов просто работает.`,
    zh: `对于键值对,该标记位于键和值之间;对于数组元素,它位于行首。
**当字符串值以 \`{\` 或 \`[\` 开头时,序列化会自动输出 \`::\`**,所以
正则与 IPv6 地址的 round-trip 自然工作。`,
  },
  {
    id: 'example-8-comments-heading',
    en: `### 8. Comments`,
    ru: `### 8. Комментарии`,
    zh: `### 8. 注释`,
  },
  {
    id: 'example-8-comments-snippet-part1',
    common: `\`\`\`text
## top-level comment
port: 8080`,
  },
  {
    id: 'example-8-comments-snippet-part2',
    common: `items: [
    ## this comment does not break the array
    a
    b
]
\`\`\``,
  },
  {
    id: 'example-8-comments-explanation',
    en: `Comments are full lines starting with \`#\`. Inline comments are not
supported — they get confused with the value too easily.`,
    ru: `Комментарии — целые строки, начинающиеся с \`#\`. Inline-комментарии
не поддерживаются — их слишком легко спутать со значением.`,
    zh: `注释为整行,以 \`#\` 开头。不支持行内注释——太容易与值混淆。`,
  },
  {
    id: 'example-9-multiline-strings-heading',
    en: `### 9. Multi-line strings: \`( ... )\` and \`(( ... ))\``,
    ru: `### 9. Многострочные строки: \`( ... )\` и \`(( ... ))\``,
    zh: `### 9. 多行字符串:\`( ... )\` 与 \`(( ... ))\``,
  },
  {
    id: 'example-9-multiline-strings-intro',
    en: `Values that span multiple lines go inside parentheses. The opening and
closing lines are NOT part of the value.`,
    ru: `Значения, занимающие несколько строк, заключаются в круглые скобки.
Открывающая и закрывающая строки НЕ входят в значение.`,
    zh: `跨越多行的值放在圆括号里。开起行与关闭行**不**属于值。`,
  },
  {
    id: 'example-9-parens-form',
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
    id: 'example-9-parens-snippet',
    common: `\`\`\`text
body: (
    {
      "qwe": 1
    }
)
\`\`\`
\`\`\`json5
{ body: "{\\n  \\"qwe\\": 1\\n}" }
\`\`\``,
  },
  {
    id: 'example-9-double-parens-form',
    en: `\`((\` ... \`))\` — verbatim: every character between the markers ends up in
the value, including leading whitespace:`,
    ru: `\`((\` ... \`))\` — побайтово: каждый символ между маркерами попадает в
значение, включая ведущие пробелы:`,
    zh: `\`((\` ... \`))\` —— 逐字节保留:开始与结束标记之间的每一个字符都
进入值,包括前导空白:`,
  },
  {
    id: 'example-9-double-parens-snippet',
    common: `\`\`\`text
sig: ((
  -----BEGIN-----
  QUJDRA==
  -----END-----
))
\`\`\`
\`\`\`json5
{ sig: "  -----BEGIN-----\\n  QUJDRA==\\n  -----END-----" }
\`\`\``,
  },
  {
    id: 'example-9-block-content-rule',
    en: `Inside a block, \`{\` / \`[\` / \`#\` are just content — **no compound parsing,
no comment skipping**. The only special sequence is the terminator on
its own line.`,
    ru: `Внутри блока \`{\` / \`[\` / \`#\` — просто содержимое, **никакого разбора
compound, никакого пропуска комментариев**. Единственная особая
последовательность — терминатор на отдельной строке.`,
    zh: `在块内,\`{\` / \`[\` / \`#\` 只是内容——**不做复合解析,不跳过注释**。
唯一的特殊序列就是独占一行的终止符。`,
  },
  {
    id: 'example-9-empty-inline-form',
    en: `Empty inline form: \`key: ()\` or \`key: (())\` — both yield the empty
string (same as \`key:\`).`,
    ru: `Пустая инлайн-форма: \`key: ()\` или \`key: (())\` — обе дают пустую
строку (то же, что \`key:\`).`,
    zh: `空的内联形式:\`key: ()\` 或 \`key: (())\`——都会产生空字符串(等同
于 \`key:\`)。`,
  },
  {
    id: 'example-9-serialization-rules',
    en: `Serialization: a string is emitted on a single line only when it has
no \`\\n\`, no leading/trailing whitespace, and no control byte other
than \`TAB\`. Anything else — including a plain string with a stray edge
space — takes a multi-line form: verbatim \`(( ... ))\` when the content
has edge whitespace that stripped would alter, stripped \`( ... )\`
otherwise; whichever the
writer picks, the round-trip is byte-for-byte lossless.`,
    ru: `Сериализация: строка выходит в одну строку, только если в ней нет
\`\\n\`, нет ведущих/замыкающих пробельных символов и нет управляющих
байтов кроме \`TAB\`. Всё остальное — включая обычную строку со
случайным пробелом на краю — уходит в многострочную форму: дословную
\`(( ... ))\`, если у содержимого есть крайние пробельные символы,
которые снятие отступа исказит, иначе форму со снятым отступом
\`( ... )\`; какую бы форму writer ни выбрал, round-trip
байт-в-байт без потерь.`,
    zh: `序列化:字符串只有在不含 \`\\n\`、没有首尾空白、且不含除 \`TAB\`
以外的控制字节时才会以单行形式输出。其他情况——哪怕只是普通字符串
边缘多了一个空格——都会采用多行形式:当内容含有会被去缩进破坏的
边缘空白时采用逐字形式 \`(( ... ))\`,否则采用去缩进形式
\`( ... )\`;无论 writer 选择哪种形式,
round-trip 都是字节级无损的。`,
  },
  {
    id: 'example-9-which-form',
    en: `Which block form you get depends on *where* the whitespace is. As of
0.7 the stripped form strips trailing whitespace from every content
line (§ 5.6), so a trailing space forces the verbatim form, which
preserves it byte-for-byte:`,
    ru: `Какая именно блочная форма — зависит от того, *где* пробелы. С 0.7
форма со снятым отступом срезает замыкающий пробельный хвост с каждой
строки содержимого (§ 5.6), так что замыкающий пробел вынуждает взять
дословную форму, которая сохраняет его байт-в-байт:`,
    zh: `具体采用哪种块形式,取决于空白出现在**哪一侧**。从 0.7 起,去缩进
形式会剥除每行内容末尾的空白(§ 5.6),因此末尾空格必须使用逐字
形式,由它逐字节保留:`,
  },
  {
    id: 'example-9-hunter2-snippet',
    common: `\`\`\`json5
{ password: "hunter2 " }
\`\`\`
\`\`\`text
password: ((
hunter2 
))
\`\`\``,
  },
  {
    id: 'example-9-leading-whitespace-note',
    en: `Leading whitespace — and, since 0.7, trailing whitespace — is what
forces the verbatim form: stripping would eat the leading indent:`,
    ru: `Ведущие пробелы — а с 0.7 ещё и замыкающие — вынуждают взять дословную
форму: снятие отступа съело бы ведущий отступ:`,
    zh: `前导空白——从 0.7 起还包括末尾空白——必须使用逐字形式:去缩进会
把前导缩进吃掉:`,
  },
  {
    id: 'example-9-indent-snippet',
    common: `\`\`\`json5
{ indent: "  padded" }
\`\`\`
\`\`\`text
indent: ((
  padded
))
\`\`\``,
  },
  {
    id: 'example-9-roundtrip-note',
    en: `Either way, reading it back gives you the original bytes.`,
    ru: `В обоих случаях при обратном чтении получите исходные байты.`,
    zh: `两种情况读回来都能得到原始字节。`,
  },
  {
    id: 'example-9-limitation',
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
    en: `### 10. Empty compounds`,
    ru: `### 10. Пустые compound-ы`,
    zh: `### 10. 空复合值`,
  },
  {
    id: 'example-10-empty-compounds-snippet',
    common: `\`\`\`text
meta: {}
tags: []
\`\`\``,
  },
  {
    id: 'example-10-empty-compounds-note',
    en: `Inline empty is allowed. Anything with contents must span multiple
lines, and the closing \`}\` / \`]\` must sit on its own line.`,
    ru: `Inline-пустой разрешён. Всё с содержимым обязано занимать несколько
строк, и закрывающий \`}\` / \`]\` обязан стоять на отдельной строке.`,
    zh: `允许内联空。带内容的值必须跨越多行,且闭合的 \`}\` / \`]\` 必须独占
一行。`,
  },
  {
    id: 'example-11-enums-heading',
    en: `### 11. Enums`,
    ru: `### 11. Enum-ы`,
    zh: `### 11. 枚举`,
  },
  {
    id: 'example-11-enums-intro',
    en: `Ktav uses serde's default *externally tagged* enum representation.`,
    ru: `Ktav использует дефолтное *externally tagged* представление enum-ов
serde.`,
    zh: `Ktav 使用 serde 默认的 *externally tagged* 枚举表示形式。`,
  },
  {
    id: 'example-11-enums-rust-snippet-part1',
    common: `\`\`\`rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode { Fast, Slow }`,
  },
  {
    id: 'example-11-enums-rust-snippet-part2',
    common: `#[derive(Serialize, Deserialize)]
enum Action {
    Log(String),
    Count(u32),
}
\`\`\``,
  },
  {
    id: 'example-11-enums-ktav-snippet-part1',
    common: `\`\`\`text
## unit variant — just the name
mode: fast`,
  },
  {
    id: 'example-11-enums-ktav-snippet-part2',
    common: `## newtype variant — single-entry object
action: {
    Log: hello
}
\`\`\``,
  },
  {
    id: 'round-trip-heading',
    common: `## Round-trip`,
  },
  {
    id: 'round-trip-snippet',
    common: `\`\`\`rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
\`\`\``,
  },
  {
    id: 'round-trip-serialization-preserves',
    en: `Serialization preserves:
- **Field order** — \`Value::Object\` is backed by an \`IndexMap\`, so the
  order is whatever serde emits (for structs: declaration order).
- **Literal strings** — values starting with \`{\` or \`[\` are emitted
  with the \`::\` marker.
- **\`None\` fields** — skipped on output; reappear as \`None\` on input
  (via serde's \`Option\` handling).`,
    ru: `Сериализация сохраняет:
- **Порядок полей** — \`Value::Object\` лежит на \`IndexMap\`, так что
  порядок — тот, что эмитит serde (для struct-ов: порядок объявления).
- **Литеральные строки** — значения, начинающиеся с \`{\` или \`[\`,
  эмитятся с маркером \`::\`.
- **Поля \`None\`** — пропускаются на выходе; восстанавливаются как
  \`None\` на входе (через обработку \`Option\` в serde).`,
    zh: `序列化会保持:
- **字段顺序** —— \`Value::Object\` 底层是 \`IndexMap\`,顺序由 serde
  输出决定(对结构体而言:就是声明顺序)。
- **字面量字符串** —— 以 \`{\` 或 \`[\` 开头的值会带 \`::\` 标记输出。
- **\`None\` 字段** —— 输出时跳过;输入时通过 serde 的 \`Option\`
  处理重新出现为 \`None\`。`,
  },
  {
    id: 'architecture-heading',
    en: `## Architecture`,
    ru: `## Архитектура`,
    zh: `## 架构`,
  },
  {
    id: 'architecture-tree',
    common: `\`\`\`
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── render/           — pretty-printer (Value → text)
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error + serde::Error impls
└── lib.rs            — glue: from_str / from_file / to_string / to_file
\`\`\``,
  },
  {
    id: 'architecture-file-conventions',
    en: `Each file holds one exported item; implementation details are private to
their parent module.`,
    ru: `В каждом файле — один экспортируемый элемент; детали реализации
приватны внутри родительского модуля.`,
    zh: `每个文件持有一个导出项;实现细节相对其父模块私有。`,
  },
  {
    id: 'not-do-heading',
    en: `## What Ktav does NOT do — and never will`,
    ru: `## Чего Ktav НЕ делает — и никогда не будет`,
    zh: `## Ktav **不**做、也永远不会做的事`,
  },
  {
    id: 'not-do-list',
    en: `- **Inline non-empty compounds** like \`x: { a: 1, b: 2 }\`. They'd bring
  commas, and commas would bring escaping. Compound values are
  multiline.
- **Anchors / aliases / merge keys** (\`&anchor\`, \`*ref\`, \`<<:\`). Any
  line whose meaning depends on a declaration far away stops being
  self-sufficient. If you want DRY, compose defaults in code.
- **File includes** (\`@include\`, \`!import\`). Write a wrapper in code
  for large configs.
- **Top-level arrays.** The document is always an object.`,
    ru: `- **Inline непустые compound-ы** вроде \`x: { a: 1, b: 2 }\`. Они
  притащили бы запятые, а запятые притащили бы escape. Compound-
  значения многострочны.
- **Якоря / алиасы / merge-ключи** (\`&anchor\`, \`*ref\`, \`<<:\`). Любая
  строка, смысл которой зависит от декларации в отдалённом месте,
  перестаёт быть самодостаточной. Если нужен DRY — композируйте
  defaults в коде.
- **Инклюды файлов** (\`@include\`, \`!import\`). Для больших конфигов
  напишите обёртку в коде.
- **Массивы на верхнем уровне.** Документ всегда — объект.`,
    zh: `- **内联的非空复合值**,比如 \`x: { a: 1, b: 2 }\`。它们会带来逗号,
  逗号又会带来转义。复合值保持多行。
- **锚点 / 别名 / 合并键**(\`&anchor\`、\`*ref\`、\`<<:\`)。任何一行
  若其含义依赖远处的声明,就不再自洽。若需要 DRY,请在代码里
  组合默认值。
- **文件包含**(\`@include\`、\`!import\`)。大型配置请在代码里包一层
  封装。
- **顶层数组。** 文档始终是对象。`,
  },
  {
    id: 'installation-heading',
    en: `## Installation`,
    ru: `## Установка`,
    zh: `## 安装`,
  },
  {
    id: 'installation-toml',
    common: `\`\`\`toml
[dependencies]
ktav = "0.6"
serde = { version = "1", features = ["derive"] }
\`\`\``,
  },
  {
    id: 'support-heading',
    en: `## Support the project`,
    ru: `## Поддержите проект`,
    zh: `## 支持本项目`,
  },
  {
    id: 'support-appeal',
    en: `The author has many ideas that could be broadly useful to IT worldwide —
not limited to Ktav. Realizing them requires funding. If you'd like to
help, please reach out at **phpcraftdream@gmail.com**.`,
    ru: `У автора много идей, которые могут быть полезны IT во всём мире, — и
далеко не только для Ktav. Их реализация требует финансирования. Если
вы хотите помочь — пишите на **phpcraftdream@gmail.com**.`,
    zh: `作者有许多构想,可能对全球 IT 广泛有益——不局限于 Ktav。实现这些
构想需要资金支持。如果您愿意提供帮助,请联系
**phpcraftdream@gmail.com**。`,
  },
  {
    id: 'license-heading',
    en: `## License`,
    ru: `## Лицензия`,
    zh: `## 许可证`,
  },
  {
    id: 'license-text',
    en: `Dual-licensed under **MIT OR Apache-2.0** at your option. See
[LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).`,
    ru: `MIT OR Apache-2.0. См. [LICENSE-MIT](LICENSE-MIT) и [LICENSE-APACHE](LICENSE-APACHE).`,
    zh: `MIT OR Apache-2.0。详见 [LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE)。`,
  },
  {
    id: 'implementations-heading',
    en: `## Other Ktav implementations`,
    ru: `## Другие реализации Ktav`,
    zh: `## 其他 Ktav 实现`,
  },
  {
    id: 'implementations-list',
    en: `- [\`spec\`](https://github.com/ktav-lang/spec) — specification + conformance suite
- [\`csharp\`](https://github.com/ktav-lang/csharp) — C# / .NET (\`dotnet add package Ktav\`)
- [\`golang\`](https://github.com/ktav-lang/golang) — Go (\`go get github.com/ktav-lang/golang\`)
- [\`java\`](https://github.com/ktav-lang/java) — Java / JVM (\`io.github.ktav-lang:ktav\` on Maven Central)
- [\`js\`](https://github.com/ktav-lang/js) — JS / TS (\`npm install @ktav-lang/ktav\`)
- [\`php\`](https://github.com/ktav-lang/php) — PHP (\`composer require ktav-lang/ktav\`)
- [\`python\`](https://github.com/ktav-lang/python) — Python (\`pip install ktav\`)`,
    ru: `- [\`spec\`](https://github.com/ktav-lang/spec) — спецификация + conformance-тесты
- [\`csharp\`](https://github.com/ktav-lang/csharp) — C# / .NET (\`dotnet add package Ktav\`)
- [\`golang\`](https://github.com/ktav-lang/golang) — Go (\`go get github.com/ktav-lang/golang\`)
- [\`java\`](https://github.com/ktav-lang/java) — Java / JVM (\`io.github.ktav-lang:ktav\` на Maven Central)
- [\`js\`](https://github.com/ktav-lang/js) — JS / TS (\`npm install @ktav-lang/ktav\`)
- [\`php\`](https://github.com/ktav-lang/php) — PHP (\`composer require ktav-lang/ktav\`)
- [\`python\`](https://github.com/ktav-lang/python) — Python (\`pip install ktav\`)`,
    zh: `- [\`spec\`](https://github.com/ktav-lang/spec) —— 规范 + 一致性测试套件
- [\`csharp\`](https://github.com/ktav-lang/csharp) —— C# / .NET(\`dotnet add package Ktav\`)
- [\`golang\`](https://github.com/ktav-lang/golang) —— Go(\`go get github.com/ktav-lang/golang\`)
- [\`java\`](https://github.com/ktav-lang/java) —— Java / JVM(\`io.github.ktav-lang:ktav\`,Maven Central)
- [\`js\`](https://github.com/ktav-lang/js) —— JS / TS(\`npm install @ktav-lang/ktav\`)
- [\`php\`](https://github.com/ktav-lang/php) —— PHP(\`composer require ktav-lang/ktav\`)
- [\`python\`](https://github.com/ktav-lang/python) —— Python(\`pip install ktav\`)`,
  },
];
