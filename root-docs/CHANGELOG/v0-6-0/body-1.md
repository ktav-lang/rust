>>>>> lang=en
## [0.6.0] — 2026-06-01

Implements Ktav specification 0.6.0. Adds **key escaping**: keys now
process the §3.7 escape set, and two new escapes — `\.` and `\:` —
allow literal dot/colon characters inside a key segment.

### Breaking

- A literal backslash in a key now requires `\\`. Previously the
  parser treated `\` in a key as an opaque content byte (no escape
  processing). Source files that contain a single `\` in a key must
  double it to keep the same key bytes. Backslashes in values are
  unchanged.

### Added

- Escape table grows from 8 to 10 sequences: the existing eight
  (`\\`, `\,`, `\}`, `\]`, `\{`, `\[`, `\n`, `\r`) plus the two new
  key-oriented ones — `\.` (literal dot in a key segment, does NOT
  split the dotted path) and `\:` (literal colon, does NOT act as the
  pair separator). The two new escapes are valid (redundant) in value
  contexts too.
- The escape-aware key scanner now treats the **first unescaped** `:`
  (or `::`) as the pair separator and splits dotted paths on
  **unescaped** `.` only. Inline-compound keys (`{a\.b: 1}`) follow
  the same rules.
- Render path re-escapes `\`, `.`, `:` in every emitted key segment,
  guaranteeing parse → render → parse identity for keys containing
  literal dots or colons.
- Zero-copy event path keeps borrowing the source slice for a key
  segment that has no `\`; segments with escapes are decoded into the
  bump arena so the event's `&'a str` lifetime is preserved.

### Errors

- `\X` in a key with `X` not in the ten-escape set is now a
  `BadEscapeSequence`. `\.` and `\:` are no longer `BadEscapeSequence`
  in any context.

>>>>> lang=ru
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

>>>>> lang=zh
## [0.6.0] —— 2026-06-01

实现 Ktav 规范 0.6.0。新增 **键转义**:键现在处理 §3.7 转义集,并
新增两条转义 —— `\.` 与 `\:` —— 允许在键段内出现字面意义的点 / 冒号。

### 破坏性变更

- 键中的字面 `\` 现在需要写作 `\\`。此前解析器把键中的 `\` 当作
  无转义的内容字节。源文件中含单个 `\` 的键需要改为双反斜杠以保
  持相同的键字节。值的处理不变。

### 新增

- 转义表由 8 条扩展到 10 条:原有 8 条(`\\`、`\,`、`\}`、`\]`、
  `\{`、`\[`、`\n`、`\r`)之上新增 `\.`(键段中的字面点 —— 不分割
  路径)与 `\:`(字面冒号 —— 不作为键/值分隔符)。两条新转义在值
  上下文中亦合法(冗余)。
- 键扫描器支持转义:首个 **未转义** 的 `:`(或 `::`)为对分隔符;
  点分路径仅在 **未转义** 的 `.` 处分割。内联 compound
  (`{a\.b: 1}`)采用同样规则。
- 渲染器在输出每个键段时回写转义 `\`、`.`、`:`,保证含字面点/冒
  号的键的 parse → render → parse 同一性。
- 零拷贝事件路径下,不含 `\` 的键段继续从源缓冲区借用;含转义的
  键段解码到 bump-arena —— `&'a str` 生命周期保持不变。

### 错误

- 键中 `\X`(`X` 不在十条转义之列)现在抛出 `BadEscapeSequence`。
  `\.` 与 `\:` 在任何上下文均不再是 `BadEscapeSequence`。

