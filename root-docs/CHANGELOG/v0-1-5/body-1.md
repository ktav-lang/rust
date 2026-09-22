>>>>> lang=en
## [0.1.5] — 2026-05-01

Major release: structured errors with byte-offset spans, public
event-based parser API, `#[non_exhaustive]` retroactively applied to
the error enums for forward-compatibility.

### Added

- `ErrorKind` enum (10 spec-defined variants + `Other`) with byte-offset
  `span: Span` on every variant, exposing `(line, column, kind)` directly
  to downstream consumers without regex-parsing the formatted message.

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

>>>>> lang=ru
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

>>>>> lang=zh
## [0.1.5] —— 2026-05-01

主要发布:带字节偏移 span 的结构化错误、公开的事件式解析器 API、
对错误枚举追溯应用 `#[non_exhaustive]` 以保证前向兼容。

### 新增

- `ErrorKind` 枚举(10 个规范定义变体 + `Other`),每个变体携带
  字节偏移 `span: Span`,直接向下游消费者暴露 `(line, column, kind)`,
  无需通过正则解析格式化消息。

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

