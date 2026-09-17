// Changelog units for 0.1.5.
export const v0_1_5 = [
  {
    id: 'v0-1-5-001',
    join: 'block',
    en: `## [0.1.5] — 2026-05-01`,
    ru: `## [0.1.5] — 2026-05-01`,
    zh: `## [0.1.5] —— 2026-05-01`,
  },
  {
    id: 'v0-1-5-002',
    join: 'block',
    en: `Major release: structured errors with byte-offset spans, public
event-based parser API, \`#[non_exhaustive]\` retroactively applied to
the error enums for forward-compatibility.`,
    ru: `Большой релиз: структурированные ошибки с byte-offset spans, публичное
event-based API парсера, retroactive \`#[non_exhaustive]\` на error-enum-ах
для forward-compatibility.`,
    zh: `主要发布:带字节偏移 span 的结构化错误、公开的事件式解析器 API、
对错误枚举追溯应用 \`#[non_exhaustive]\` 以保证前向兼容。`,
  },
  {
    id: 'v0-1-5-003',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-1-5-004',
    join: 'block',
    en: `- \`ErrorKind\` enum (10 spec-defined variants + \`Other\`) with byte-offset
  \`span: Span\` on every variant, exposing \`(line, column, kind)\` directly
  to downstream consumers without regex-parsing the formatted message.`,
    ru: `- Enum \`ErrorKind\` (10 спек-определённых вариантов + \`Other\`) с
  byte-offset \`span: Span\` на каждом варианте, выставляющий
  \`(line, column, kind)\` напрямую downstream-потребителям без regex-
  парсинга форматированного сообщения.`,
    zh: `- \`ErrorKind\` 枚举(10 个规范定义变体 + \`Other\`),每个变体携带
  字节偏移 \`span: Span\`,直接向下游消费者暴露 \`(line, column, kind)\`,
  无需通过正则解析格式化消息。`,
  },
  {
    id: 'v0-1-5-005',
    join: 'block',
    common: `      pub enum ErrorKind {
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
      }`,
  },
  {
    id: 'v0-1-5-006',
    join: 'block',
    en: `- \`Error::Structured(ErrorKind)\` variant on the existing \`Error\` enum.`,
    ru: `- Вариант \`Error::Structured(ErrorKind)\` на существующем enum \`Error\`.`,
    zh: `- 现有 \`Error\` 枚举上的 \`Error::Structured(ErrorKind)\` 变体。`,
  },
  {
    id: 'v0-1-5-007',
    join: 'tight',
    en: `- \`pub struct Span { start: u32, end: u32 }\` with \`Span::new\`,
  \`Span::EMPTY\`, \`slice(input)\`, and \`line_col(input)\` (1-based line,
  0-based byte column — multi-byte UTF-8 aware via tests pinning
  Cyrillic and 🦀).`,
    ru: `- \`pub struct Span { start: u32, end: u32 }\` с \`Span::new\`,
  \`Span::EMPTY\`, \`slice(input)\`, и \`line_col(input)\` (line 1-based,
  column 0-based байтовая — multi-byte UTF-8 учтён через тесты,
  пинящие кириллицу и 🦀).`,
    zh: `- \`pub struct Span { start: u32, end: u32 }\`,提供 \`Span::new\`、
  \`Span::EMPTY\`、\`slice(input)\` 与 \`line_col(input)\`(line 从 1 起,
  column 从 0 起按字节计;多字节 UTF-8 已通过西里尔与 🦀 测试固定)。`,
  },
  {
    id: 'v0-1-5-008',
    join: 'tight',
    en: `- \`Error::line() -> Option<u32>\` and \`Error::span() -> Option<Span>\`
  convenience accessors covering every variant.`,
    ru: `- \`Error::line() -> Option<u32>\` и \`Error::span() -> Option<Span>\` —
  convenience-accessors, покрывающие каждый вариант.`,
    zh: `- \`Error::line() -> Option<u32>\` 和 \`Error::span() -> Option<Span>\` —
  覆盖每个变体的便捷访问器。`,
  },
  {
    id: 'v0-1-5-009',
    join: 'tight',
    en: `- \`pub mod thin\` — public event-based parser API:
  \`ktav::parse_events(input, callback)\` invoking the supplied
  \`FnMut(ParseEvent<'_>)\` for each event borrowed from the input.`,
    ru: `- \`pub mod thin\` — публичное event-based API парсера:
  \`ktav::parse_events(input, callback)\` вызывает поставленный
  \`FnMut(ParseEvent<'_>)\` для каждого события, заимствованного из
  input.`,
    zh: `- \`pub mod thin\` —— 公开的事件式解析器 API:
  \`ktav::parse_events(input, callback)\` 对从 input 借用的每个事件
  调用所提供的 \`FnMut(ParseEvent<'_>)\`。`,
  },
  {
    id: 'v0-1-5-010',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  \`ParseEvent\` is a \`#[non_exhaustive]\` enum with 10 variants
  (\`Null\`, \`Bool\`, \`Integer\`, \`Float\`, \`Str\`, \`Key\`, \`BeginObject\`,
  \`EndObject\`, \`BeginArray\`, \`EndArray\`).`,
    ru: `\`ParseEvent\` — \`#[non_exhaustive]\` enum с 10 вариантами
  (\`Null\`, \`Bool\`, \`Integer\`, \`Float\`, \`Str\`, \`Key\`, \`BeginObject\`,
  \`EndObject\`, \`BeginArray\`, \`EndArray\`).`,
    zh: `\`ParseEvent\` 是
  \`#[non_exhaustive]\` 枚举,有 10 个变体。`,
  },
  {
    id: 'v0-1-5-011',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The internal bumpalo arena
  stays private — the public API does not leak the arena type.`,
    ru: `Внутренняя bumpalo arena
  остаётся приватной — публичный API не утекает тип арены.`,
    zh: `内部 bumpalo arena
  保持私有 —— 公共 API 不泄露 arena 类型。`,
  },
  {
    id: 'v0-1-5-012',
    join: 'tight',
    en: `- Crate-level runnable doctest in \`src/lib.rs\` demonstrating both
  \`Error::Structured\` matching with \`Span::slice\` and the
  \`parse_events\` callback shape.`,
    ru: `- Crate-level runnable doctest в \`src/lib.rs\` демонстрирует и
  matching \`Error::Structured\` со \`Span::slice\`, и форму
  \`parse_events\` callback.`,
    zh: `- \`src/lib.rs\` 的 crate 级可运行 doctest,演示 \`Error::Structured\`
  匹配配合 \`Span::slice\` 以及 \`parse_events\` 回调形态。`,
  },
  {
    id: 'v0-1-5-013',
    join: 'tight',
    en: `- Six new top-level test files:
  \`tests/error_format.rs\` — Display-string regression net (canonical
  pinning for the 7 categories that LSP / bindings rely on);
  \`tests/structured_errors.rs\` — variant identity + (line, span) byte
  ranges per spec invalid fixture;
  \`tests/error_spans.rs\` — span byte-range semantics + \`Span::slice\`
  and \`Span::line_col\` edge cases (UTF-8 multi-byte, char-boundary
  rounding);
  \`tests/error_accessors.rs\` — every \`Error\` variant tested for
  \`line()\` / \`span()\` returning \`Some\` / \`None\` as documented;
  \`tests/non_exhaustive.rs\` — wildcard-arm reachability proof for
  \`Error\` and \`ErrorKind\`;
  \`tests/thin_public.rs\` — event sequencing, nested compounds, marker
  items, error propagation, borrow contract.`,
    ru: `- Шесть новых top-level test-файлов:
  \`tests/error_format.rs\` — регрессионная сеть по Display-строкам
  (канонический пиннинг 7 категорий, на которые опираются LSP и
  биндинги);
  \`tests/structured_errors.rs\` — идентичность варианта плюс диапазоны
  (line, span) в байтах на каждую невалидную фикстуру спеки;
  \`tests/error_spans.rs\` — семантика байтовых диапазонов span плюс
  краевые случаи \`Span::slice\` и \`Span::line_col\` (многобайтовый UTF-8,
  округление до границы символа);
  \`tests/error_accessors.rs\` — каждый вариант \`Error\` проверен на то,
  что \`line()\` / \`span()\` возвращают \`Some\` / \`None\` как задокументировано;
  \`tests/non_exhaustive.rs\` — доказательство достижимости
  wildcard-ветки для \`Error\` и \`ErrorKind\`;
  \`tests/thin_public.rs\` — последовательность событий, вложенные
  составные, marker-элементы, распространение ошибок, контракт
  заимствования.`,
    zh: `- 六个新顶层测试文件:
  \`tests/error_format.rs\` —— Display 字符串的回归网(为 LSP 与绑定所
  依赖的 7 个类别做规范化钉定);
  \`tests/structured_errors.rs\` —— 按规范的每个无效 fixture 校验变体
  身份以及 (line, span) 字节范围;
  \`tests/error_spans.rs\` —— span 字节范围语义,以及 \`Span::slice\` 与
  \`Span::line_col\` 的边界情形(多字节 UTF-8、按字符边界取整);
  \`tests/error_accessors.rs\` —— 逐一验证每个 \`Error\` 变体的
  \`line()\` / \`span()\` 是否如文档所述返回 \`Some\` / \`None\`;
  \`tests/non_exhaustive.rs\` —— \`Error\` 与 \`ErrorKind\` 的 wildcard 分支
  可达性证明;
  \`tests/thin_public.rs\` —— 事件序列、嵌套复合、marker 元素、错误传播、
  借用契约。`,
  },
  {
    id: 'v0-1-5-014',
    join: 'tight',
    en: `- Synthetic Criterion benchmarks under \`benches/\` covering parse
  perf at small_1k / medium_50k / large_500k workloads on both
  success and error paths.`,
    ru: `- Синтетические Criterion-бенчи под \`benches/\` покрывающие parse-perf
  на small_1k / medium_50k / large_500k workloads, на success и error
  путях.`,
    zh: `- \`benches/\` 下的合成 Criterion 基准,覆盖 small_1k / medium_50k /
  large_500k 负载在成功路径和错误路径上的解析性能。`,
  },
  {
    id: 'v0-1-5-015',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Baseline numbers in \`bench-baseline.md\`.`,
    ru: `Baseline-числа в \`bench-baseline.md\`.`,
    zh: `baseline 数字
  位于 \`bench-baseline.md\`。`,
  },
  {
    id: 'v0-1-5-016',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-1-5-017',
    join: 'block',
    en: `- \`#[non_exhaustive]\` retroactively applied to \`Error\`, \`ErrorKind\`,
  \`ConflictKind\`, and \`CompoundKind\`.`,
    ru: `- \`#[non_exhaustive]\` retroactively применён к \`Error\`, \`ErrorKind\`,
  \`ConflictKind\` и \`CompoundKind\`.`,
    zh: `- 对 \`Error\`、\`ErrorKind\`、\`ConflictKind\`、\`CompoundKind\` 追溯应用
  \`#[non_exhaustive]\`。`,
  },
  {
    id: 'v0-1-5-018',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Future variant additions are
  no longer breaking changes for downstream \`match\`-ers, who must
  now include a \`_ =>\` arm.`,
    ru: `Будущие добавления вариантов
  больше не являются ломающим изменением для downstream-\`match\`-еров,
  которые теперь обязаны иметь arm \`_ =>\`.`,
    zh: `未来添加变体对下游的 \`match\` 不再是破坏性
  变更 —— 调用方现在必须包含 \`_ =>\` 分支。`,
  },
  {
    id: 'v0-1-5-019',
    join: 'tight',
    en: `- The parser no longer constructs \`Error::Syntax(format!(...))\` at any
  internal call site (~37 sites refactored to \`Error::Structured\`).`,
    ru: `- Парсер больше не конструирует \`Error::Syntax(format!(...))\` ни на
  одном внутреннем call-сайте (~37 сайтов перерефакторены на
  \`Error::Structured\`).`,
    zh: `- 解析器在任何内部调用点都不再构造 \`Error::Syntax(format!(...))\`
  (约 37 个站点重构为 \`Error::Structured\`)。`,
  },
  {
    id: 'v0-1-5-020',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  A regression guard test
  (\`parser_no_longer_emits_legacy_syntax_variant\`) runs 12 invalid
  inputs and fails CI loudly if anyone reintroduces the legacy
  variant inside \`src/\`.`,
    ru: `Регрессионный guard-test
  (\`parser_no_longer_emits_legacy_syntax_variant\`) гоняет 12
  невалидных входов и громко фейлит CI, если кто-то реинтродуктит
  legacy-вариант внутри \`src/\`.`,
    zh: `回归保护测试
  (\`parser_no_longer_emits_legacy_syntax_variant\`)运行 12 个非法
  输入,若有人在 \`src/\` 内重新引入旧变体,CI 会大声失败。`,
  },
  {
    id: 'v0-1-5-021',
    join: 'tight',
    en: `- \`parser/parse_str.rs\` replaces \`str::lines()\` with a manual
  byte-walking loop maintaining a cumulative \`line_start\` counter
  so byte-offset spans can be computed at every error site without
  rescanning.`,
    ru: `- \`parser/parse_str.rs\` заменяет \`str::lines()\` на ручной
  byte-walking loop, поддерживающий cumulative-\`line_start\` счётчик —
  byte-offset spans вычисляются на каждом error-сайте без
  пересканирования.`,
    zh: `- \`parser/parse_str.rs\` 用维护累积 \`line_start\` 计数的手工字节遍历
  循环替代 \`str::lines()\`,以便在每个错误站点计算字节偏移 span 而
  无需重新扫描。`,
  },
  {
    id: 'v0-1-5-022',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`thin/event_parser.rs\` mirrors the same plumbing on
  the zero-copy path.`,
    ru: `\`thin/event_parser.rs\` зеркалит то же plumbing
  на zero-copy пути.`,
    zh: `\`thin/event_parser.rs\` 在零拷贝路径上镜像同一
  plumbing。`,
  },
  {
    id: 'v0-1-5-023',
    join: 'tight',
    en: `- \`Display for ErrorKind\` is byte-identical to the strings the parser
  previously formatted into \`Error::Syntax(...)\` for the seven
  pre-existing categories — the contract that lets every existing
  string-based caller keep working unmodified during the
  ecosystem-wide migration tracked in
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md).`,
    ru: `- \`Display for ErrorKind\` byte-identical к строкам, которые парсер
  ранее форматировал в \`Error::Syntax(...)\` для семи pre-existing
  категорий — это контракт, позволяющий каждому существующему
  string-based caller-у работать без изменений на время
  ecosystem-wide миграции, описанной в
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md).`,
    zh: `- \`Display for ErrorKind\` 与解析器先前格式化进 \`Error::Syntax(...)\`
  的字符串完全字节相同(覆盖七个先前已固定类别)—— 这是让现有
  基于字符串的调用方在
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md) 描述的生态系统
  级迁移期间保持不变工作的契约。`,
  },
  {
    id: 'v0-1-5-024',
    join: 'tight',
    en: `- Three formerly-\`Other\` shapes promoted to named \`ErrorKind\` variants:
  \`UnbalancedBracket\` (stray closer / shape mismatch),
  \`InlineNonEmptyCompound\` (\`x: {foo}\` — spec § 6.7),
  \`MissingSeparator\` (line with no \`:\`).`,
    ru: `- Три прежде-\`Other\` формы промоутированы в именованные варианты
  \`ErrorKind\`: \`UnbalancedBracket\` (lone closer / shape mismatch),
  \`InlineNonEmptyCompound\` (\`x: {foo}\` — спека § 6.7),
  \`MissingSeparator\` (строка без \`:\`).`,
    zh: `- 三个先前归入 \`Other\` 的形态升级为命名 \`ErrorKind\` 变体:
  \`UnbalancedBracket\`(孤立闭符 / 形状不匹配)、
  \`InlineNonEmptyCompound\`(\`x: {foo}\` —— 规范 § 6.7)、
  \`MissingSeparator\`(无 \`:\` 的行)。`,
  },
  {
    id: 'v0-1-5-025',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `After this promotion, \`Other\`
  contains only parser-internal invariants that no spec invalid
  fixture can trigger.`,
    ru: `После promotion-а \`Other\`
  содержит только парсер-внутренние invariants, которые ни одна
  spec invalid фикстура не триггерит.`,
    zh: `升级后,\`Other\` 仅保留没有任何
  规范非法夹具能触发的解析器内部不变量。`,
  },
  {
    id: 'v0-1-5-026',
    join: 'block',
    en: `### Performance`,
    ru: `### Производительность`,
    zh: `### 性能`,
  },
  {
    id: 'v0-1-5-027',
    join: 'block',
    en: `\`cargo bench --bench parse -- --quick\` against the 0.1.4 baseline:`,
    ru: `\`cargo bench --bench parse -- --quick\` против baseline 0.1.4:`,
    zh: `\`cargo bench --bench parse -- --quick\` 与 0.1.4 baseline 对比:`,
  },
  {
    id: 'v0-1-5-028',
    join: 'block',
    common: `|                                | 0.1.4 baseline | 0.1.5  | Δ      |
|--------------------------------|----------------|--------|--------|
| \`parse_synth/small_1k\`         | 16.1 µs        | 16.0 µs| −0.6 % |
| \`parse_synth/medium_50k\`       | 896 µs         | 663 µs | −26 %  |
| \`parse_synth/large_500k\`       | 9.49 ms        | 9.27 ms| −2.3 % |
| \`parse_synth_error/small_1k\`   | 7.5 µs         | 7.2 µs | −4.0 % |
| \`parse_synth_error/medium_50k\` | 340 µs         | 346 µs | +1.8 % |
| \`parse_synth_error/large_500k\` | 4.47 ms        | 4.50 ms| +0.7 % |`,
  },
  {
    id: 'v0-1-5-029',
    join: 'block',
    en: `Net: zero success-path regression.`,
    ru: `Итог: регрессии на success-path нет.`,
    zh: `结论:成功路径零回归。`,
  },
  {
    id: 'v0-1-5-030',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Error-path slightly faster — the
new \`Display\` impl constructs the formatted string lazily at
\`.to_string()\` time, whereas the prior \`format!(...)\` allocated a
\`String\` at every error site eagerly.`,
    ru: `Error-path немного быстрее — новая
реализация \`Display\` собирает форматированную строку лениво, в момент
\`.to_string()\`, тогда как прежний \`format!(...)\` выделял \`String\` жадно
на каждой точке ошибки.`,
    zh: `错误路径略快——新的 \`Display\` 实现在调用
\`.to_string()\` 时才惰性构造格式化字符串,而此前的 \`format!(...)\` 会在
每一处错误点即时分配一个 \`String\`。`,
  },
  {
    id: 'v0-1-5-031',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The cumulative-byte counter
that powers spans is statistically free.`,
    ru: `Счётчик накопленных байтов, на котором держатся
spans, статистически бесплатен.`,
    zh: `支撑 span 的累积字节计数器在统计上
是免费的。`,
  },
  {
    id: 'v0-1-5-032',
    join: 'block',
    en: `### Notes`,
    ru: `### Заметки`,
    zh: `### 备注`,
  },
  {
    id: 'v0-1-5-033',
    join: 'block',
    en: `- \`Error::Syntax(String)\` is preserved for backward compatibility —
  the public API stays deny-no-old-callers.`,
    ru: `- \`Error::Syntax(String)\` сохранён для обратной совместимости —
  публичный API остаётся deny-no-old-callers.`,
    zh: `- 为了向后兼容,\`Error::Syntax(String)\` 保留 —— 公共 API 仍不拒绝
  老调用方。`,
  },
  {
    id: 'v0-1-5-034',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Removal is deferred to
  ktav 1.0.`,
    ru: `Удаление отложено до
  ktav 1.0.`,
    zh: `移除推迟到 ktav 1.0。`,
  },
  {
    id: 'v0-1-5-035',
    join: 'tight',
    en: `- Test count: 332 (0.1.4) → 391 (+59) plus 1 new doctest.`,
    ru: `- Тестов: 332 (0.1.4) → 391 (+59) плюс 1 новый doctest.`,
    zh: `- 测试数:332 (0.1.4) → 391 (+59),外加 1 个新 doctest。`,
  },
  {
    id: 'v0-1-5-036',
    join: 'tight',
    en: `- The cabi/binding migration to consume \`ErrorKind\` over the FFI
  boundary is tracked separately in
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md) and ships as a
  coordinated ecosystem 0.2.0.`,
    ru: `- Миграция cabi/биндингов на потребление \`ErrorKind\` через FFI
  трекается отдельно в
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md) и идёт как
  coordinated ecosystem 0.2.0.`,
    zh: `- cabi/绑定迁移以通过 FFI 边界消费 \`ErrorKind\` 的工作单独记录在
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md),并作为协调发布
  的生态系统 0.2.0 一并交付。`,
  },
  {
    id: 'v0-1-5-037',
    join: 'block',
    en: `### SemVer note`,
    ru: `### SemVer-замечание`,
    zh: `### SemVer 说明`,
  },
  {
    id: 'v0-1-5-038',
    join: 'block',
    en: `Adding \`#[non_exhaustive]\` to a previously-unmarked enum (\`Error\`,
\`ConflictKind\`, \`CompoundKind\`) is, per the
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
a breaking change that would normally require a major bump (0.2.0).`,
    ru: `Добавление \`#[non_exhaustive]\` к ранее непомеченным enum-ам (\`Error\`,
\`ConflictKind\`, \`CompoundKind\`) — согласно
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive)
— breaking change, требующий major-bump-а (0.2.0).`,
    zh: `按
[Cargo SemVer 参考](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
为先前未标注的枚举(\`Error\`、\`ConflictKind\`、\`CompoundKind\`)添加
\`#[non_exhaustive]\` 是破坏性变更,通常需要主版本号 bump(0.2.0)。`,
  },
  {
    id: 'v0-1-5-039',
    join: { en: 'tight', ru: 'flow', zh: 'tight' },
    en: `This release ships as **0.1.5** intentionally:`,
    ru: `Этот релиз
выпускается как **0.1.5** намеренно:`,
    zh: `本次发布有意作为 **0.1.5** 推出:`,
  },
  {
    id: 'v0-1-5-040',
    join: 'block',
    en: `1. Pre-1.0 Cargo convention permits breaking changes on any bump,
   including patches.`,
    ru: `1. Pre-1.0 Cargo-конвенция допускает breaking-изменения на любом
   bump-е, включая патчи.`,
    zh: `1. Pre-1.0 的 Cargo 惯例允许在任何 bump(包括 patch)上做破坏性
   变更。`,
  },
  {
    id: 'v0-1-5-041',
    join: 'tight',
    en: `2. All known downstream consumers of \`ktav::Error\` (the six language
   bindings under \`ktav-lang/\`) call \`Err(e) => e.to_string()\` only.`,
    ru: `2. Все известные downstream-потребители \`ktav::Error\` (шесть
   языковых биндингов под \`ktav-lang/\`) делают только
   \`Err(e) => e.to_string()\`.`,
    zh: `2. \`ktav::Error\` 所有已知的下游消费者(\`ktav-lang/\` 下的六个语言
   绑定)均仅调用 \`Err(e) => e.to_string()\`。`,
  },
  {
    id: 'v0-1-5-042',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `   No exhaustive \`match err { Error::Io(_) => …, Error::Syntax(_)
   => …, Error::Message(_) => … }\` patterns exist in the ecosystem
   that this change would silently break.`,
    ru: `Exhaustive \`match err { Error::Io(_)
   => …, Error::Syntax(_) => …, Error::Message(_) => … }\` в
   экосистеме нет — ломать тихо нечего.`,
    zh: `生态系统中不存在会被
   该变更悄悄破坏的穷尽 \`match err { Error::Io(_) => …,
   Error::Syntax(_) => …, Error::Message(_) => … }\` 模式。`,
  },
  {
    id: 'v0-1-5-043',
    join: 'tight',
    en: `3. The seven canonical-category Display strings remain byte-identical
   to 0.1.4, so any hypothetical out-of-tree consumer doing string
   matching keeps working unmodified.`,
    ru: `3. Display-строки семи канонических категорий byte-identical к
   0.1.4, поэтому любой гипотетический out-of-tree-потребитель,
   делающий string matching, продолжает работать без изменений.`,
    zh: `3. 七个标准类别的 Display 字符串与 0.1.4 字节相同,因此任何在
   tree 之外做字符串匹配的假想消费者也无需修改即可继续工作。`,
  },
  {
    id: 'v0-1-5-044',
    join: 'block',
    en: `If your code does keep an exhaustive match over \`ktav::Error\` and
this release breaks it, add an \`_ => …\` arm.`,
    ru: `Если ваш код всё-таки держит exhaustive \`match\` по \`ktav::Error\` и
этот релиз его ломает — добавьте arm \`_ => …\`.`,
    zh: `如果你的代码确实保留了对 \`ktav::Error\` 的穷尽 \`match\` 而本次发布
将其破坏 —— 请添加 \`_ => …\` 分支。`,
  },
  {
    id: 'v0-1-5-045',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `That arm is now
required forever and will not need to change again as future
variants are added.`,
    ru: `Этот arm теперь
обязателен навсегда и больше не потребует изменений при добавлении
будущих вариантов.`,
    zh: `该分支从此永久必须存在,且不会
随未来变体的添加而再次需要变更。`,
  },
];
