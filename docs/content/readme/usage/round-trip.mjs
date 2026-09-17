// README units: round-trip section.
export const roundTrip = [
  {
    id: 'round-trip-heading',
    join: 'block',
    common: `## Round-trip`,
  },
  {
    id: 'round-trip-snippet',
    join: 'block',
    common: `\`\`\`rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
\`\`\``,
  },
  {
    id: 'preamble-163',
    join: 'block',
    en: `Serialization preserves:`,
    ru: `Сериализация сохраняет:`,
    zh: `序列化会保持:`,
  },
  {
    id: 'preamble-164',
    join: 'tight',
    en: `- **Field order** — \`Value::Object\` is backed by an \`IndexMap\`, so the
  order is whatever serde emits (for structs: declaration order).`,
    ru: `- **Порядок полей** — \`Value::Object\` лежит на \`IndexMap\`, так что
  порядок — тот, что эмитит serde (для struct-ов: порядок объявления).`,
    zh: `- **字段顺序** —— \`Value::Object\` 底层是 \`IndexMap\`,顺序由 serde
  输出决定(对结构体而言:就是声明顺序)。`,
  },
  {
    id: 'preamble-165',
    join: 'tight',
    en: `- **Literal strings** — values starting with \`{\` or \`[\` are emitted
  with the \`::\` marker.`,
    ru: `- **Литеральные строки** — значения, начинающиеся с \`{\` или \`[\`,
  эмитятся с маркером \`::\`.`,
    zh: `- **字面量字符串** —— 以 \`{\` 或 \`[\` 开头的值会带 \`::\` 标记输出。`,
  },
  {
    id: 'preamble-166',
    join: 'tight',
    en: `- **\`None\` fields** — skipped on output; reappear as \`None\` on input
  (via serde's \`Option\` handling).`,
    ru: `- **Поля \`None\`** — пропускаются на выходе; восстанавливаются как
  \`None\` на входе (через обработку \`Option\` в serde).`,
    zh: `- **\`None\` 字段** —— 输出时跳过;输入时通过 serde 的 \`Option\`
  处理重新出现为 \`None\`。`,
  },
];
