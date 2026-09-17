// README units: installation section.
export const installation = [
  {
    id: 'installation-heading',
    join: 'block',
    en: `## Installation`,
    ru: `## Установка`,
    zh: `## 安装`,
  },
  {
    id: 'installation-toml',
    join: 'block',
    common: `\`\`\`toml
[dependencies]
ktav = "0.7.1"
serde = { version = "1", features = ["derive"] }
\`\`\``,
  },
  {
    id: 'installation-cli',
    join: 'block',
    en: `The formatter is also available as a binary, behind an off-by-default
feature:`,
    ru: `Форматтер доступен и как исполняемый файл — за выключенным по
умолчанию feature-флагом:`,
    zh: `格式化器也可作为二进制取用,位于一个默认关闭的 feature 之后:`,
  },
  {
    id: 'installation-cli-cmd',
    join: 'block',
    common: `\`\`\`sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
\`\`\``,
  },
];
