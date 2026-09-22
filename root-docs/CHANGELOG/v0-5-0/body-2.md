>>>>> lang=en
- **Inline compounds** `{k: v, …}` / `[i, …]` (spec § 5.8) with
  trailing comma, mid-value brace literal (§ 5.8.5), and nesting
  depth limit of 128.
- **Eight escape sequences** in inline scalars: `\\`, `\,`, `\}`,
  `\]`, `\{`, `\[`, `\n`, `\r` (spec § 3.7).
- **Number literal grammar** — `0x`, `0o`, `0b`, decimal, underscore
  separators; i64 overflow falls back to String (spec § 3.6).
- **`emit_canonical()`** — spec § 5.9 normative writer output,
  byte-deterministic across implementations.
- **`src/parser/inline.rs`** — inline-compound parser.
- **`src/render/canonical.rs`** — canonical writer.
- New error variants: `UnterminatedInlineCompound`,
  `MalformedInlineCompound`, `BadEscapeSequence`,
  `OrphanLineAfterTopLevelInline`.
- Triple-test conformance harness (`tests/spec_conformance.rs`):
  93 valid + 31 invalid fixtures from `spec/versions/0.5/tests/`.
- Parser fast-paths: plain-decimal integers, no-underscore floats,
  ryu-reuse, first-byte fast-reject, LF-only line splitting,
  pre-sized Bump arena.

### Changed

- License: `MIT` → `MIT OR Apache-2.0`.
- Spec submodule pinned to `v0.5.0` (`4d0a8aa`).
- Doctests disabled (`[lib] doctest = false`); examples remain as
  `text` blocks in doc comments.

>>>>> lang=ru
- **Инлайн-составные** `{k: v, …}` / `[i, …]` (спек § 5.8) с хвостовой
  запятой, литеральной скобкой в середине значения (§ 5.8.5) и
  ограничением глубины вложенности в 128.
- **Восемь escape-последовательностей** в инлайн-скалярах: `\\`, `\,`,
  `\}`, `\]`, `\{`, `\[`, `\n`, `\r` (спек § 3.7).
- **Грамматика числовых литералов** — `0x`, `0o`, `0b`, десятичные,
  разделители-подчёркивания; переполнение i64 откатывается в String
  (спек § 3.6).
- **`emit_canonical()`** — нормативный вывод writer-а по спек § 5.9,
  байт-детерминированный между реализациями.
- **`src/parser/inline.rs`** — парсер инлайн-составных.
- **`src/render/canonical.rs`** — канонический writer.
- Новые варианты ошибок: `UnterminatedInlineCompound`,
  `MalformedInlineCompound`, `BadEscapeSequence`,
  `OrphanLineAfterTopLevelInline`.
- Тройной conformance-harness (`tests/spec_conformance.rs`):
  93 валидных + 31 невалидная фикстура из `spec/versions/0.5/tests/`.
- Быстрые пути парсера: простые десятичные целые, float без
  подчёркиваний, переиспользование ryu, быстрый отказ по первому
  байту, разбиение строк только по LF, преднастроенная Bump-арена.

### Изменено

- Лицензия: `MIT` → `MIT OR Apache-2.0`.
- Сабмодуль спеки закреплён на `v0.5.0` (`4d0a8aa`).
- Doctests отключены (`[lib] doctest = false`); примеры остаются
  блоками `text` в doc-комментариях.

>>>>> lang=zh
- **内联复合** `{k: v, …}` / `[i, …]`（规范 § 5.8），支持尾随逗号、
  值中部的花括号字面量（§ 5.8.5），嵌套深度上限 128。
- 内联标量中的**八种转义序列**：`\\`、`\,`、`\}`、`\]`、`\{`、`\[`、
  `\n`、`\r`（规范 § 3.7）。
- **数字字面量文法** —— `0x`、`0o`、`0b`、十进制、下划线分隔符；
  i64 溢出回退为 String（规范 § 3.6）。
- **`emit_canonical()`** —— 规范 § 5.9 规定的 writer 输出，跨实现
  字节确定。
- **`src/parser/inline.rs`** —— 内联复合解析器。
- **`src/render/canonical.rs`** —— 规范化 writer。
- 新增错误变体：`UnterminatedInlineCompound`、
  `MalformedInlineCompound`、`BadEscapeSequence`、
  `OrphanLineAfterTopLevelInline`。
- 三重 conformance 测试台（`tests/spec_conformance.rs`）：取自
  `spec/versions/0.5/tests/` 的 93 个有效 + 31 个无效 fixture。
- 解析器快速路径：纯十进制整数、无下划线浮点、ryu 复用、首字节快速
  拒绝、仅按 LF 切行、预分配的 Bump arena。

### 变更

- 许可证：`MIT` → `MIT OR Apache-2.0`。
- 规范 submodule 固定到 `v0.5.0`（`4d0a8aa`）。
- 关闭 doctest（`[lib] doctest = false`）；示例以 `text` 块形式保留在
  文档注释中。

