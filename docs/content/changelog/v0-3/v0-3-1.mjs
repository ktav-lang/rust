// Changelog units for 0.3.1.
export const v0_3_1 = [
  {
    id: 'v0-3-1-001',
    join: 'block',
    en: `## [0.3.1] — 2026-05-10`,
    ru: `## [0.3.1] — 2026-05-10`,
    zh: `## [0.3.1] —— 2026-05-10`,
  },
  {
    id: 'v0-3-1-002',
    join: 'block',
    en: `Backward-compatible feature release tracking spec 0.1.1.`,
    ru: `Обратно совместимый функциональный релиз, следующий за спекой 0.1.1.`,
    zh: `向后兼容的功能发布，跟进规范 0.1.1。`,
  },
  {
    id: 'v0-3-1-003',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-3-1-004',
    join: 'block',
    en: `- **Top-level Array support** (spec § 5.0.1, added in spec 0.1.1) —
  a document whose first content line has an array-item shape
  (bare scalar, \`:: text\`, \`:i 42\`, \`:f 3.14\`, lone \`{\` / \`[\`, or a
  multi-line opener \`(\` / \`((\`) is now parsed as a root-level
  \`Value::Array\`.`,
    ru: `- **Поддержка Array на верхнем уровне** (спек § 5.0.1, добавлено в
  спеке 0.1.1) — документ, первая содержательная строка которого имеет
  форму элемента массива (голый скаляр, \`:: текст\`, \`:i 42\`, \`:f 3.14\`,
  одиночная \`{\` / \`[\` или многострочный опенер \`(\` / \`((\`), теперь
  разбирается как корневой \`Value::Array\`.`,
    zh: `- **顶层 Array 支持**（规范 § 5.0.1，于规范 0.1.1 加入）——首个内容行
  具备数组元素形态（裸标量、\`:: 文本\`、\`:i 42\`、\`:f 3.14\`、单独的
  \`{\` / \`[\`，或多行开启符 \`(\` / \`((\`）的文档，现在解析为根级
  \`Value::Array\`。`,
  },
  {
    id: 'v0-3-1-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Previously the root was always \`Value::Object\`,
  so a bare scalar at line 1 errored as \`MissingSeparator\`.`,
    ru: `Раньше корнем всегда был
  \`Value::Object\`, поэтому голый скаляр в строке 1 давал ошибку
  \`MissingSeparator\`.`,
    zh: `此前根始终是 \`Value::Object\`，因此第 1 行的裸标量
  会以 \`MissingSeparator\` 报错。`,
  },
  {
    id: 'v0-3-1-006',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  Empty / comments-only documents still default to an empty Object
  (preserves 0.3.0 behaviour).`,
    ru: `Пустые документы и документы из одних
  комментариев по-прежнему дают пустой Object (поведение 0.3.0
  сохранено).`,
    zh: `空文档与仅含注释的文档仍默认为空
  Object（保持 0.3.0 行为）。`,
  },
  {
    id: 'v0-3-1-007',
    join: 'tight',
    en: `- **\`ktav::to_string_force_strings(value)\`** — render any \`Value\`
  with every scalar coerced to a String.`,
    ru: `- **\`ktav::to_string_force_strings(value)\`** — рендер любого \`Value\` с
  приведением каждого скаляра к String.`,
    zh: `- **\`ktav::to_string_force_strings(value)\`** —— 渲染任意 \`Value\`，并将
  每个标量强制为 String。`,
  },
  {
    id: 'v0-3-1-008',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Typed integers (\`:i\`),
  typed floats (\`:f\`), booleans, and null are flattened to their
  textual form; compounds preserve their structure.`,
    ru: `Типизированные целые (\`:i\`),
  типизированные float (\`:f\`), булевы значения и null уплощаются в
  текстовую форму; составные сохраняют структуру.`,
    zh: `类型化整数（\`:i\`）、类型化浮点（\`:f\`）、
  布尔与 null 会被展平为其文本形态；复合结构保持不变。`,
  },
  {
    id: 'v0-3-1-009',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The output
  round-trips back through the parser as the same set of String
  scalars.`,
    ru: `Вывод проходит
  round-trip обратно через парсер как тот же набор String-скаляров.`,
    zh: `输出经解析器
  round-trip 后仍是同一组 String 标量。`,
  },
  {
    id: 'v0-3-1-010',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `Useful for "everything is a string" dumps for downstream
  consumers that don't understand typed markers, or for diff-
  friendly canonical text.`,
    ru: `  Полезно для дампов «всё — строка» для потребителей, не понимающих
  типизированных маркеров, или для канонического текста, удобного для
  diff-а.`,
    zh: `适用于面向不理解类型标记的
  下游消费者的「一切皆字符串」转储，或便于 diff 的规范化文本。`,
  },
  {
    id: 'v0-3-1-011',
    join: 'tight',
    en: `- New \`Render\` exit point: \`render\` accepts both Object and Array
  top-level values (top-level Arrays render as bare item-per-line,
  no \`[...]\` brackets).`,
    ru: `- Новая точка выхода \`Render\`: \`render\` принимает на верхнем уровне и
  Object, и Array (Array верхнего уровня рендерится голыми элементами
  по строке, без скобок \`[...]\`).`,
    zh: `- 新的 \`Render\` 出口：\`render\` 在顶层同时接受 Object 与 Array
  （顶层 Array 渲染为每行一个裸元素，不带 \`[...]\` 方括号）。`,
  },
  {
    id: 'v0-3-1-012',
    join: 'block',
    en: `### Compatibility`,
    ru: `### Совместимость`,
    zh: `### 兼容性`,
  },
  {
    id: 'v0-3-1-013',
    join: 'block',
    en: `Strictly additive.`,
    ru: `Строго аддитивно.`,
    zh: `严格增量。`,
  },
  {
    id: 'v0-3-1-014',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Every document valid under 0.3.0 stays valid
under 0.3.1 and produces the same \`Value\`.`,
    ru: `Любой документ, валидный по 0.3.0, остаётся валидным
по 0.3.1 и даёт тот же \`Value\`.`,
    zh: `凡在 0.3.0 下有效的文档，在 0.3.1 下依然有效并产生相同的
\`Value\`。`,
  },
  {
    id: 'v0-3-1-015',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Only inputs 0.3.0
rejected as \`MissingSeparator\` (bare-scalar first lines) are now
accepted as Arrays.`,
    ru: `Только те входы, которые 0.3.0
отвергал как \`MissingSeparator\` (первая строка — голый скаляр), теперь
принимаются как Array.`,
    zh: `只有此前被 0.3.0 以 \`MissingSeparator\` 拒绝的输入（首行为
裸标量）现在被接受为 Array。`,
  },
  {
    id: 'v0-3-1-016',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The error variants and their spans are
unchanged for the Object path.`,
    ru: `Варианты ошибок и их spans для пути Object
не изменились.`,
    zh: `Object 路径上的错误变体及其 span 未变。`,
  },
  {
    id: 'v0-3-1-017',
    join: 'block',
    en: `The parser, render, thin event-parser, and thin event-deserializer
all honour spec § 5.0.1 consistently — \`parse\`, \`parse_events\`, and
\`from_str\` agree on the root kind for any given input.`,
    ru: `Парсер, render, thin event-парсер и thin event-десериализатор
согласованно соблюдают спек § 5.0.1 — \`parse\`, \`parse_events\` и
\`from_str\` дают одинаковый вид корня для любого входа.`,
    zh: `解析器、render、thin 事件解析器与 thin 事件反序列化器一致遵循规范
§ 5.0.1 —— 对任意给定输入，\`parse\`、\`parse_events\` 与 \`from_str\` 对根
的种类判断一致。`,
  },
];
