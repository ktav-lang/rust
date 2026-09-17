// Changelog units for 0.1.3.
export const v0_1_3 = [
  {
    id: 'v0-1-3-001',
    join: 'block',
    en: `## [0.1.3] — 2026-04-26`,
    ru: `## [0.1.3] — 2026-04-26`,
    zh: `## [0.1.3] —— 2026-04-26`,
  },
  {
    id: 'v0-1-3-002',
    join: 'block',
    en: `Same content as the yanked 0.1.2 — re-released through the new
automated \`Release\` workflow (CI verify → \`cargo publish\`) so future
releases never depend on a manual \`cargo publish\` from a maintainer's
machine.`,
    ru: `То же содержимое, что и yank-нутая 0.1.2 — перевыпуск через новый
автоматизированный workflow \`Release\` (CI verify → \`cargo publish\`),
чтобы будущие релизы не зависели от ручного \`cargo publish\` с машины
сопровождающего.`,
    zh: `与已 yank 的 0.1.2 内容完全相同 —— 通过新的自动化 \`Release\` 工作流
(CI verify → \`cargo publish\`)重新发布,从而后续发布不再依赖维护
者本地机器上的手动 \`cargo publish\`。`,
  },
  {
    id: 'v0-1-3-003',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `0.1.2 was yanked solely to validate the pipeline end-to-end
on a fresh version (crates.io is immutable; we can't re-publish 0.1.2
itself).`,
    ru: `0.1.2 был отозван только ради end-to-end проверки
пайплайна на свежей версии (crates.io immutable, перевыпустить саму
0.1.2 нельзя).`,
    zh: `0.1.2 被 yank 仅用于在一个全新
版本上端到端验证流水线(crates.io 不可变,无法重新发布 0.1.2 本身)。`,
  },
];
