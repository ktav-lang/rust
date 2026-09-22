>>>>> lang=en
## [0.6.3] — 2026-08-23

### Fixed

- **Keys accept all ten § 3.7 escape sequences, not three.** § 3.7 / § 4
  define `\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`, `\r`, `\.`, `\:` as
  valid in a key; only `\\`, `\.` and `\:` actually worked. `a\,b: 1`
  and the six other forms were rejected with `InvalidKey`. The cause
  was ordering: the key was escape-decoded first, then re-validated,
  and validation blanket-rejected `,` `{` `}` `[` `]` `LF` `CR` in the
  decoded bytes without distinguishing a byte that arrived through a
  legitimate escape from one written raw. Validation now runs on the
  raw (pre-decode) segment, so raw structural bytes stay rejected while
  their escaped forms pass ([#7](https://github.com/ktav-lang/rust/issues/7)).
- **The event parser and the tree parser no longer disagree on keys.**
  `src/thin/event_parser.rs` carried its own copy of the key check with
  the same post-decode bug, plus a second divergence of its own: it
  omitted `LF`/`CR` from the forbidden set, so `parse()` and
  `from_str::<T>()` already accepted different key sets. The duplicate
  is gone; both front ends now share one validator.
- **The writer emits the full escape set.** `push_escaped_key_segment`
  escaped only `\\`, `.` and `:`, so a `Value` built through the API
  (rather than parsed) with a key containing `,` `{` `}` `[` `]` `LF`
  or `CR` serialised to a document that did not parse back — on all
  three writer surfaces, which share the helper (`emit_canonical`,
  serde `to_string`, `render` / `to_string_force_strings`). `(` and `)`
  are deliberately left alone: § 3.7 defines no escape for them, so a
  key containing one stays unrepresentable
  ([#5](https://github.com/ktav-lang/rust/issues/5) covers refusing
  those explicitly, on the `0.7` branch).

>>>>> lang=ru
## [0.6.3] — 2026-08-23

### Исправлено

- **Ключи принимают все десять escape-последовательностей § 3.7, а не
  три.** § 3.7 / § 4 объявляют допустимыми в ключе `\\`, `\,`, `\}`,
  `\]`, `\{`, `\[`, `\n`, `\r`, `\.`, `\:`; реально работали только
  `\\`, `\.` и `\:`. Запись `a\,b: 1` и шесть других форм отвергались
  с `InvalidKey`. Причина — в порядке проверок: ключ сначала
  декодировался, а потом валидировался повторно, и валидация
  безоговорочно запрещала `,` `{` `}` `[` `]` `LF` `CR` в уже
  декодированных байтах, не различая байт, пришедший через легитимный
  escape, и байт, записанный сырым. Теперь валидация работает по
  сырому (до декодирования) сегменту: сырые структурные байты
  по-прежнему запрещены, а их экранированные формы проходят
  ([#7](https://github.com/ktav-lang/rust/issues/7)).
- **Event-парсер и tree-парсер больше не расходятся по ключам.**
  В `src/thin/event_parser.rs` жила собственная копия проверки ключа с
  той же ошибкой порядка плюс собственное расхождение: в её списке
  запрещённых байтов не было `LF`/`CR`, поэтому `parse()` и
  `from_str::<T>()` уже принимали разные наборы ключей. Дубликат
  удалён; оба фронтенда используют один валидатор.
- **Writer выводит полный набор escape.** `push_escaped_key_segment`
  экранировал только `\\`, `.` и `:`, поэтому `Value`, собранный через
  API (а не разбором), с ключом, содержащим `,` `{` `}` `[` `]` `LF`
  или `CR`, сериализовался в документ, который обратно не разбирался, —
  на всех трёх поверхностях записи, использующих этот хелпер
  (`emit_canonical`, serde `to_string`, `render` /
  `to_string_force_strings`). `(` и `)` намеренно не трогаем: § 3.7 не
  определяет для них escape, поэтому ключ с такой скобкой остаётся
  непредставимым (явный отказ по ним — предмет
  [#5](https://github.com/ktav-lang/rust/issues/5), в ветке `0.7`).

>>>>> lang=zh
## [0.6.3] —— 2026-08-23

### 修复

- **键现在接受 § 3.7 全部十种转义序列，而非三种。** § 3.7 / § 4 规定
  `\\`、`\,`、`\}`、`\]`、`\{`、`\[`、`\n`、`\r`、`\.`、`\:` 在键中均
  合法；但实际只有 `\\`、`\.` 和 `\:` 可用。`a\,b: 1` 及其余六种形式
  都会以 `InvalidKey` 被拒绝。根因在于顺序：键先被转义解码，随后再次
  校验，而校验会一律拒绝解码后字节中的 `,` `{` `}` `[` `]` `LF` `CR`，
  并不区分该字节是经由合法转义而来，还是原样写入的。现在校验改在
  原始（解码前）的段上进行：原样的结构字节依旧被拒绝，而其转义形式
  得以通过（[#7](https://github.com/ktav-lang/rust/issues/7)）。
- **事件解析器与树解析器不再对键产生分歧。**
  `src/thin/event_parser.rs` 自带一份键校验副本，除了同样的顺序缺陷，
  还有它自己的偏差：其禁止集合遗漏了 `LF`/`CR`，因此 `parse()` 与
  `from_str::<T>()` 早已接受不同的键集合。该副本已删除；两个前端现在
  共用同一个校验器。
- **写入端输出完整的转义集合。** `push_escaped_key_segment` 只转义
  `\\`、`.` 和 `:`，因此通过 API 构造（而非解析得到）的 `Value`，若键
  中含有 `,` `{` `}` `[` `]` `LF` 或 `CR`，序列化出的文档将无法被解析
  回来 —— 三个写入面因共用该辅助函数而全部受影响（`emit_canonical`、
  serde `to_string`、`render` / `to_string_force_strings`）。`(` 与 `)`
  刻意保持原状：§ 3.7 未为其定义转义，因此含有这两个字符的键仍不可
  表示（对其显式拒绝属于 [#5](https://github.com/ktav-lang/rust/issues/5)，
  在 `0.7` 分支处理）。

