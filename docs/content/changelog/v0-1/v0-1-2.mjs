// Changelog units for 0.1.2.
export const v0_1_2 = [
  {
    id: 'v0-1-2-001',
    join: 'block',
    en: `## [0.1.2] — 2026-04-26`,
    ru: `## [0.1.2] — 2026-04-26`,
    zh: `## [0.1.2] —— 2026-04-26`,
  },
  {
    id: 'v0-1-2-002',
    join: 'block',
    en: `Re-publish of 0.1.1's contents with the source tree run through
\`cargo fmt\`.`,
    ru: `Перевыпуск содержимого 0.1.1 после прогона \`cargo fmt\`.`,
    zh: `0.1.1 内容的重新发布,源码经过 \`cargo fmt\` 处理。`,
  },
  {
    id: 'v0-1-2-003',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `0.1.1 was yanked because the new files (\`benches/vs_json.rs\`,
\`src/thin/event*.rs\`, \`src/thin/fast_num.rs\`) hadn't been formatted
through rustfmt before publish, which tripped the CI lint check on the
tag push.`,
    ru: `0.1.1 был
отозван (yanked), потому что новые файлы (\`benches/vs_json.rs\`,
\`src/thin/event*.rs\`, \`src/thin/fast_num.rs\`) не были отформатированы
через rustfmt перед публикацией, что обвалило CI lint при пуше тега.`,
    zh: `0.1.1 被 yank,因为
新增文件(\`benches/vs_json.rs\`, \`src/thin/event*.rs\`,
\`src/thin/fast_num.rs\`)在发布前未经 rustfmt 处理,导致 CI lint 在
tag push 时失败。`,
  },
  {
    id: 'v0-1-2-004',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `**Functionally identical to 0.1.1** — only whitespace differs.`,
    ru: `**Функционально идентично 0.1.1** — отличается только пробелами.`,
    zh: `**功能与 0.1.1 完全一致** —— 仅空白字符不同。`,
  },
];
