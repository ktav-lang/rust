# Журнал изменений — crate `ktav`

**Languages:** [English](CHANGELOG.md) · **Русский** · [简体中文](CHANGELOG.zh.md)

Все значимые изменения в crate `ktav` документируются здесь. Формат
основан на [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
crate следует [Semantic Versioning](https://semver.org/) с
Cargo-конвенцией: до 1.0 bump MINOR считается ломающим.

Историю самой спецификации формата см. в репозитории
[`ktav-lang/spec`](https://github.com/ktav-lang/spec).


## [0.6.2] — 2026-08-19

### Добавлено

- `parse_strict()` — опциональный строгий режим разбора, отвергающий
  **скаляры с потерей**: значения, лексическая форма которых отличается
  от канонической формы числа, в которое они были бы выведены (`1.10` →
  `1.1`, `01234` → `1234`, `+7`, `0x1A`, `0o755`, `1_000`, `5e3`).
  Обычный `parse()` молча канонизирует такие значения, поэтому
  round-trip переписывает документ без всякой диагностики; строгий
  режим вместо этого сообщает о них, называя обе формы и подсказывая
  два способа исправления (дописать `::`, чтобы оставить строку, либо
  записать число в канонической форме). Документы, принятые
  `parse_strict()`, дают ровно то же дерево `Value`, что и `parse()`.
- `ErrorKind::LossyScalar { line, body, canonical, span }` — новый
  вариант ошибки, существующий только в строгом режиме; учтён в
  `ErrorKind::line()` / `ErrorKind::span()`. `ErrorKind` помечен
  `#[non_exhaustive]`, поэтому изменение аддитивное.

Поведение `parse()`, serde-пути (`from_str`) и C ABI не изменилось;
у serde event-пути строгого варианта пока нет.

### Изменено

- **MSRV поднят с 1.70 до 1.71.** Не наш выбор: `serde_core` требует
  `serde_derive = "=1.0.229"`, а тот релиз (2026-07-18) объявляет
  `rust-version = 1.71`. Библиотека не поставляет свой `Cargo.lock`,
  поэтому пользователь на 1.70 всё равно уже не смог бы собрать этот
  crate — манифест теперь говорит правду о реальных требованиях.

Спасибо [@chappihappymeal](https://github.com/chappihappymeal) за
найденную проблему и реализацию
([#1](https://github.com/ktav-lang/rust/issues/1),
[#2](https://github.com/ktav-lang/rust/pull/2)).

## [0.6.1] — 2026-06-05

- Документация: все примеры в README переписаны под синтаксис спецификации 0.6 (голые числа вместо удалённых маркеров `:i`/`:f`; комментарии `##` вместо `#`).

## [0.6.0] — 2026-06-01

Реализация спецификации Ktav 0.6.0. Добавлено **экранирование в
ключах**: ключи теперь обрабатывают набор escape-последовательностей
из §3.7, и два новых escape — `\.` и `\:` — позволяют использовать
буквальные точку/двоеточие внутри сегмента ключа.

### Breaking

- Буквальный символ `\` в ключе теперь требует `\\`. Раньше парсер
  трактовал `\` в ключе как обычный байт без обработки escape.
  Файлы-источники с одиночными `\` в ключах нужно удвоить, чтобы
  сохранить те же байты ключа. Значения не затронуты.

### Добавлено

- Таблица escape расширена с 8 до 10 последовательностей: к восьми
  существующим (`\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`, `\r`)
  добавлены `\.` (буквальная точка в сегменте ключа — НЕ разделяет
  путь) и `\:` (буквальное двоеточие — НЕ работает как разделитель
  ключ/значение). Оба новых escape допустимы (избыточны) и в
  значениях.
- Сканер ключей учитывает экранирование: первый **неэкранированный**
  `:` (или `::`) — разделитель пары; путь разбивается только по
  **неэкранированным** `.`. То же действует для inline-compound
  (`{a\.b: 1}`).
- Рендерер экранирует обратно `\`, `.`, `:` в каждом сегменте ключа —
  гарантия parse → render → parse тождественности для ключей с
  буквальными точкой/двоеточием.
- В zero-copy event-пути сегмент без `\` остаётся заимствованным из
  исходного буфера; сегмент с escape декодируется в bump-arena —
  время жизни `&'a str` сохраняется.

### Ошибки

- `\X` в ключе, где `X` не входит в десятку, теперь
  `BadEscapeSequence`. `\.` и `\:` больше не `BadEscapeSequence`
  ни в одном контексте.


## [0.2.0] — 2026-05-07

Минорный релиз с двумя breaking-изменениями вывода / валидации:

### Изменено (breaking)

- **Многострочные строки по умолчанию выводятся в форме stripped
  `( ... )`** с отступом, а не verbatim `(( ... ))`. Verbatim остаётся
  как fallback когда содержимое имеет собственный leading-whitespace
  (parser-side dedent его съест) или строку, тримящуюся в `)` (закрыла
  бы stripped преждевременно). Код, сравнивающий вывод `to_string` /
  `render` побайтово с фиксированным `((...))`, нужно обновить.
  Round-trip (`parse(to_string(v)) == v`) не изменился.

      // Было (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // Стало (0.2.0):
      // body: (
      //     line1
      //     line2
      // )

  Оба пути сериализации — `Value` → `render::render(&value)` и
  `T: Serialize` → `ser::to_string(&t)` — обновлены консистентно.

- **Типизированный маркер `:f` принимает integer-литералы.** Десятичная
  точка в мантиссе теперь **опциональна**: `:f 42` валидно (парсится
  как `42.0`) — конвенция JSON / TOML / YAML, где integer литералы
  приводятся к float. `:f 1.` (без дробной части) и `:f .5` (без
  целой части) по-прежнему невалидны. Код, ожидающий
  `InvalidTypedScalar` для `:f 42`, нужно обновить.

### Spec

- В `spec/versions/0.1/tests` фикстура `typed_float_without_decimal`
  перенесена из `invalid/` в `valid/typed_float_integer_body` под
  новую семантику. Submodule spec синхронизирован.


## [0.1.5] — 2026-05-01

Большой релиз: структурированные ошибки с byte-offset spans, публичное
event-based API парсера, retroactive `#[non_exhaustive]` на error-enum-ах
для forward-compatibility.

### Добавлено

- Enum `ErrorKind` (10 спек-определённых вариантов + `Other`) с
  byte-offset `span: Span` на каждом варианте, выставляющий
  `(line, column, kind)` напрямую downstream-потребителям без regex-
  парсинга форматированного сообщения.

      pub enum ErrorKind {
          MissingSeparatorSpace { line, column, marker, span },
          InvalidTypedScalar    { line, marker, body, span },
          DuplicateKey          { line, key, span },
          KeyPathConflict       { line, path, kind: ConflictKind, span },
          EmptyKey              { line, span },
          InvalidKey            { line, key, span },
          UnclosedCompound      { kind: CompoundKind, span },
          UnbalancedBracket     { line, expected: CompoundKind, found: char, span },
          InlineNonEmptyCompound{ line, body, span },
          MissingSeparator      { line, span },
          Other                 { line: Option<u32>, message, span },
      }

- Вариант `Error::Structured(ErrorKind)` на существующем enum `Error`.
- `pub struct Span { start: u32, end: u32 }` с `Span::new`,
  `Span::EMPTY`, `slice(input)`, и `line_col(input)` (line 1-based,
  column 0-based байтовая — multi-byte UTF-8 учтён через тесты,
  пинящие кириллицу и 🦀).
- `Error::line() -> Option<u32>` и `Error::span() -> Option<Span>` —
  convenience-accessors, покрывающие каждый вариант.
- `pub mod thin` — публичное event-based API парсера:
  `ktav::parse_events(input, callback)` вызывает поставленный
  `FnMut(ParseEvent<'_>)` для каждого события, заимствованного из
  input. `ParseEvent` — `#[non_exhaustive]` enum с 10 вариантами
  (`Null`, `Bool`, `Integer`, `Float`, `Str`, `Key`, `BeginObject`,
  `EndObject`, `BeginArray`, `EndArray`). Внутренняя bumpalo arena
  остаётся приватной — публичный API не утекает тип арены.
- Crate-level runnable doctest в `src/lib.rs` демонстрирует и
  matching `Error::Structured` со `Span::slice`, и форму
  `parse_events` callback.
- Шесть новых top-level test-файлов: `tests/error_format.rs`,
  `tests/structured_errors.rs`, `tests/error_spans.rs`,
  `tests/error_accessors.rs`, `tests/non_exhaustive.rs`,
  `tests/thin_public.rs` — детально описаны в английской версии.
- Синтетические Criterion-бенчи под `benches/` покрывающие parse-perf
  на small_1k / medium_50k / large_500k workloads, на success и error
  путях. Baseline-числа в `bench-baseline.md`.

### Изменено

- `#[non_exhaustive]` retroactively применён к `Error`, `ErrorKind`,
  `ConflictKind` и `CompoundKind`. Будущие добавления вариантов
  больше не являются ломающим изменением для downstream-`match`-еров,
  которые теперь обязаны иметь arm `_ =>`.
- Парсер больше не конструирует `Error::Syntax(format!(...))` ни на
  одном внутреннем call-сайте (~37 сайтов перерефакторены на
  `Error::Structured`). Регрессионный guard-test
  (`parser_no_longer_emits_legacy_syntax_variant`) гоняет 12
  невалидных входов и громко фейлит CI, если кто-то реинтродуктит
  legacy-вариант внутри `src/`.
- `parser/parse_str.rs` заменяет `str::lines()` на ручной
  byte-walking loop, поддерживающий cumulative-`line_start` счётчик —
  byte-offset spans вычисляются на каждом error-сайте без
  пересканирования. `thin/event_parser.rs` зеркалит то же plumbing
  на zero-copy пути.
- `Display for ErrorKind` byte-identical к строкам, которые парсер
  ранее форматировал в `Error::Syntax(...)` для семи pre-existing
  категорий — это контракт, позволяющий каждому существующему
  string-based caller-у работать без изменений на время
  ecosystem-wide миграции, описанной в
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md).
- Три прежде-`Other` формы промоутированы в именованные варианты
  `ErrorKind`: `UnbalancedBracket` (lone closer / shape mismatch),
  `InlineNonEmptyCompound` (`x: {foo}` — спека § 6.7),
  `MissingSeparator` (строка без `:`). После promotion-а `Other`
  содержит только парсер-внутренние invariants, которые ни одна
  spec invalid фикстура не триггерит.

### Производительность

`cargo bench --bench parse -- --quick` против baseline 0.1.4: zero
регрессии на success-path, error-path немного быстрее (lazy Display
вместо eager `format!`). Полная таблица — в английской версии.

### Заметки

- `Error::Syntax(String)` сохранён для обратной совместимости —
  публичный API остаётся deny-no-old-callers. Удаление отложено до
  ktav 1.0.
- Тестов: 332 (0.1.4) → 391 (+59) плюс 1 новый doctest.
- Миграция cabi/биндингов на потребление `ErrorKind` через FFI
  трекается отдельно в
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md) и идёт как
  coordinated ecosystem 0.2.0.

### SemVer-замечание

Добавление `#[non_exhaustive]` к ранее непомеченным enum-ам (`Error`,
`ConflictKind`, `CompoundKind`) — согласно
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive)
— breaking change, требующий major-bump-а (0.2.0). Этот релиз
выпускается как **0.1.5** намеренно:

1. Pre-1.0 Cargo-конвенция допускает breaking-изменения на любом
   bump-е, включая патчи.
2. Все известные downstream-потребители `ktav::Error` (шесть
   языковых биндингов под `ktav-lang/`) делают только
   `Err(e) => e.to_string()`. Exhaustive `match err { Error::Io(_)
   => …, Error::Syntax(_) => …, Error::Message(_) => … }` в
   экосистеме нет — ломать тихо нечего.
3. Display-строки семи канонических категорий byte-identical к
   0.1.4, поэтому любой гипотетический out-of-tree-потребитель,
   делающий string matching, продолжает работать без изменений.

Если ваш код всё-таки держит exhaustive `match` по `ktav::Error` и
этот релиз его ломает — добавьте arm `_ => …`. Этот arm теперь
обязателен навсегда и больше не потребует изменений при добавлении
будущих вариантов.

## [0.1.4] — 2026-04-26

### Изменено

- **`Frame::Object` initial capacity 4 → 8** (`src/parser/frame.rs`).
  Per-compound `IndexMap` парсера теперь pre-sizes под 8 элементов
  вместо 4 — устраняется первый growth/rehash для типичной строки
  конфига (5–8 полей). Это **untyped** парсинг-путь
  (`ktav::parse → Value`) — тот самый путь, через который идут все
  C-ABI биндинги (PHP/JS/Python/Go/Java/C#) через `cabi`, поэтому
  они **получат** ускорение, как только подхватят 0.1.4.
- Эффект на бенче `parse_to_value` (медиана 3 прогонов): small
  **−30%** (18.9 µs → 13.3 µs), large **−13%** (5.04 ms → 4.4 ms),
  medium в шуме (~−3%).

Изменение в одну строку; полный набор тестов (334 кейса вкл.
spec conformance) не затронут.

## [0.1.3] — 2026-04-26

То же содержимое, что и yank-нутая 0.1.2 — перевыпуск через новый
автоматизированный workflow `Release` (CI verify → `cargo publish`),
чтобы будущие релизы не зависели от ручного `cargo publish` с машины
сопровождающего. 0.1.2 был отозван только ради end-to-end проверки
пайплайна на свежей версии (crates.io immutable, перевыпустить саму
0.1.2 нельзя).

## [0.1.2] — 2026-04-26

Перевыпуск содержимого 0.1.1 после прогона `cargo fmt`. 0.1.1 был
отозван (yanked), потому что новые файлы (`benches/vs_json.rs`,
`src/thin/event*.rs`, `src/thin/fast_num.rs`) не были отформатированы
через rustfmt перед публикацией, что обвалило CI lint при пуше тега.
**Функционально идентично 0.1.1** — отличается только пробелами.

## [0.1.1] — 2026-04-26

### Изменено

- **Быстрый путь типизированной десериализации** — `from_str` и
  `from_file` больше не строят промежуточное дерево `ThinValue`.
  Парсер сразу эмитит плоский `Vec<Event>` в bump-арену, а serde-
  десериализатор линейно идёт по нему одним курсором — одна
  аллокация на документ вместо одной на компаунд, без подгрузки
  enum-discriminant'а через косвенность. На 275 KB конфиге:
  **−18.7%** на `parse → struct` (3.60 ms → 2.93 ms).
- **`fast_num` byte-loop atoi** — пути `i8`..`i64` / `u8`..`u64` в
  типизированном десериализаторе обходят generic-маршрут через
  `<T as FromStr>` и используют ручные `parse_i64` / `parse_u64`
  с проверкой ширины. Float-пути остаются на `f64::from_str`.

### Добавлено

- Внутренние `Event` enum и `EventCursor` walker (`thin/event*.rs`).

### Удалено

- Enum `ThinValue` и его `ThinDeserializer` (заменены event-stream'ом
  — оба были `pub(crate)`, публичный API не сломан).

### Изменение поведения

- **Чередование dotted-key префиксов теперь отклоняется как
  conflict.** Документ вида `a.x: 1\nb.y: 2\na.z: 3` (synthetic `a`
  открыт, закрыт через `b.`, попытка переоткрыть через `a.z`)
  раньше тихо сливался в один объект `a` через tree-builder.
  Event-stream-токенизатор не может это сделать без буферизации
  всего документа, поэтому теперь возвращает понятный conflict-
  ошибку с предложением сгруппировать строки с одним префиксом
  вместе. Документы со сгруппированными dotted-ключами (канонический
  паттерн) не затронуты — все spec-conformance фикстуры зелёные.

## [0.1.0] — 2026-04-22

Первый релиз. Реализует [Ktav spec 0.1.0](https://github.com/ktav-lang/spec/blob/main/versions/0.1/spec.md).

### Added

- **Parser** — превращает текст Ktav в `Value` (владеющий) или
  `ThinValue` (zero-copy view поверх входного буфера). Построчная
  state machine с разворачиванием точечных ключей, многострочными
  строками (со снятием отступа и побайтовыми), JSON-подобными
  ключевыми словами `null` / `true` / `false` и типизированными
  скалярными маркерами `:i` (Integer) и `:f` (Float).
- **Serializer** — два пути:
  - `ktav::to_string` (прямая эмиссия текста, основной путь).
  - `ktav::ser::to_value` / `ktav::render` (двухшаговый вариант для
    тех, кто хочет осмотреть `Value` между стадиями).
  Оба автоматически эмитят `::` для строк, которые иначе были бы
  неверно прочитаны парсером, и эмитят `:i` / `:f` для числовых
  Rust-типов.
- **Deserializer** — zero-copy путь через `ThinValue<'a>` и
  `ThinDeserializer`. Ключи объектов и однострочные скалярные
  значения заимствуются напрямую из входа; выделение памяти
  происходит только для многострочных строк. Принимает обе формы
  чисел — с маркером и без: документы, написанные без маркеров,
  десериализуются прозрачно через `FromStr`.
- **Serde integration** — `from_str`, `from_file`, `to_string`,
  `to_file` принимают любой `T: Serialize` / `DeserializeOwned`,
  включая типы, сгенерированные `#[derive]`, вложенные struct-ы,
  `Vec`, `Option`, `HashMap` и стандартные externally-tagged формы
  enum-ов. Целочисленные Rust-типы (`u8`..`u128`, `i8`..`i128`,
  `usize`, `isize`) сериализуются с `:i`; плавающие (`f32`, `f64`) —
  с `:f`; `NaN` и `±Infinity` отвергаются сериализатором (Ktav 0.1.0
  их не представляет).
- **Raw-маркер `::`** — заставляет значение быть литеральной String,
  как в позиции пары (`key:: value`), так и как префикс элемента
  массива (`:: value`).
- **Типизированные маркеры `:i` и `:f`** — явные Integer / Float в
  позиции пары (`port:i 8080`, `ratio:f 0.5`) и как префиксы
  элементов массива (`:i 42`, `:f 3.14`). На уровне `Value` хранятся
  как строки — для сохранения произвольной точности.
- **Многострочные строки** — `( ... )` (со снятием общего отступа) и
  `(( ... ))` (побайтово). Round-trip байт-в-байт через побайтовую
  форму.
- **Публичный enum `Value`** — `Null`, `Bool`, `Integer`, `Float`,
  `String`, `Array`, `Object` (на основе `IndexMap` с
  `rustc_hash::FxBuildHasher`). Аксессоры `Value::as_integer` /
  `as_float`; аналогичные на `ThinValue`.
- **Сообщения об ошибках** — каждая синтаксическая ошибка несёт номер
  строки; ошибки десериализации несут точечный путь
  (`upstreams.[0].port`). Нарушения типизированных скаляров
  отмечаются префиксом `InvalidTypedScalar` в сообщении.
- **Spec conformance тесты** — `tests/spec_conformance.rs` прогоняет
  language-agnostic набор из репозитория `ktav-lang/spec`
  (находится через env-переменную `KTAV_SPEC_DIR` или fallback
  `../spec`). Три проверки: соответствие Value JSON-оракулу,
  отвержение invalid-fixture-ов и lossless round-trip Value-уровня
  через рендерер.

### Performance (criterion, typed-конфиг 22 KB, Windows release)

- `parse → struct`: **275 µs** (~80 MB/s)
- `render struct → text`: **46 µs** (~475 MB/s)
- `round-trip`: **377 µs**

### Dependencies

- `serde` с `derive`
- `indexmap` с фичей `serde`
- `rustc-hash` (FxHash — быстрый и детерминированный; не
  устойчив к коллизиям, а парсеру конфигов это и не нужно)

### MSRV

`rustc 1.70` или новее.
