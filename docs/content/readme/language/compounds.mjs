// README units: compounds section.
export const compounds = [
  {
    id: 'compounds-multiline-heading',
    join: 'block',
    en: `## Compound values are multi-line`,
    ru: `## Составные значения — многострочные`,
    zh: `## 复合值是多行的`,
  },
  {
    id: 'compounds-multiline-rule-1',
    join: 'block',
    en: `Non-empty \`{ ... }\` / \`[ ... ]\` **must** span multiple lines, with the
closing bracket on its own line.`,
    ru: `Непустые \`{ ... }\` / \`[ ... ]\` **обязаны** занимать несколько строк,
с закрывающей скобкой на отдельной строке.`,
    zh: `非空的 \`{ ... }\` / \`[ ... ]\` **必须**跨越多行,闭合括号独占一行。`,
  },
  {
    id: 'compounds-multiline-rule-2',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `\`x: { a: 1 }\` and \`x: [1, 2, 3]\` are
rejected with a clear error — Ktav has no comma-separation rules and
no escape mechanism for them.`,
    ru: `\`x: { a: 1 }\` и
\`x: [1, 2, 3]\` отклоняются с ясной ошибкой — в Ktav нет правил
разделения запятыми и нет механизма escape для них.`,
    zh: `\`x: { a: 1 }\` 与 \`x: [1, 2, 3]\` 会被以清晰的错误拒绝——Ktav 没有
逗号分隔的规则,也没有针对它们的转义机制。`,
  },
  {
    id: 'preamble-055',
    join: 'block',
    common: `\`\`\`text
## rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]

## accepted — multi-line form
server: {
    host: 127.0.0.1
    port: 8080
}

tags: [
    primary
    eu
    prod
]
\`\`\``,
  },
];
