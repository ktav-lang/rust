# Ktav (כְּתָב)

> Простой формат конфигурации. Формы JSON5, но без кавычек, без
> запятых, с точечными ключами для вложенности. Нативная интеграция
> с `serde`.

**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)

**Спецификация:** этот crate реализует **Ktav 0.1**. Формат версионируется
и поддерживается независимо от crate-а — см.
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) для
канонического документа.

---

## Название

*Ktav* (иврит: **כְּתָב**) означает «письмо, то, что записано» —
нечто зафиксированное в форме, достаточно устойчивой, чтобы смысл не
зависел от того, кто это передаёт дальше. Название подходит буквально:
конфиг-файл *и есть* ktav на диске, а библиотека его читает и отдаёт
живую структуру, ничего не выдумывая по дороге.

## Девиз

> **Будь другом конфига, а не его экзаменатором. Конфиг неидеален —
> но он лучший из возможных.**

Каждое правило локально. Каждая строка либо стоит сама по себе, либо
зависит только от видимых скобок. Никаких ловушек с отступами, никаких
забытых кавычек, никакой арифметики замыкающих запятых.

## Правила

Документ Ktav — это неявный объект верхнего уровня. Внутри любого
объекта — пары; внутри любого массива — элементы.

```text
# comment              — any line starting with '#'
key: value             — scalar pair; key may be a dotted path (a.b.c)
key:: value            — scalar pair; value is ALWAYS a literal string
key: { ... }           — multi-line object; `}` closes on its own line
key: [ ... ]           — multi-line array; `]` closes on its own line
key: {}   /   key: []  — empty compound, inline
key: ( ... )           — multi-line string; common indent stripped
key: (( ... ))         — multi-line string; verbatim (no stripping)
:: value               — inside an array: literal-string item
```

Это весь язык. Никаких запятых, никаких кавычек, никаких escape
внутри самого значения — единственный «escape» это маркер `::`, и он
живёт в разделителе (для пар) или в префиксе строки (для элементов
массива).

## Значения и специальные токены

### Строки

Дефолт для любого скаляра. Внутри хранятся как `Value::String`.
Значение — это всё, что идёт после `:`, после обрезки пробелов.

```text
name: Russia
path: /etc/hosts
greeting: hello world
# `::` принудительно задаёт литеральную строку
pattern:: [a-z]+
```

### Числа

Числа пишутся без кавычек. На уровне `Value` они — строки; serde
разбирает их в целевой Rust-тип (`u16`, `i64`, `f64`, …) через
`FromStr` при десериализации и форматирует через `Display` при
сериализации.

```text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
```

Значение вида `port: abc` парсится нормально *на уровне Ktav* (строка
`"abc"`), но `serde::deserialize` в `u16` вернёт понятный
`ParseError`.

### Булевы: `true` / `false`

Строго нижний регистр. Всё остальное — строка.

```text
# Value::Bool(true)
on: true
# Value::Bool(false)
off: false
# Value::String("True")
capitalized: True
# Value::String("FALSE")
yelling:    FALSE
# Value::String("true")
literal:: true
```

### Null: `null`

Строго нижний регистр. На стороне Rust соответствует `Option::None`,
а также `()` для unit.

```text
# Value::Null
label: null
# Value::String("Null")
capitalized: Null
# Value::String("null")
literal:: null
```

При сериализации `Option::None` эмитится как `null`. Подавить можно
через `#[serde(skip_serializing_if = "Option::is_none")]`, если вы
предпочитаете, чтобы поля не было вовсе.

### Пустой объект / пустой массив

**Единственные** allowed inline compound-значения — разделять нечего,
запятые не нужны.

```text
# пустой объект
meta: {}
# пустой массив
tags: []
```

### Ключеподобные строки требуют `::`

Если содержимое строки совпадает с ключевым словом (`true`, `false`,
`null`) или начинается с `{` или `[`, **сериализатор автоматически
эмитит `::`**, чтобы round-trip был без потерь. На стороне записи
поступайте так же:

```text
# строка "true", а не булево
flag:: true
# строка "null", а не Null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
```

## Составные значения — многострочные

Непустые `{ ... }` / `[ ... ]` **обязаны** занимать несколько строк,
с закрывающей скобкой на отдельной строке. `x: { a: 1 }` и
`x: [1, 2, 3]` отклоняются с ясной ошибкой — в Ktav нет правил
разделения запятыми и нет механизма escape для них.

```text
# rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]

# accepted — multi-line form
server: {
    host: 127.0.0.1
    port: 8080
}

tags: [
    primary
    eu
    prod
]
```

## Использование из Rust

Ktav — serde-нативный. Любой тип, реализующий `Serialize` /
`Deserialize` (включая сгенерированные через `#[derive]`),
round-trip-ится через Ktav из коробки.

### Парсинг — декод сразу в типизированную структуру

```rust
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

const SRC: &str = "\
service: web
port:i 8080
ratio:f 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout:i 30
";

let cfg: Config = ktav::from_str(SRC)?;
println!("port={} db.host={}", cfg.port, cfg.db.host);
```

### Обход — match по динамическому enum `Value`

```rust
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
```

### Билд + рендер — собираем документ в коде

```rust
use ktav::value::{ObjectMap, Value};

let mut top = ObjectMap::default();
top.insert("name".into(),  Value::String("frontend".into()));
top.insert("port".into(),  Value::Integer("8443".into()));
top.insert("tls".into(),   Value::Bool(true));
top.insert("ratio".into(), Value::Float("0.95".into()));
top.insert("notes".into(), Value::Null);

let text = ktav::render::render(&Value::Object(top))?;
```

В обычных сценариях используйте serde-путь — `ktav::to_string(&cfg)`.
К `Value` обращайтесь только когда схема динамическая.

Четыре публичных entry point-а: [`from_str`](https://docs.rs/ktav) /
[`from_file`](https://docs.rs/ktav) — для чтения,
[`to_string`](https://docs.rs/ktav) / [`to_file`](https://docs.rs/ktav) —
для записи. Полный запускаемый пример — в
[`examples/basic.rs`](examples/basic.rs).

### Типизированные маркеры

Числовые Rust-типы (`u8`..`u128`, `i8`..`i128`, `usize`, `isize`, `f32`,
`f64`) сериализуются в Ktav с явными типизированными маркерами:
`port:i 8080`, `ratio:f 0.5`. Десериализация принимает *обе* формы —
и с маркерами, и plain-string; документы, написанные без маркеров,
по-прежнему работают, как и раньше. `NaN` / `±Infinity` отвергаются
сериализатором (Ktav 0.1.0 их не представляет).

## Примеры: Ktav → JSON5

JSON5 справа потому, что читается как обычный JavaScript, допускает
комментарии и показывает ровно то, что производит парсер.

### 1. Скаляры

```text
name: Russia
port: 20082
```
```json5
{
  name: "Russia",
  port: "20082"
}
```

Все скаляры выходят строками на уровне `Value`; числовые / булевые
типы разбираются через serde, когда вы десериализуете в `u16` / `bool`
/ `f64` / …

### 2. Точечные ключи = вложенные объекты

```text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
```
```json5
{
  server: { host: "127.0.0.1", port: "8080" },
  app: { debug: "true" }
}
```

Любая глубина работает. Полный адрес — на каждой строке.

### 3. Вложенный объект как значение

```text
server: {
    host: 127.0.0.1
    port: 8080
    endpoints.api: /v1
    endpoints.admin: /admin
}
```
```json5
{
  server: {
    host: "127.0.0.1",
    port: "8080",
    endpoints: { api: "/v1", admin: "/admin" }
  }
}
```

### 4. Массив скаляров

```text
banned_patterns: [
    .*\.onion:\d+
    .*:25
]
```
```json5
{
  banned_patterns: [".*\\.onion:\\d+", ".*:25"]
}
```

### 5. Массив объектов

```text
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
```
```json5
{
  upstreams: [
    { host: "a.example", port: "1080" },
    { host: "b.example", port: "1080" }
  ]
}
```

### 6. Произвольная вложенность

Каждое составное значение занимает несколько строк (однострочные
`{ ... }` / `[ ... ]` с содержимым не принимаются — инлайн разрешены
только пустые формы `{}` / `[]`). Вкладывайте сколько угодно:

```text
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
```

### 7. Литеральные строки: `::`

Некоторые значения иначе были бы разобраны как compound (потому что
начинаются с `{` или `[`): регулярные выражения, IPv6-адреса,
placeholders шаблонов. Двойное двоеточие `::` помечает их как «сырая
строка, не разбирать дальше».

```text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl

hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
```
```json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
```

Для пар маркер стоит между ключом и значением; для элементов массива —
в начале строки. **Сериализация эмитит `::` автоматически**, когда
строковое значение начинается с `{` или `[`, так что round-trip
regex-ов и IPv6-адресов просто работает.

### 8. Комментарии

```text
# top-level comment
port: 8080

items: [
    # this comment does not break the array
    a
    b
]
```

Комментарии — целые строки, начинающиеся с `#`. Inline-комментарии
не поддерживаются — их слишком легко спутать со значением.

### 9. Многострочные строки: `( ... )` и `(( ... ))`

Значения, занимающие несколько строк, заключаются в круглые скобки.
Открывающая и закрывающая строки НЕ входят в значение.

`(` ... `)` — общий ведущий отступ срезается, так что можно выравнивать
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

`((` ... `))` — побайтово: каждый символ между маркерами попадает в
значение, включая ведущие пробелы:

```text
sig: ((
  -----BEGIN-----
  QUJDRA==
  -----END-----
))
```
```json5
{ sig: "  -----BEGIN-----\n  QUJDRA==\n  -----END-----" }
```

Внутри блока `{` / `[` / `#` — просто содержимое, **никакого разбора
compound, никакого пропуска комментариев**. Единственная особая
последовательность — терминатор на отдельной строке.

Пустая инлайн-форма: `key: ()` или `key: (())` — обе дают пустую
строку (то же, что `key:`).

Сериализация: любая строка с `\n` эмитится через `(( ... ))`, так что
round-trip байт-в-байт без потерь. Строки без переноса используют
обычную однострочную форму.

Ограничение: строка, у которой trimmed-содержимое ровно `)` / `))`,
всегда закрывает блок, так что такой литерал не может попасть в
содержимое без использования внешнего файла.

### 10. Пустые compound-ы

```text
meta: {}
tags: []
```

Inline-пустой разрешён. Всё с содержимым обязано занимать несколько
строк, и закрывающий `}` / `]` обязан стоять на отдельной строке.

### 11. Enum-ы

Ktav использует дефолтное *externally tagged* представление enum-ов
serde.

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode { Fast, Slow }

#[derive(Serialize, Deserialize)]
enum Action {
    Log(String),
    Count(u32),
}
```

```text
# unit variant — just the name
mode: fast

# newtype variant — single-entry object
action: {
    Log: hello
}
```

## Round-trip

```rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
```

Сериализация сохраняет:
- **Порядок полей** — `Value::Object` лежит на `IndexMap`, так что
  порядок — тот, что эмитит serde (для struct-ов: порядок объявления).
- **Литеральные строки** — значения, начинающиеся с `{` или `[`,
  эмитятся с маркером `::`.
- **Поля `None`** — пропускаются на выходе; восстанавливаются как
  `None` на входе (через обработку `Option` в serde).

## Архитектура

```
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── render/           — pretty-printer (Value → text)
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error + serde::Error impls
└── lib.rs            — glue: from_str / from_file / to_string / to_file
```

В каждом файле — один экспортируемый элемент; детали реализации
приватны внутри родительского модуля.

## Чего Ktav НЕ делает — и никогда не будет

- **Inline непустые compound-ы** вроде `x: { a: 1, b: 2 }`. Они
  притащили бы запятые, а запятые притащили бы escape. Compound-
  значения многострочны.
- **Якоря / алиасы / merge-ключи** (`&anchor`, `*ref`, `<<:`). Любая
  строка, смысл которой зависит от декларации в отдалённом месте,
  перестаёт быть самодостаточной. Если нужен DRY — композируйте
  defaults в коде.
- **Инклюды файлов** (`@include`, `!import`). Для больших конфигов
  напишите обёртку в коде.
- **Массивы на верхнем уровне.** Документ всегда — объект.

## Установка

После публикации:

```toml
[dependencies]
ktav = "0.1"
serde = { version = "1", features = ["derive"] }
```

## Поддержите проект

У автора много идей, которые могут быть полезны IT во всём мире, — и
далеко не только для Ktav. Их реализация требует финансирования. Если
вы хотите помочь — пишите на **phpcraftdream@gmail.com**.

## Лицензия

MIT. См. [LICENSE](LICENSE).

## Другие реализации Ktav

- [`spec`](https://github.com/ktav-lang/spec) — спецификация + conformance-тесты
- [`csharp`](https://github.com/ktav-lang/csharp) — C# / .NET (`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) — Go (`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) — Java / JVM (`io.github.ktav-lang:ktav` на Maven Central)
- [`js`](https://github.com/ktav-lang/js) — JS / TS (`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) — PHP (`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) — Python (`pip install ktav`)
