// README units: not-do section.
export const notDo = [
  {
    id: 'not-do-heading',
    join: 'block',
    en: `## What Ktav does NOT do — and never will`,
    ru: `## Чего Ktav НЕ делает — и никогда не будет`,
    zh: `## Ktav **不**做、也永远不会做的事`,
  },
  {
    id: 'preamble-171',
    join: 'block',
    en: `- **Inline non-empty compounds** like \`x: { a: 1, b: 2 }\`.`,
    ru: `- **Inline непустые compound-ы** вроде \`x: { a: 1, b: 2 }\`.`,
    zh: `- **内联的非空复合值**,比如 \`x: { a: 1, b: 2 }\`。`,
  },
  {
    id: 'preamble-172',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `They'd bring
  commas, and commas would bring escaping.`,
    ru: `Они
  притащили бы запятые, а запятые притащили бы escape.`,
    zh: `它们会带来逗号,
  逗号又会带来转义。`,
  },
  {
    id: 'preamble-173',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Compound values are
  multiline.`,
    ru: `Compound-
  значения многострочны.`,
    zh: `复合值保持多行。`,
  },
  {
    id: 'preamble-174',
    join: 'tight',
    en: `- **Anchors / aliases / merge keys** (\`&anchor\`, \`*ref\`, \`<<:\`).`,
    ru: `- **Якоря / алиасы / merge-ключи** (\`&anchor\`, \`*ref\`, \`<<:\`).`,
    zh: `- **锚点 / 别名 / 合并键**(\`&anchor\`、\`*ref\`、\`<<:\`)。`,
  },
  {
    id: 'preamble-175',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Any
  line whose meaning depends on a declaration far away stops being
  self-sufficient.`,
    ru: `Любая
  строка, смысл которой зависит от декларации в отдалённом месте,
  перестаёт быть самодостаточной.`,
    zh: `任何一行
  若其含义依赖远处的声明,就不再自洽。`,
  },
  {
    id: 'preamble-176',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `If you want DRY, compose defaults in code.`,
    ru: `Если нужен DRY — композируйте
  defaults в коде.`,
    zh: `若需要 DRY,请在代码里
  组合默认值。`,
  },
  {
    id: 'preamble-177',
    join: 'tight',
    en: `- **File includes** (\`@include\`, \`!import\`).`,
    ru: `- **Инклюды файлов** (\`@include\`, \`!import\`).`,
    zh: `- **文件包含**(\`@include\`、\`!import\`)。`,
  },
  {
    id: 'preamble-178',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Write a wrapper in code
  for large configs.`,
    ru: `Для больших конфигов
  напишите обёртку в коде.`,
    zh: `大型配置请在代码里包一层
  封装。`,
  },
  {
    id: 'preamble-179',
    join: 'tight',
    en: `- **Top-level arrays.** The document is always an object.`,
    ru: `- **Массивы на верхнем уровне.** Документ всегда — объект.`,
    zh: `- **顶层数组。** 文档始终是对象。`,
  },
];
