// Changelog units for 0.5.0.
export const v0_5_0 = [
  {
    id: 'v0-5-0-001',
    join: 'block',
    en: `## [0.5.0] — 2026-05-28`,
    ru: `## [0.5.0] — 2026-05-28`,
    zh: `## [0.5.0] —— 2026-05-28`,
  },
  {
    id: 'v0-5-0-002',
    join: 'block',
    en: `Implements Ktav specification 0.5.0. This is a breaking release:
the parser, serializer, and Value model are rewritten for the new
language semantics.`,
    ru: `Реализует спецификацию Ktav 0.5.0. Это ломающий релиз: парсер,
сериализатор и модель Value переписаны под новую семантику языка.`,
    zh: `实现 Ktav 规范 0.5.0。这是一次破坏性发布：解析器、序列化器与 Value
模型均按新的语言语义重写。`,
  },
  {
    id: 'v0-5-0-003',
    join: 'block',
    en: `### Breaking`,
    ru: `### Breaking`,
    zh: `### 破坏性变更`,
  },
  {
    id: 'v0-5-0-004',
    join: 'block',
    en: `- Typed markers \`:i\` and \`:f\` removed.`,
    ru: `- Типизированные маркеры \`:i\` и \`:f\` удалены.`,
    zh: `- 移除类型标记 \`:i\` 与 \`:f\`。`,
  },
  {
    id: 'v0-5-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Numbers, booleans, and \`null\`
  are inferred from the lexical form (spec §§ 3.6, 5.2).`,
    ru: `Числа, булевы значения
  и \`null\` выводятся из лексической формы (спек §§ 3.6, 5.2).`,
    zh: `数字、布尔与 \`null\` 由词法形态推断
  （规范 §§ 3.6、5.2）。`,
  },
  {
    id: 'v0-5-0-006',
    join: 'tight',
    en: `- Comments use \`##\` (line-start only).`,
    ru: `- Комментарии пишутся через \`##\` (только с начала строки).`,
    zh: `- 注释使用 \`##\`（仅限行首）。`,
  },
  {
    id: 'v0-5-0-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `A single \`#\` byte is content.`,
    ru: `Одиночный
  байт \`#\` — это содержимое.`,
    zh: `单个 \`#\` 字节属于内容。`,
  },
  {
    id: 'v0-5-0-008',
    join: 'tight',
    en: `- Bare \`port: 8080\` is now \`Integer(8080)\`, not \`String("8080")\`.`,
    ru: `- Голое \`port: 8080\` теперь \`Integer(8080)\`, а не \`String("8080")\`.`,
    zh: `- 裸写的 \`port: 8080\` 现在是 \`Integer(8080)\`，不再是 \`String("8080")\`。`,
  },
  {
    id: 'v0-5-0-009',
    join: 'tight',
    en: `  Write \`port:: 8080\` to keep a String.`,
    ru: `  Чтобы сохранить String, пишите \`port:: 8080\`.`,
    zh: `  若要保留 String，请写 \`port:: 8080\`。`,
  },
  {
    id: 'v0-5-0-010',
    join: 'tight',
    en: `- Lone \`{\` / \`[\` on the first content line opens a multi-line root
  Object / Array (spec § 5.0.1 rules 4–5).`,
    ru: `- Одиночная \`{\` / \`[\` на первой содержательной строке открывает
  многострочный корневой Object / Array (спек § 5.0.1, правила 4–5).`,
    zh: `- 首个内容行上单独的 \`{\` / \`[\` 开启多行的根 Object / Array
  （规范 § 5.0.1 规则 4–5）。`,
  },
  {
    id: 'v0-5-0-011',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `The 0.1.1 JSONL-style
  semantic is removed.`,
    ru: `  Семантика 0.1.1 в стиле JSONL удалена.`,
    zh: `0.1.1 的 JSONL 式语义已移除。`,
  },
  {
    id: 'v0-5-0-012',
    join: 'tight',
    en: `- \`Float\` Values no longer carry the textual form; canonicalised via
  \`ryu\`.`,
    ru: `- Значения \`Float\` больше не несут текстовую форму; канонизируются
  через \`ryu\`.`,
    zh: `- \`Float\` 值不再携带文本形态；改由 \`ryu\` 规范化。`,
  },
  {
    id: 'v0-5-0-013',
    join: 'tight',
    en: `- Key segments are trimmed of leading/trailing whitespace (spec § 4).`,
    ru: `- Сегменты ключа обрезаются от ведущих и хвостовых пробелов (спек § 4).`,
    zh: `- 键段的首尾空白会被裁剪（规范 § 4）。`,
  },
  {
    id: 'v0-5-0-014',
    join: 'tight',
    en: `- Line terminators are \`LF\`, \`CR\`, or \`CR LF\`; \`CR\` is never a
  content byte.`,
    ru: `- Терминаторы строк — \`LF\`, \`CR\` или \`CR LF\`; \`CR\` никогда не является
  байтом содержимого.`,
    zh: `- 行终止符为 \`LF\`、\`CR\` 或 \`CR LF\`；\`CR\` 永远不作为内容字节。`,
  },
  {
    id: 'v0-5-0-015',
    join: 'tight',
    en: `- \`ErrorKind::InlineNonEmptyCompound\` and \`InvalidTypedScalar\` are
  deprecated (\`#[doc(hidden)]\`); the parser no longer emits them.`,
    ru: `- \`ErrorKind::InlineNonEmptyCompound\` и \`InvalidTypedScalar\` объявлены
  устаревшими (\`#[doc(hidden)]\`); парсер их больше не выдаёт.`,
    zh: `- \`ErrorKind::InlineNonEmptyCompound\` 与 \`InvalidTypedScalar\` 标记为
  废弃（\`#[doc(hidden)]\`）；解析器不再产生它们。`,
  },
  {
    id: 'v0-5-0-016',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-5-0-017',
    join: 'block',
    en: `- **Inline compounds** \`{k: v, …}\` / \`[i, …]\` (spec § 5.8) with
  trailing comma, mid-value brace literal (§ 5.8.5), and nesting
  depth limit of 128.`,
    ru: `- **Инлайн-составные** \`{k: v, …}\` / \`[i, …]\` (спек § 5.8) с хвостовой
  запятой, литеральной скобкой в середине значения (§ 5.8.5) и
  ограничением глубины вложенности в 128.`,
    zh: `- **内联复合** \`{k: v, …}\` / \`[i, …]\`（规范 § 5.8），支持尾随逗号、
  值中部的花括号字面量（§ 5.8.5），嵌套深度上限 128。`,
  },
  {
    id: 'v0-5-0-018',
    join: 'tight',
    en: `- **Eight escape sequences** in inline scalars: \`\\\\\`, \`\\,\`, \`\\}\`,
  \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\` (spec § 3.7).`,
    ru: `- **Восемь escape-последовательностей** в инлайн-скалярах: \`\\\\\`, \`\\,\`,
  \`\\}\`, \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\` (спек § 3.7).`,
    zh: `- 内联标量中的**八种转义序列**：\`\\\\\`、\`\\,\`、\`\\}\`、\`\\]\`、\`\\{\`、\`\\[\`、
  \`\\n\`、\`\\r\`（规范 § 3.7）。`,
  },
  {
    id: 'v0-5-0-019',
    join: 'tight',
    en: `- **Number literal grammar** — \`0x\`, \`0o\`, \`0b\`, decimal, underscore
  separators; i64 overflow falls back to String (spec § 3.6).`,
    ru: `- **Грамматика числовых литералов** — \`0x\`, \`0o\`, \`0b\`, десятичные,
  разделители-подчёркивания; переполнение i64 откатывается в String
  (спек § 3.6).`,
    zh: `- **数字字面量文法** —— \`0x\`、\`0o\`、\`0b\`、十进制、下划线分隔符；
  i64 溢出回退为 String（规范 § 3.6）。`,
  },
  {
    id: 'v0-5-0-020',
    join: 'tight',
    en: `- **\`emit_canonical()\`** — spec § 5.9 normative writer output,
  byte-deterministic across implementations.`,
    ru: `- **\`emit_canonical()\`** — нормативный вывод writer-а по спек § 5.9,
  байт-детерминированный между реализациями.`,
    zh: `- **\`emit_canonical()\`** —— 规范 § 5.9 规定的 writer 输出，跨实现
  字节确定。`,
  },
  {
    id: 'v0-5-0-021',
    join: 'tight',
    en: `- **\`src/parser/inline.rs\`** — inline-compound parser.`,
    ru: `- **\`src/parser/inline.rs\`** — парсер инлайн-составных.`,
    zh: `- **\`src/parser/inline.rs\`** —— 内联复合解析器。`,
  },
  {
    id: 'v0-5-0-022',
    join: 'tight',
    en: `- **\`src/render/canonical.rs\`** — canonical writer.`,
    ru: `- **\`src/render/canonical.rs\`** — канонический writer.`,
    zh: `- **\`src/render/canonical.rs\`** —— 规范化 writer。`,
  },
  {
    id: 'v0-5-0-023',
    join: 'tight',
    en: `- New error variants: \`UnterminatedInlineCompound\`,
  \`MalformedInlineCompound\`, \`BadEscapeSequence\`,
  \`OrphanLineAfterTopLevelInline\`.`,
    ru: `- Новые варианты ошибок: \`UnterminatedInlineCompound\`,
  \`MalformedInlineCompound\`, \`BadEscapeSequence\`,
  \`OrphanLineAfterTopLevelInline\`.`,
    zh: `- 新增错误变体：\`UnterminatedInlineCompound\`、
  \`MalformedInlineCompound\`、\`BadEscapeSequence\`、
  \`OrphanLineAfterTopLevelInline\`。`,
  },
  {
    id: 'v0-5-0-024',
    join: 'tight',
    en: `- Triple-test conformance harness (\`tests/spec_conformance.rs\`):
  93 valid + 31 invalid fixtures from \`spec/versions/0.5/tests/\`.`,
    ru: `- Тройной conformance-harness (\`tests/spec_conformance.rs\`):
  93 валидных + 31 невалидная фикстура из \`spec/versions/0.5/tests/\`.`,
    zh: `- 三重 conformance 测试台（\`tests/spec_conformance.rs\`）：取自
  \`spec/versions/0.5/tests/\` 的 93 个有效 + 31 个无效 fixture。`,
  },
  {
    id: 'v0-5-0-025',
    join: 'tight',
    en: `- Parser fast-paths: plain-decimal integers, no-underscore floats,
  ryu-reuse, first-byte fast-reject, LF-only line splitting,
  pre-sized Bump arena.`,
    ru: `- Быстрые пути парсера: простые десятичные целые, float без
  подчёркиваний, переиспользование ryu, быстрый отказ по первому
  байту, разбиение строк только по LF, преднастроенная Bump-арена.`,
    zh: `- 解析器快速路径：纯十进制整数、无下划线浮点、ryu 复用、首字节快速
  拒绝、仅按 LF 切行、预分配的 Bump arena。`,
  },
  {
    id: 'v0-5-0-026',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-5-0-027',
    join: 'block',
    en: `- License: \`MIT\` → \`MIT OR Apache-2.0\`.`,
    ru: `- Лицензия: \`MIT\` → \`MIT OR Apache-2.0\`.`,
    zh: `- 许可证：\`MIT\` → \`MIT OR Apache-2.0\`。`,
  },
  {
    id: 'v0-5-0-028',
    join: 'tight',
    en: `- Spec submodule pinned to \`v0.5.0\` (\`4d0a8aa\`).`,
    ru: `- Сабмодуль спеки закреплён на \`v0.5.0\` (\`4d0a8aa\`).`,
    zh: `- 规范 submodule 固定到 \`v0.5.0\`（\`4d0a8aa\`）。`,
  },
  {
    id: 'v0-5-0-029',
    join: 'tight',
    en: `- Doctests disabled (\`[lib] doctest = false\`); examples remain as
  \`text\` blocks in doc comments.`,
    ru: `- Doctests отключены (\`[lib] doctest = false\`); примеры остаются
  блоками \`text\` в doc-комментариях.`,
    zh: `- 关闭 doctest（\`[lib] doctest = false\`）；示例以 \`text\` 块形式保留在
  文档注释中。`,
  },
];
