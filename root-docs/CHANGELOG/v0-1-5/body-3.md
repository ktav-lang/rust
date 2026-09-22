>>>>> lang=en
### Changed

- `#[non_exhaustive]` retroactively applied to `Error`, `ErrorKind`,
  `ConflictKind`, and `CompoundKind`. Future variant additions are
  no longer breaking changes for downstream `match`-ers, who must
  now include a `_ =>` arm.
- The parser no longer constructs `Error::Syntax(format!(...))` at any
  internal call site (~37 sites refactored to `Error::Structured`).
  A regression guard test
  (`parser_no_longer_emits_legacy_syntax_variant`) runs 12 invalid
  inputs and fails CI loudly if anyone reintroduces the legacy
  variant inside `src/`.
- `parser/parse_str.rs` replaces `str::lines()` with a manual
  byte-walking loop maintaining a cumulative `line_start` counter
  so byte-offset spans can be computed at every error site without
  rescanning. `thin/event_parser.rs` mirrors the same plumbing on
  the zero-copy path.
- `Display for ErrorKind` is byte-identical to the strings the parser
  previously formatted into `Error::Syntax(...)` for the seven
  pre-existing categories — the contract that lets every existing
  string-based caller keep working unmodified during the
  ecosystem-wide migration tracked in
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md).
- Three formerly-`Other` shapes promoted to named `ErrorKind` variants:
  `UnbalancedBracket` (stray closer / shape mismatch),
  `InlineNonEmptyCompound` (`x: {foo}` — spec § 6.7),
  `MissingSeparator` (line with no `:`). After this promotion, `Other`
  contains only parser-internal invariants that no spec invalid
  fixture can trigger.

>>>>> lang=ru
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

>>>>> lang=zh
### 变更

- 对 `Error`、`ErrorKind`、`ConflictKind`、`CompoundKind` 追溯应用
  `#[non_exhaustive]`。未来添加变体对下游的 `match` 不再是破坏性
  变更 —— 调用方现在必须包含 `_ =>` 分支。
- 解析器在任何内部调用点都不再构造 `Error::Syntax(format!(...))`
  (约 37 个站点重构为 `Error::Structured`)。回归保护测试
  (`parser_no_longer_emits_legacy_syntax_variant`)运行 12 个非法
  输入,若有人在 `src/` 内重新引入旧变体,CI 会大声失败。
- `parser/parse_str.rs` 用维护累积 `line_start` 计数的手工字节遍历
  循环替代 `str::lines()`,以便在每个错误站点计算字节偏移 span 而
  无需重新扫描。`thin/event_parser.rs` 在零拷贝路径上镜像同一
  plumbing。
- `Display for ErrorKind` 与解析器先前格式化进 `Error::Syntax(...)`
  的字符串完全字节相同(覆盖七个先前已固定类别)—— 这是让现有
  基于字符串的调用方在
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md) 描述的生态系统
  级迁移期间保持不变工作的契约。
- 三个先前归入 `Other` 的形态升级为命名 `ErrorKind` 变体:
  `UnbalancedBracket`(孤立闭符 / 形状不匹配)、
  `InlineNonEmptyCompound`(`x: {foo}` —— 规范 § 6.7)、
  `MissingSeparator`(无 `:` 的行)。升级后,`Other` 仅保留没有任何
  规范非法夹具能触发的解析器内部不变量。

