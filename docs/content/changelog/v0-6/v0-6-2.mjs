// Changelog units for 0.6.2.
export const v0_6_2 = [
  {
    id: 'v0-6-2-001',
    join: 'block',
    en: `## [0.6.2] — 2026-08-19`,
    ru: `## [0.6.2] — 2026-08-19`,
    zh: `## [0.6.2] —— 2026-08-19`,
  },
  {
    id: 'v0-6-2-002',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-6-2-003',
    join: 'block',
    en: `- \`parse_strict()\` — opt-in strict parse mode that rejects **lossy
  scalars**: values whose lexical form differs from the canonical form
  of the number they would be inferred as (\`1.10\` → \`1.1\`, \`01234\` →
  \`1234\`, \`+7\`, \`0x1A\`, \`0o755\`, \`1_000\`, \`5e3\`).`,
    ru: `- \`parse_strict()\` — опциональный строгий режим разбора, отвергающий
  **скаляры с потерей**: значения, лексическая форма которых отличается
  от канонической формы числа, в которое они были бы выведены (\`1.10\` →
  \`1.1\`, \`01234\` → \`1234\`, \`+7\`, \`0x1A\`, \`0o755\`, \`1_000\`, \`5e3\`).`,
    zh: `- \`parse_strict()\` —— 可选的严格解析模式，拒绝**有损标量**：即词法形式
  与其将被推断成的数字的规范形式不一致的值（\`1.10\` → \`1.1\`、\`01234\` →
  \`1234\`、\`+7\`、\`0x1A\`、\`0o755\`、\`1_000\`、\`5e3\`）。`,
  },
  {
    id: 'v0-6-2-004',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `The default \`parse()\`
  silently canonicalises such values, so a round-trip rewrites the
  document with no diagnostic; strict mode surfaces them instead, and
  the message names both forms and suggests the two fixes (append \`::\`
  to keep a String, or write the canonical number).`,
    ru: `  Обычный \`parse()\` молча канонизирует такие значения, поэтому
  round-trip переписывает документ без всякой диагностики; строгий
  режим вместо этого сообщает о них, называя обе формы и подсказывая
  два способа исправления (дописать \`::\`, чтобы оставить строку, либо
  записать число в канонической форме).`,
    zh: `默认的 \`parse()\`
  会静默地将这类值规范化，因此往返一次就会改写文档且没有任何提示；
  严格模式则将其暴露出来，错误信息同时给出两种形式，并提示两种修复
  方式（追加 \`::\` 以保留字符串，或直接写规范数字）。`,
  },
  {
    id: 'v0-6-2-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Documents accepted
  by \`parse_strict()\` produce exactly the same \`Value\` tree as
  \`parse()\`.`,
    ru: `Документы, принятые
  \`parse_strict()\`, дают ровно то же дерево \`Value\`, что и \`parse()\`.`,
    zh: `被
  \`parse_strict()\` 接受的文档产生的 \`Value\` 树与 \`parse()\` 完全一致。`,
  },
  {
    id: 'v0-6-2-006',
    join: 'tight',
    en: `- \`ErrorKind::LossyScalar { line, body, canonical, span }\` — the new
  strict-mode-only error variant, wired into \`ErrorKind::line()\` /
  \`ErrorKind::span()\`.`,
    ru: `- \`ErrorKind::LossyScalar { line, body, canonical, span }\` — новый
  вариант ошибки, существующий только в строгом режиме; учтён в
  \`ErrorKind::line()\` / \`ErrorKind::span()\`.`,
    zh: `- \`ErrorKind::LossyScalar { line, body, canonical, span }\` —— 仅在严格
  模式下出现的新错误变体，已接入 \`ErrorKind::line()\` /
  \`ErrorKind::span()\`。`,
  },
  {
    id: 'v0-6-2-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`ErrorKind\` is \`#[non_exhaustive]\`, so this is
  an additive change.`,
    ru: `\`ErrorKind\` помечен
  \`#[non_exhaustive]\`, поэтому изменение аддитивное.`,
    zh: `\`ErrorKind\` 标注了 \`#[non_exhaustive]\`，因此这是
  一次增量式变更。`,
  },
  {
    id: 'v0-6-2-008',
    join: 'block',
    en: `Behaviour of \`parse()\`, the serde path (\`from_str\`) and the C ABI is
unchanged; the serde event path has no strict variant yet.`,
    ru: `Поведение \`parse()\`, serde-пути (\`from_str\`) и C ABI не изменилось;
у serde event-пути строгого варианта пока нет.`,
    zh: `\`parse()\`、serde 路径（\`from_str\`）与 C ABI 的行为均未改变；serde
事件路径目前尚无严格模式变体。`,
  },
  {
    id: 'v0-6-2-009',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-6-2-010',
    join: 'block',
    en: `- **MSRV raised from 1.70 to 1.71.** Not a choice of ours: \`serde_core\`
  requires \`serde_derive = "=1.0.229"\`, and that release (2026-07-18)
  declares \`rust-version = 1.71\`.`,
    ru: `- **MSRV поднят с 1.70 до 1.71.** Не наш выбор: \`serde_core\` требует
  \`serde_derive = "=1.0.229"\`, а тот релиз (2026-07-18) объявляет
  \`rust-version = 1.71\`.`,
    zh: `- **MSRV 从 1.70 提升至 1.71。** 这并非我们的主动选择：\`serde_core\`
  要求 \`serde_derive = "=1.0.229"\`，而该版本（2026-07-18）声明
  \`rust-version = 1.71\`。`,
  },
  {
    id: 'v0-6-2-011',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Since a library does not ship its
  \`Cargo.lock\`, a 1.70 user could no longer build this crate anyway —
  the manifest now states what is actually required.`,
    ru: `Библиотека не поставляет свой \`Cargo.lock\`,
  поэтому пользователь на 1.70 всё равно уже не смог бы собрать этот
  crate — манифест теперь говорит правду о реальных требованиях.`,
    zh: `库不会随包发布自己的 \`Cargo.lock\`，因此
  1.70 的用户本就已经无法构建本 crate —— 现在清单如实反映了真实要求。`,
  },
  {
    id: 'v0-6-2-012',
    join: 'block',
    en: `Thanks to [@chappihappymeal](https://github.com/chappihappymeal) for
reporting the issue and contributing the implementation
([#1](https://github.com/ktav-lang/rust/issues/1),
[#2](https://github.com/ktav-lang/rust/pull/2)).`,
    ru: `Спасибо [@chappihappymeal](https://github.com/chappihappymeal) за
найденную проблему и реализацию
([#1](https://github.com/ktav-lang/rust/issues/1),
[#2](https://github.com/ktav-lang/rust/pull/2)).`,
    zh: `感谢 [@chappihappymeal](https://github.com/chappihappymeal) 报告问题
并贡献实现（[#1](https://github.com/ktav-lang/rust/issues/1)、
[#2](https://github.com/ktav-lang/rust/pull/2)）。`,
  },
];
