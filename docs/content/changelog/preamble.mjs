// Changelog units for preamble.
export const preamble = [
  {
    id: 'preamble-001',
    en: `# Changelog — \`ktav\` crate`,
    ru: `# Журнал изменений — crate \`ktav\``,
    zh: `# 变更日志 —— \`ktav\` crate`,
  },
  {
    id: 'preamble-002',
    join: 'block',
    en: `**Languages:** **English** · [Русский](CHANGELOG.ru.md) · [简体中文](CHANGELOG.zh.md)`,
    ru: `**Languages:** [English](CHANGELOG.md) · **Русский** · [简体中文](CHANGELOG.zh.md)`,
    zh: `**Languages:** [English](CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**`,
  },
  {
    id: 'preamble-003',
    join: 'block',
    en: `All notable changes to the \`ktav\` crate are documented here.`,
    ru: `Все значимые изменения в crate \`ktav\` документируются здесь.`,
    zh: `本文件记录 \`ktav\` crate 的全部重要变更。`,
  },
  {
    id: 'preamble-004',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The
format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate adheres to [Semantic Versioning](https://semver.org/) with
the Cargo convention that a minor bump is breaking while pre-1.0.`,
    ru: `Формат
основан на [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
crate следует [Semantic Versioning](https://semver.org/) с
Cargo-конвенцией: до 1.0 bump MINOR считается ломающим.`,
    zh: `格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);crate
遵循 [Semantic Versioning](https://semver.org/),并采用 Cargo 惯例:
在 1.0 之前,MINOR 递进视为破坏性变更。`,
  },
  {
    id: 'preamble-005',
    join: 'block',
    en: `For the format specification's own history, see the
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec) repository.`,
    ru: `Историю самой спецификации формата см. в репозитории
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec).`,
    zh: `格式规范自身的历史,请见
[\`ktav-lang/spec\`](https://github.com/ktav-lang/spec) 仓库。`,
  },
];
