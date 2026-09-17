// Changelog units for 0.6.0.
export const v0_6_0 = [
  {
    id: 'v0-6-0-001',
    join: 'block',
    en: `## [0.6.0] — 2026-06-01`,
    ru: `## [0.6.0] — 2026-06-01`,
    zh: `## [0.6.0] —— 2026-06-01`,
  },
  {
    id: 'v0-6-0-002',
    join: 'block',
    en: `Implements Ktav specification 0.6.0. Adds **key escaping**: keys now
process the §3.7 escape set, and two new escapes — \`\\.\` and \`\\:\` —
allow literal dot/colon characters inside a key segment.`,
    ru: `Реализация спецификации Ktav 0.6.0. Добавлено **экранирование в
ключах**: ключи теперь обрабатывают набор escape-последовательностей
из §3.7, и два новых escape — \`\\.\` и \`\\:\` — позволяют использовать
буквальные точку/двоеточие внутри сегмента ключа.`,
    zh: `实现 Ktav 规范 0.6.0。新增 **键转义**:键现在处理 §3.7 转义集,并
新增两条转义 —— \`\\.\` 与 \`\\:\` —— 允许在键段内出现字面意义的点 / 冒号。`,
  },
  {
    id: 'v0-6-0-003',
    join: 'block',
    en: `### Breaking`,
    ru: `### Breaking`,
    zh: `### 破坏性变更`,
  },
  {
    id: 'v0-6-0-004',
    join: 'block',
    en: `- A literal backslash in a key now requires \`\\\\\`.`,
    ru: `- Буквальный символ \`\\\` в ключе теперь требует \`\\\\\`.`,
    zh: `- 键中的字面 \`\\\` 现在需要写作 \`\\\\\`。`,
  },
  {
    id: 'v0-6-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Previously the
  parser treated \`\\\` in a key as an opaque content byte (no escape
  processing).`,
    ru: `Раньше парсер
  трактовал \`\\\` в ключе как обычный байт без обработки escape.`,
    zh: `此前解析器把键中的 \`\\\` 当作
  无转义的内容字节。`,
  },
  {
    id: 'v0-6-0-006',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `Source files that contain a single \`\\\` in a key must
  double it to keep the same key bytes.`,
    ru: `  Файлы-источники с одиночными \`\\\` в ключах нужно удвоить, чтобы
  сохранить те же байты ключа.`,
    zh: `源文件中含单个 \`\\\` 的键需要改为双反斜杠以保
  持相同的键字节。`,
  },
  {
    id: 'v0-6-0-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Backslashes in values are
  unchanged.`,
    ru: `Значения не затронуты.`,
    zh: `值的处理不变。`,
  },
  {
    id: 'v0-6-0-008',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-6-0-009',
    join: 'block',
    en: `- Escape table grows from 8 to 10 sequences: the existing eight
  (\`\\\\\`, \`\\,\`, \`\\}\`, \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\`) plus the two new
  key-oriented ones — \`\\.\` (literal dot in a key segment, does NOT
  split the dotted path) and \`\\:\` (literal colon, does NOT act as the
  pair separator).`,
    ru: `- Таблица escape расширена с 8 до 10 последовательностей: к восьми
  существующим (\`\\\\\`, \`\\,\`, \`\\}\`, \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\`)
  добавлены \`\\.\` (буквальная точка в сегменте ключа — НЕ разделяет
  путь) и \`\\:\` (буквальное двоеточие — НЕ работает как разделитель
  ключ/значение).`,
    zh: `- 转义表由 8 条扩展到 10 条:原有 8 条(\`\\\\\`、\`\\,\`、\`\\}\`、\`\\]\`、
  \`\\{\`、\`\\[\`、\`\\n\`、\`\\r\`)之上新增 \`\\.\`(键段中的字面点 —— 不分割
  路径)与 \`\\:\`(字面冒号 —— 不作为键/值分隔符)。`,
  },
  {
    id: 'v0-6-0-010',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The two new escapes are valid (redundant) in value
  contexts too.`,
    ru: `Оба новых escape допустимы (избыточны) и в
  значениях.`,
    zh: `两条新转义在值
  上下文中亦合法(冗余)。`,
  },
  {
    id: 'v0-6-0-011',
    join: 'tight',
    en: `- The escape-aware key scanner now treats the **first unescaped** \`:\`
  (or \`::\`) as the pair separator and splits dotted paths on
  **unescaped** \`.\` only.`,
    ru: `- Сканер ключей учитывает экранирование: первый **неэкранированный**
  \`:\` (или \`::\`) — разделитель пары; путь разбивается только по
  **неэкранированным** \`.\`.`,
    zh: `- 键扫描器支持转义:首个 **未转义** 的 \`:\`(或 \`::\`)为对分隔符;
  点分路径仅在 **未转义** 的 \`.\` 处分割。`,
  },
  {
    id: 'v0-6-0-012',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Inline-compound keys (\`{a\\.b: 1}\`) follow
  the same rules.`,
    ru: `То же действует для inline-compound
  (\`{a\\.b: 1}\`).`,
    zh: `内联 compound
  (\`{a\\.b: 1}\`)采用同样规则。`,
  },
  {
    id: 'v0-6-0-013',
    join: 'tight',
    en: `- Render path re-escapes \`\\\`, \`.\`, \`:\` in every emitted key segment,
  guaranteeing parse → render → parse identity for keys containing
  literal dots or colons.`,
    ru: `- Рендерер экранирует обратно \`\\\`, \`.\`, \`:\` в каждом сегменте ключа —
  гарантия parse → render → parse тождественности для ключей с
  буквальными точкой/двоеточием.`,
    zh: `- 渲染器在输出每个键段时回写转义 \`\\\`、\`.\`、\`:\`,保证含字面点/冒
  号的键的 parse → render → parse 同一性。`,
  },
  {
    id: 'v0-6-0-014',
    join: 'tight',
    en: `- Zero-copy event path keeps borrowing the source slice for a key
  segment that has no \`\\\`; segments with escapes are decoded into the
  bump arena so the event's \`&'a str\` lifetime is preserved.`,
    ru: `- В zero-copy event-пути сегмент без \`\\\` остаётся заимствованным из
  исходного буфера; сегмент с escape декодируется в bump-arena —
  время жизни \`&'a str\` сохраняется.`,
    zh: `- 零拷贝事件路径下,不含 \`\\\` 的键段继续从源缓冲区借用;含转义的
  键段解码到 bump-arena —— \`&'a str\` 生命周期保持不变。`,
  },
  {
    id: 'v0-6-0-015',
    join: 'block',
    en: `### Errors`,
    ru: `### Ошибки`,
    zh: `### 错误`,
  },
  {
    id: 'v0-6-0-016',
    join: 'block',
    en: `- \`\\X\` in a key with \`X\` not in the ten-escape set is now a
  \`BadEscapeSequence\`.`,
    ru: `- \`\\X\` в ключе, где \`X\` не входит в десятку, теперь
  \`BadEscapeSequence\`.`,
    zh: `- 键中 \`\\X\`(\`X\` 不在十条转义之列)现在抛出 \`BadEscapeSequence\`。`,
  },
  {
    id: 'v0-6-0-017',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `\`\\.\` and \`\\:\` are no longer \`BadEscapeSequence\`
  in any context.`,
    ru: `\`\\.\` и \`\\:\` больше не \`BadEscapeSequence\`
  ни в одном контексте.`,
    zh: `  \`\\.\` 与 \`\\:\` 在任何上下文均不再是 \`BadEscapeSequence\`。`,
  },
];
