# Ktav (כְּתָב)

[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

> Простой формат конфигурации. Формы JSON5, но без кавычек, без
> запятых, с точечными ключами для вложенности. Нативная интеграция
> с `serde`.

**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)

**Песочница:** конвертация JSON / YAML / TOML / INI ⇄ Ktav прямо в браузере — **[ktav-lang.github.io](https://ktav-lang.github.io/)**.

**Спецификация:** этот crate реализует **Ktav 0.7.1** — версию,
указанную в `[package.metadata.ktav] spec-version` в `Cargo.toml`.
Формат версионируется и поддерживается независимо от crate-а, и два
номера намеренно расходятся: выпуск crate-а, не меняющий поведение
формата, оставляет `spec-version` на месте. См.
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) для канонического
документа и [`CHANGELOG.ru.md`](CHANGELOG.ru.md) — историю crate-а.

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
## comment              — any line starting with '##'
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
## `::` принудительно задаёт литеральную строку
pattern:: [a-z]+
```

### Числа

Числа пишутся без кавычек и типизируются по лексической форме: голое
целое тело разбирается в `Value::Integer`, голая десятичная запись —
в `Value::Float`. Каждый хранит *нормализованный* payload, а не
исходное написание: `Integer` хранит канонический десятичный вид (без
подчёркиваний, без ведущих нулей/`+`), `Float` — кратчайшую десятичную
форму, восстанавливающую точные биты `f64` — `+1_000` становится
`Integer("1000")`, `1.0e+2` становится `Float("100.0")`. `Integer` на
уровне `Value` покрывает диапазон i64; более широкий native Rust
целочисленный тип (`u64`, `i128`, `u128`), не помещающийся в i64, при
проходе через `ser::to_value` сохраняется как `Value::String` — точно
так же, как если бы этот же десятичный текст был разобран парсером.
serde десериализует числа в целевой Rust-тип (`u16`, `i64`, `i128`,
`f64`, …) прямым разбором и форматирует их с той же канонизацией при
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
```

### Null: `null`

Строго нижний регистр. На стороне Rust соответствует `Option::None`,
а также `()` для unit.

```text
## Value::Null
label: null
## Value::String("Null")
capitalized: Null
## Value::String("null")
literal:: null
```

При сериализации `Option::None` эмитится как `null`. Подавить можно
через `#[serde(skip_serializing_if = "Option::is_none")]`, если вы
предпочитаете, чтобы поля не было вовсе.

### Пустой объект / пустой массив

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

## Составные значения — многострочные

Непустые `{ ... }` / `[ ... ]` **обязаны** занимать несколько строк,
с закрывающей скобкой на отдельной строке. `x: { a: 1 }` и
`x: [1, 2, 3]` отклоняются с ясной ошибкой — в Ktav нет правил
разделения запятыми и нет механизма escape для них.

```text
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
```

## Углы, о которых стоит знать

Четыре правила, до которых примеры выше не доходят. Каждое решено
спецификацией, а не этой реализацией, поэтому любой соответствующий
парсер ведёт себя так же.

### Кавычные ключи — когда в ключе точка или пробел

Точка в голом ключе означает вложенность: `db.host: primary`
строит `db` → `host`. Кавычки отключают это прочтение, и точка
становится частью имени (§ 5.3.3):

```ktav
db.host: primary
"db.host": literal
"a b": spaces are fine too
```

Первая строка вкладывает. Вторая — один ключ, буквально названный
`db.host`. Именно поэтому конверт ошибок отдаёт `path` массивом
сегментов, а не склеенной строкой: склеенная строка не смогла бы
различить эти два случая.

### Экранирование действует внутри inline-компаундов и кавычных ключей

`\uXXXX` и именованные escape-последовательности (§ 3.7, § 3.7.1)
читаются там, где разделитель иначе был бы структурным — внутри
`{ }` и `[ ]`, а также внутри кавычного ключа. У голого блочного
значения экранировать нечего, поэтому обратный слэш там — обычный
текст:

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

### Анализ ошибок — структурированные варианты для тулинга

`Error::Structured(ErrorKind)` несёт типизированную категорию плюс
byte-offset `Span` для каждого parse-фейла, так что редакторы и
линтеры могут подсветить именно offending диапазон, а не всю строку.

```rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\nport: 443\n";
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
```

Варианты: `MissingSeparatorSpace`, `InvalidTypedScalar`, `DuplicateKey`,
`KeyPathConflict`, `EmptyKey`, `InvalidKey`, `UnclosedCompound`,
`UnbalancedBracket`, `InlineNonEmptyCompound`, `MissingSeparator`,
`LossyScalar` (только в строгом режиме, см. ниже), `Other`. Enum помечен
`#[non_exhaustive]` — обязателен arm `_ =>`. `Error::line()` /
`Error::span()` — convenience accessors для случаев когда вариант
неважен. `Display` выдаёт те же читаемые строки, что и legacy
`Error::Syntax(_)`, поэтому существующие string-based вызывающие
работают без изменений.

Полный запускаемый пример проходит по всем вариантам:
[`examples/errors.rs`](examples/errors.rs) — `cargo run --example errors`.

### Один JSON-конверт для любой структурированной ошибки

Аксессоры выше существуют только в Rust. `ErrorEnvelope` — контракт
для всех остальных: один JSON-объект, десять полей, всегда все
десять, в фиксированном порядке — `error`, `reason`, `line`,
`line_text`, `span`, `path`, `body`, `canonical`,
`spec_section`, `message`.

```rust
use ktav::{parse, ErrorEnvelope};

let src = "a: 1.10\n";
if let Err(e) = ktav::parse_strict(src) {
    println!("{}", ErrorEnvelope::from_error(&e, src).to_json());
}
```

```json
{"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
 "span":{"start":0,"end":7},"path":null,"body":"1.10",
 "canonical":"1.1","spec_section":"§3.6/§5.2",
 "message":"Syntax error: Line 1: LossyScalar: '1.10' would be …"}
```

`message` — последнее поле и единственное, которое никогда не
`null`: в нём дословно лежит `Display`-представление этой ошибки.
Биндинг показывает его пользователю как есть, а не собирает текст сам
из структурированных полей, поэтому один и тот же документ даёт
одинаковый текст ошибки на любом языке. Девять прежних полей сохранили
свои позиции — `message` дописано в конец, а не вставлено в середину.

Отсутствующие сведения — явный `null`, а не пропущенный ключ, поэтому
потребитель читает каждое поле позиционно, без предварительного
согласования схемы.

`span` — это `{"start":N,"end":M}` в **байтовых смещениях по
UTF-8-исходнику**, а не в code units UTF-16: та же единица, что у самого
[`Span`](https://docs.rs/ktav). Потребителю LSP нужно либо
конвертировать, либо договариваться о `positionEncoding: "utf-8"`.

`path` — **массив точных декодированных сегментов ключа, а не
склеенная строка**. Ключ, буквально названный `a.b`, — это один
сегмент, и спутать его с путём из двух нельзя: в контракте просто нет
разделителя, вокруг которого возникла бы двусмысленность.

Отказы писателя используют тот же конверт: `reason` несёт код причины
из § 5.9.0 (`NonFiniteFloat`, `EmptyKeyName`, …), а сами отказы
названы раздельно — `UnrepresentableAt`, когда писатель может указать
узел (тогда он заполняет и `path`), и `Unrepresentable`, когда не
может.

Печать — `to_json()` (или `push_json(&mut String)`, чтобы дописать в
собственный буфер). Результат — валидный JSON для любого содержимого:
каждая строка экранируется по RFC 8259, зависимость от serde не
задействована.

### Строгий режим — ловим молча канонизированные числа

Типы выводятся по лексической форме скаляра, а выведенные числа
канонизируются: `version: 1.10` разбирается как `Float(1.1)`. Десятичный
литерал с избыточным ведущим нулём — единственное исключение: `zip: 01234`
это String `"01234"` (§ 5.2), потому что отбрасывание этого нуля
уничтожило бы идентификатор. Обычный `parse()` делает это молча,
поэтому обратная запись документа его переписывает.

`parse_strict()` вместо этого отвергает такие **скаляры с потерей**:

```rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "version: 1.10\n";

assert!(parse(src).is_ok());                 // Float(1.1) — замыкающий ноль потерян
assert!(parse("zip: 01234\n").is_ok());       // String("01234") — ведущий ноль сохранён

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
    }
    other => panic!("ожидался LossyScalar, получено {other:?}"),
}
```

Исправить можно либо дописав `::`, чтобы значение осталось строкой
(`zip:: 01234`), либо записав число в канонической форме. Любой
документ, принятый `parse_strict()`, даёт ровно то же дерево `Value`,
что и `parse()` — строгий режим это проверочный шлюз, а не другой
диалект. У serde-пути (`from_str`) строгого варианта пока нет.

### Stream-парсинг — события без промежуточного дерева

`parse_events` вызывает callback на каждое событие парсинга, со
строками заимствующимися напрямую из input-буфера — без аллокации на
событие, без промежуточного `Value`-дерева. Полезно когда полный
документ не нужен: подсчёт ключей, стриминг в другой формат, построение
custom-shape-а.

```rust
use ktav::{parse_events, ParseEvent};

let src = "port: 8080\nhost: example.com\n";
let mut keys = Vec::new();
parse_events(src, |ev| {
    if let ParseEvent::Key(k) = ev {
        keys.push(k.to_string());
    }
})?;
assert_eq!(keys, ["port", "host"]);
```

Корень — `BeginObject`/`EndObject` или `BeginArray`/`EndArray`, в
зависимости от первой содержательной строки документа (здесь Object,
так как `port: 8080` — это пара); вложенные компаунды обрамляют своё
содержимое так же. `ParseEvent` помечен
`#[non_exhaustive]`. Полный запускаемый пример с tracking-ом глубины
и pretty-print-ом:
[`examples/events.rs`](examples/events.rs) — `cargo run --example events`.

### Числа

Числовые Rust-типы (`u8`..`u128`, `i8`..`i128`, `usize`, `isize`, `f32`,
`f64`) сериализуются в Ktav как голые числа: `port: 8080`,
`ratio: 0.5`. На обратном пути голое целое/десятичное тело
десериализуется прямо в нужный числовой тип; значение, пришедшее
строкой (например, форсированное через `::`), по-прежнему принимается
через `FromStr`. `NaN` / `±Infinity` отвергаются сериализатором
(Ktav их не представляет).

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
  port: 20082
}
```

Скаляры типизируются на уровне `Value` по своей лексической форме
(`Integer`/`Float`/`Bool`/`Null`/`String`); принудить числоподобное
тело остаться строкой можно raw-маркером `::` (например,
`port:: 20082`).

### 2. Точечные ключи = вложенные объекты

```text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
```
```json5
{
  server: { host: "127.0.0.1", port: 8080 },
  app: { debug: true }
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
    port: 8080,
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
    { host: "a.example", port: 1080 },
    { host: "b.example", port: 1080 }
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

```json5
{ indent: "  padded" }
```
```text
indent: ((
  padded
))
```

В обоих случаях при обратном чтении получите исходные байты.

Ограничение: тело, где есть строка с trimmed-содержимым ровно `))`,
не может использовать дословную форму. Вместо неё берётся форма со
снятым отступом — если только в теле нет ещё и строки ровно `)`,
строки из одних пробелов, строки с замыкающим пробельным хвостом,
или все строки с отступом (не от чего
отсчитывать выравнивание к нулю) — тогда ни одна форма не подходит, и
сериализация возвращает ошибку вместо документа, который не
восстановится обратно.

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
## unit variant — just the name
mode: fast

## newtype variant — single-entry object
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

## Форматирование — каноническое написание, комментарии на месте

`ktav::format_str` переписывает документ в то структурное написание,
которое выдаёт `emit_canonical`, но сохраняет оформление, выбрасываемое
каноническим писателем. Каждый комментарий сохраняется дословно.

```rust
let tidied = ktav::format_str("## the server\nserver: {host: a, port: 80}\n")?;
assert_eq!(tidied, "## the server\nserver: {\n    host: a\n    port: 80\n}\n");
```

То есть inline-компаунд разворачивается в каноническую многострочную
форму, а комментарий остаётся ровно там, где был:

```ktav
## the server
server: {
    host: a
    port: 80
}
```

Пустые строки сохраняются как подсказка группировки, но серия из двух
и более схлопывается ровно в одну, а пустой отступ сразу внутри скобки
выбрасывается. Именно это делает преобразование неподвижной точкой:
форматирование уже отформатированного вывода больше ничего не меняет.

Порядок ключей не меняется никогда. У канонической формы нет правила
сортировки (§ 5.9), а перестановка ключей ухудшила бы диффы на ревью,
а не улучшила: это нормализатор написания, а не инструмент
рефакторинга.

Для документа без комментариев **и без пустых строк** `format_str`
совпадает с `emit_canonical` его разбора. Условие намеренно сильнее
очевидного: пустые строки входят в модель `Value` ровно настолько же,
насколько комментарии, — то есть никак, поэтому `emit_canonical` их
выбрасывает, а `format_str` нет.

### `ktav-fmt` — необязательный форматтер командной строки

Живёт за feature-флагом `cli` и по умолчанию выключен. В проекте, где
тулчейн уже есть, библиотечный вызов выше плюс хук сборки обычно
уместнее, а редакторы форматируют через `ktav-lsp`; бинарь — для
случая, когда ни того, ни другого под рукой нет.

```text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
```

`--check` ничего не пишет и печатает путь каждого файла, который ещё
не отформатирован, — так что он встаёт в CI прямо рядом с
`cargo fmt --check`.

## C ABI для других языков

Шесть языковых биндингов — Go, Java, PHP, C#, JS и Python — загружают
небольшую нативную библиотеку, собранную на этом crate. Разделяемая
половина шима живёт за выключенной по умолчанию фичей `cabi`: модуль
`ktav::cabi` несёт декодирование wire-формата, шесть операций над
документами и кодирование ошибок, а один макровызов в `cdylib`
биндинга разворачивает экспортируемые символы.

```text
// crates/cabi/src/lib.rs of a binding — the whole body:
ktav::declare_cabi!();
```

Развёртка экспортирует девять символов: шесть функций-операций
(`ktav_loads`, `ktav_loads_strict`, `ktav_dumps`,
`ktav_dumps_force_strings`, `ktav_emit_canonical`,
`ktav_format`), `ktav_free`, `ktav_version` и
`ktav_abi_version`. Каждая ошибка уходит девятиполевым
JSON-конвертом, поэтому хосту не приходится гадать, текст перед ним
или JSON, а `ktav_abi_version()` позволяет хосту отказаться от
устаревшей нативной библиотеки вместо порчи памяти. Полный контракт —
сигнатуры, владение, кодирование ошибок, конвенция имён артефактов
(`ktav_cabi-windows-amd64.dll`, `libktav_cabi-darwin-arm64.dylib`,
`libktav_cabi-linux-amd64.so`, переопределение `$KTAV_LIB_PATH`) —
описан в [docs/CABI.md](docs/CABI.md).

## Архитектура

```
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

```toml
[dependencies]
ktav = "0.7.1"
serde = { version = "1", features = ["derive"] }
```

Форматтер доступен и как исполняемый файл — за выключенным по
умолчанию feature-флагом:

```sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
```

## Поддержите проект

У автора много идей, которые могут быть полезны IT во всём мире, — и
далеко не только для Ktav. Их реализация требует финансирования. Если
вы хотите помочь — пишите на **phpcraftdream@gmail.com**.

## Лицензия

Двойное лицензирование: **MIT OR Apache-2.0** — на ваш выбор. См.
[LICENSE-MIT](LICENSE-MIT) и [LICENSE-APACHE](LICENSE-APACHE).

## Другие реализации Ktav

- [`spec`](https://github.com/ktav-lang/spec) — спецификация + conformance-тесты
- [`csharp`](https://github.com/ktav-lang/csharp) — C# / .NET (`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) — Go (`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) — Java / JVM (`io.github.ktav-lang:ktav` на Maven Central)
- [`js`](https://github.com/ktav-lang/js) — JS / TS (`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) — PHP (`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) — Python (`pip install ktav`)
