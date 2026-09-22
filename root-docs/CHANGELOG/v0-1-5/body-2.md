>>>>> lang=en
- `Error::Structured(ErrorKind)` variant on the existing `Error` enum.
- `pub struct Span { start: u32, end: u32 }` with `Span::new`,
  `Span::EMPTY`, `slice(input)`, and `line_col(input)` (1-based line,
  0-based byte column — multi-byte UTF-8 aware via tests pinning
  Cyrillic and 🦀).
- `Error::line() -> Option<u32>` and `Error::span() -> Option<Span>`
  convenience accessors covering every variant.
- `pub mod thin` — public event-based parser API:
  `ktav::parse_events(input, callback)` invoking the supplied
  `FnMut(ParseEvent<'_>)` for each event borrowed from the input.
  `ParseEvent` is a `#[non_exhaustive]` enum with 10 variants
  (`Null`, `Bool`, `Integer`, `Float`, `Str`, `Key`, `BeginObject`,
  `EndObject`, `BeginArray`, `EndArray`). The internal bumpalo arena
  stays private — the public API does not leak the arena type.
- Crate-level runnable doctest in `src/lib.rs` demonstrating both
  `Error::Structured` matching with `Span::slice` and the
  `parse_events` callback shape.
- Six new top-level test files:
  `tests/error_format.rs` — Display-string regression net (canonical
  pinning for the 7 categories that LSP / bindings rely on);
  `tests/structured_errors.rs` — variant identity + (line, span) byte
  ranges per spec invalid fixture;
  `tests/error_spans.rs` — span byte-range semantics + `Span::slice`
  and `Span::line_col` edge cases (UTF-8 multi-byte, char-boundary
  rounding);
  `tests/error_accessors.rs` — every `Error` variant tested for
  `line()` / `span()` returning `Some` / `None` as documented;
  `tests/non_exhaustive.rs` — wildcard-arm reachability proof for
  `Error` and `ErrorKind`;
  `tests/thin_public.rs` — event sequencing, nested compounds, marker
  items, error propagation, borrow contract.
- Synthetic Criterion benchmarks under `benches/` covering parse
  perf at small_1k / medium_50k / large_500k workloads on both
  success and error paths. Baseline numbers in `bench-baseline.md`.

>>>>> lang=ru
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
- Шесть новых top-level test-файлов:
  `tests/error_format.rs` — регрессионная сеть по Display-строкам
  (канонический пиннинг 7 категорий, на которые опираются LSP и
  биндинги);
  `tests/structured_errors.rs` — идентичность варианта плюс диапазоны
  (line, span) в байтах на каждую невалидную фикстуру спеки;
  `tests/error_spans.rs` — семантика байтовых диапазонов span плюс
  краевые случаи `Span::slice` и `Span::line_col` (многобайтовый UTF-8,
  округление до границы символа);
  `tests/error_accessors.rs` — каждый вариант `Error` проверен на то,
  что `line()` / `span()` возвращают `Some` / `None` как задокументировано;
  `tests/non_exhaustive.rs` — доказательство достижимости
  wildcard-ветки для `Error` и `ErrorKind`;
  `tests/thin_public.rs` — последовательность событий, вложенные
  составные, marker-элементы, распространение ошибок, контракт
  заимствования.
- Синтетические Criterion-бенчи под `benches/` покрывающие parse-perf
  на small_1k / medium_50k / large_500k workloads, на success и error
  путях. Baseline-числа в `bench-baseline.md`.

>>>>> lang=zh
- 现有 `Error` 枚举上的 `Error::Structured(ErrorKind)` 变体。
- `pub struct Span { start: u32, end: u32 }`,提供 `Span::new`、
  `Span::EMPTY`、`slice(input)` 与 `line_col(input)`(line 从 1 起,
  column 从 0 起按字节计;多字节 UTF-8 已通过西里尔与 🦀 测试固定)。
- `Error::line() -> Option<u32>` 和 `Error::span() -> Option<Span>` —
  覆盖每个变体的便捷访问器。
- `pub mod thin` —— 公开的事件式解析器 API:
  `ktav::parse_events(input, callback)` 对从 input 借用的每个事件
  调用所提供的 `FnMut(ParseEvent<'_>)`。`ParseEvent` 是
  `#[non_exhaustive]` 枚举,有 10 个变体。内部 bumpalo arena
  保持私有 —— 公共 API 不泄露 arena 类型。
- `src/lib.rs` 的 crate 级可运行 doctest,演示 `Error::Structured`
  匹配配合 `Span::slice` 以及 `parse_events` 回调形态。
- 六个新顶层测试文件:
  `tests/error_format.rs` —— Display 字符串的回归网(为 LSP 与绑定所
  依赖的 7 个类别做规范化钉定);
  `tests/structured_errors.rs` —— 按规范的每个无效 fixture 校验变体
  身份以及 (line, span) 字节范围;
  `tests/error_spans.rs` —— span 字节范围语义,以及 `Span::slice` 与
  `Span::line_col` 的边界情形(多字节 UTF-8、按字符边界取整);
  `tests/error_accessors.rs` —— 逐一验证每个 `Error` 变体的
  `line()` / `span()` 是否如文档所述返回 `Some` / `None`;
  `tests/non_exhaustive.rs` —— `Error` 与 `ErrorKind` 的 wildcard 分支
  可达性证明;
  `tests/thin_public.rs` —— 事件序列、嵌套复合、marker 元素、错误传播、
  借用契约。
- `benches/` 下的合成 Criterion 基准,覆盖 small_1k / medium_50k /
  large_500k 负载在成功路径和错误路径上的解析性能。baseline 数字
  位于 `bench-baseline.md`。

