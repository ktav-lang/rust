>>>>> lang=en
## [0.8.1] — 2026-10-21

### Fixed

- Uppercase `0X` / `0O` / `0B` prefixes remain Strings, including signed and inline forms. Strict parsing accepts them as Strings, and formatting no longer rewrites them into numbers. Lowercase numeric prefixes and uppercase hex digits remain supported.
- An explicit root left open at EOF is rejected as `UnclosedCompound` by the owned parser, thin parser and formatter. Closed explicit roots and implicit roots retain their behaviour.
- C ABI source operations report `InvalidUtf8`, not `Message`, with a 1-based line counted over original bytes and a non-empty covering span. C ABI EOF errors with spans obtain the opener line.

### Verification and compatibility

- The pinned spec corpus includes 228 valid and 90 invalid fixtures. Invalid fixtures also exercise the formatter and C ABI; focused tests cover UTF-8 locations.
- Public Rust error variants, exported signatures and C ABI version are unchanged. These are corrections to existing specification requirements, not new language rules.
- **Known limitation:** direct Rust `InvalidUtf8` errors still have `line() == None` and an empty insertion-point span; direct EOF `UnclosedCompound` errors still lack a line. Public `ErrorEnvelope::from_error` preserves those accessor values. Tracked in [#15](https://github.com/ktav-lang/rust/issues/15). The corrected diagnostic locations apply to C ABI source operations; this release does not claim full diagnostic conformance of the direct Rust API.

>>>>> lang=ru
## [0.8.1] — 2026-10-21

### Исправлено

- Префиксы `0X` / `0O` / `0B` в верхнем регистре остаются String, включая формы со знаком и inline-формы. Строгий разбор принимает их как String, а форматирование больше не превращает их в числа. Числовые префиксы в нижнем регистре и hex-цифры в верхнем регистре по-прежнему поддерживаются.
- Незакрытый к EOF явный корень отвергается как `UnclosedCompound` в owned-парсере, thin-парсере и formatter. Поведение закрытых явных и неявных корней сохранено.
- Source-операции C ABI возвращают `InvalidUtf8`, а не `Message`, с номером строки от 1 по исходным байтам и непустым покрывающим span. EOF-ошибки C ABI со span получают строку opener.

### Проверка и совместимость

- Закреплённый корпус спеки включает 228 valid и 90 invalid fixtures. Invalid-fixtures также проверяют formatter и C ABI; отдельные тесты покрывают позиции UTF-8-ошибок.
- Публичные варианты Rust-ошибок, экспортируемые сигнатуры и версия C ABI не меняются. Исправлены нарушения существующих требований спеки, новые правила языка не вводятся.
- **Известное ограничение:** прямые Rust-ошибки `InvalidUtf8` сохраняют `line() == None` и пустой span точки вставки; прямые EOF-ошибки `UnclosedCompound` по-прежнему не имеют строки. Публичный `ErrorEnvelope::from_error` сохраняет значения этих accessor. Отслеживается в [#15](https://github.com/ktav-lang/rust/issues/15). Исправленные позиции применяются к source-операциям C ABI; выпуск не заявляет полного соответствия диагностического контракта прямого Rust API.

>>>>> lang=zh
## [0.8.1] — 2026-10-21

### 修复

- 大写 `0X` / `0O` / `0B` 前缀保持 String 类型，包括带符号和 inline 形式。严格解析将其作为 String 接受，格式化不再将其改写为数值。小写数值前缀和大写十六进制数字仍受支持。
- Owned 解析器、thin 解析器和 formatter 将到 EOF 仍未闭合的显式根拒绝为 `UnclosedCompound`。已闭合显式根和隐式根的行为保持不变。
- C ABI 源操作将无效 UTF-8 报告为 `InvalidUtf8`，而非 `Message`，并提供按原始字节计算的从 1 开始的行号及非空覆盖 span。带 span 的 C ABI EOF 错误获得开启符所在行。

### 验证与兼容性

- 固定的规范语料库包含 228 个 valid 和 90 个 invalid fixtures。Invalid fixtures 也覆盖 formatter 和 C ABI；专项测试覆盖 UTF-8 错误位置。
- 公开 Rust 错误变体、导出签名和 C ABI 版本不变。这些修复落实已有规范要求，并非新增语言规则。
- **已知限制：**直接 Rust `InvalidUtf8` 错误仍返回 `line() == None` 和空的插入点 span；直接 EOF `UnclosedCompound` 错误仍无行号。公开的 `ErrorEnvelope::from_error` 保留这些 accessor 值。后续工作见 [#15](https://github.com/ktav-lang/rust/issues/15)。修正后的诊断位置适用于 C ABI 源操作；本版本不声称直接 Rust API 完全符合诊断契约。

