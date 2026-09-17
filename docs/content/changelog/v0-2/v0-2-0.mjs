// Changelog units for 0.2.0.
export const v0_2_0 = [
  {
    id: 'v0-2-0-001',
    join: 'block',
    en: `## [0.2.0] — 2026-05-07`,
    ru: `## [0.2.0] — 2026-05-07`,
    zh: `## [0.2.0] —— 2026-05-07`,
  },
  {
    id: 'v0-2-0-002',
    join: 'block',
    en: `Minor release with two breaking output / validation changes:`,
    ru: `Минорный релиз с двумя breaking-изменениями вывода / валидации:`,
    zh: `次要发布,带两项 breaking 输出 / 校验改动:`,
  },
  {
    id: 'v0-2-0-003',
    join: 'block',
    en: `### Changed (breaking)`,
    ru: `### Изменено (breaking)`,
    zh: `### 变更(breaking)`,
  },
  {
    id: 'v0-2-0-004',
    join: 'block',
    en: `- **Multi-line strings emit indented stripped form \`( ... )\` by default**,
  not verbatim \`(( ... ))\`.`,
    ru: `- **Многострочные строки по умолчанию выводятся в форме stripped
  \`( ... )\`** с отступом, а не verbatim \`(( ... ))\`.`,
    zh: `- **多行字符串默认输出为缩进 stripped 形式 \`( ... )\`**,而非 verbatim
  \`(( ... ))\`。`,
  },
  {
    id: 'v0-2-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Verbatim is still produced as a fallback when
  the content has its own leading whitespace (which the parser-side
  dedent would clobber) or contains a sole-\`)\` line that would close the
  stripped form prematurely.`,
    ru: `Verbatim остаётся
  как fallback когда содержимое имеет собственный leading-whitespace
  (parser-side dedent его съест) или строку, тримящуюся в \`)\` (закрыла
  бы stripped преждевременно).`,
    zh: `当内容自带前导空白(会被解析侧的 dedent 吞掉)或包含
  仅为 \`)\` 的行(会提前关闭 stripped)时,fallback 到 verbatim。`,
  },
  {
    id: 'v0-2-0-006',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Code that compares \`to_string\` /
  \`render\` output byte-for-byte to a baked-in \`((...))\` literal needs to
  be updated.`,
    ru: `Код, сравнивающий вывод \`to_string\` /
  \`render\` побайтово с фиксированным \`((...))\`, нужно обновить.`,
    zh: `逐字节
  比较 \`to_string\` / \`render\` 输出与硬编码 \`((...))\` 的代码需要更新。`,
  },
  {
    id: 'v0-2-0-007',
    join: { en: 'flow', ru: 'tight', zh: 'tight' },
    en: `Round-tripping (\`parse(to_string(v)) == v\`) is unchanged.`,
    ru: `  Round-trip (\`parse(to_string(v)) == v\`) не изменился.`,
    zh: `  Round-trip(\`parse(to_string(v)) == v\`)未受影响。`,
  },
  {
    id: 'v0-2-0-008',
    join: 'block',
    en: `      // Before (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // After (0.2.0):
      // body: (
      //     line1
      //     line2
      // )`,
    ru: `      // Было (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // Стало (0.2.0):
      // body: (
      //     line1
      //     line2
      // )`,
    zh: `      // 之前 (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // 之后 (0.2.0):
      // body: (
      //     line1
      //     line2
      // )`,
  },
  {
    id: 'v0-2-0-009',
    join: 'block',
    en: `  Both \`Value\` → \`render::render(&value)\` and \`T: Serialize\` →
  \`ser::to_string(&t)\` paths are updated consistently.`,
    ru: `  Оба пути сериализации — \`Value\` → \`render::render(&value)\` и
  \`T: Serialize\` → \`ser::to_string(&t)\` — обновлены консистентно.`,
    zh: `  序列化双路径(\`Value\` → \`render::render(&value)\` 与
  \`T: Serialize\` → \`ser::to_string(&t)\`)同步更新,行为一致。`,
  },
  {
    id: 'v0-2-0-010',
    join: 'block',
    en: `- **Typed-float marker \`:f\` now accepts integer literals.** The mantissa's
  decimal point is **optional**: \`:f 42\` is valid (parsed as \`42.0\`),
  matching the JSON / TOML / YAML convention that integer literals
  coerce to floats.`,
    ru: `- **Типизированный маркер \`:f\` принимает integer-литералы.** Десятичная
  точка в мантиссе теперь **опциональна**: \`:f 42\` валидно (парсится
  как \`42.0\`) — конвенция JSON / TOML / YAML, где integer литералы
  приводятся к float.`,
    zh: `- **类型标记 \`:f\` 接受整数字面量。** 尾数中的小数点现在是**可选**:
  \`:f 42\` 合法(解析为 \`42.0\`),沿用 JSON / TOML / YAML 的惯例
  (整数字面量隐式提升为 float)。`,
  },
  {
    id: 'v0-2-0-011',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`:f 1.\` (no fractional digits) and \`:f .5\` (no
  integer part) remain invalid.`,
    ru: `\`:f 1.\` (без дробной части) и \`:f .5\` (без
  целой части) по-прежнему невалидны.`,
    zh: `\`:f 1.\`(无小数部分)与 \`:f .5\`
  (无整数部分)仍然非法。`,
  },
  {
    id: 'v0-2-0-012',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Code that depends on \`:f 42\` raising
  \`InvalidTypedScalar\` needs to be updated.`,
    ru: `Код, ожидающий
  \`InvalidTypedScalar\` для \`:f 42\`, нужно обновить.`,
    zh: `依赖 \`:f 42\` 报 \`InvalidTypedScalar\` 的
  代码需要更新。`,
  },
  {
    id: 'v0-2-0-013',
    join: 'block',
    common: `### Spec`,
  },
  {
    id: 'v0-2-0-014',
    join: 'block',
    en: `- \`spec/versions/0.1/tests\` fixture \`typed_float_without_decimal\` moved
  from \`invalid/\` to \`valid/typed_float_integer_body\` to reflect the new
  semantics.`,
    ru: `- В \`spec/versions/0.1/tests\` фикстура \`typed_float_without_decimal\`
  перенесена из \`invalid/\` в \`valid/typed_float_integer_body\` под
  новую семантику.`,
    zh: `- \`spec/versions/0.1/tests\` 中的 fixture \`typed_float_without_decimal\`
  从 \`invalid/\` 移到 \`valid/typed_float_integer_body\`,以反映新语义。`,
  },
  {
    id: 'v0-2-0-015',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `Spec submodule synced.`,
    ru: `Submodule spec синхронизирован.`,
    zh: `  spec submodule 已同步。`,
  },
];
