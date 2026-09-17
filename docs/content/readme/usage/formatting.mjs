// README units: formatting section.
export const formatting = [
  {
    id: 'formatting-heading',
    join: 'block',
    en: `## Formatting — canonical spelling, comments kept`,
    ru: `## Форматирование — каноническое написание, комментарии на месте`,
    zh: `## 格式化 —— 规范写法,注释保留`,
  },
  {
    id: 'formatting-intro',
    join: 'block',
    en: `\`ktav::format_str\` rewrites a document into the structural spelling
\`emit_canonical\` produces, but keeps the trivia the canonical writer
drops. Every comment survives verbatim.`,
    ru: `\`ktav::format_str\` переписывает документ в то структурное написание,
которое выдаёт \`emit_canonical\`, но сохраняет оформление, выбрасываемое
каноническим писателем. Каждый комментарий сохраняется дословно.`,
    zh: `\`ktav::format_str\` 把文档改写为 \`emit_canonical\` 所产出的结构写法,
但保留规范写入器会丢弃的附属内容。每条注释都逐字保留。`,
  },
  {
    id: 'formatting-snippet',
    join: 'block',
    common: `\`\`\`rust
let tidied = ktav::format_str("## the server\\nserver: {host: a, port: 80}\\n")?;
assert_eq!(tidied, "## the server\\nserver: {\\n    host: a\\n    port: 80\\n}\\n");
\`\`\``,
  },
  {
    id: 'formatting-snippet-rendered',
    join: 'block',
    en: `That is, the inline compound expands to the canonical multi-line
form and the comment stays exactly where it was:`,
    ru: `То есть inline-компаунд разворачивается в каноническую многострочную
форму, а комментарий остаётся ровно там, где был:`,
    zh: `也就是说,行内复合值展开为规范的多行形式,而注释仍停留在原处:`,
  },
  {
    id: 'formatting-snippet-ktav',
    join: 'block',
    common: `\`\`\`ktav
## the server
server: {
    host: a
    port: 80
}
\`\`\``,
  },
  {
    id: 'formatting-blank-lines',
    join: 'block',
    en: `Blank lines survive as a grouping hint, but a run of two or more
collapses to exactly one, and blank padding just inside a bracket is
dropped. That is what makes the transform a fixed point: formatting
already-formatted output never changes it again.`,
    ru: `Пустые строки сохраняются как подсказка группировки, но серия из двух
и более схлопывается ровно в одну, а пустой отступ сразу внутри скобки
выбрасывается. Именно это делает преобразование неподвижной точкой:
форматирование уже отформатированного вывода больше ничего не меняет.`,
    zh: `空行作为分组提示保留,但连续两行及以上会折叠为恰好一行,紧贴括号内侧
的空行填充会被丢弃。正是这一点让该变换成为不动点:对已格式化的输出再
次格式化不会再有任何改变。`,
  },
  {
    id: 'formatting-key-order',
    join: 'block',
    en: `Key order is never changed. Canonical form has no sorting rule
(§ 5.9), and reordering keys would make review diffs worse, not
better — this is a spelling normaliser, not a refactoring tool.`,
    ru: `Порядок ключей не меняется никогда. У канонической формы нет правила
сортировки (§ 5.9), а перестановка ключей ухудшила бы диффы на ревью,
а не улучшила: это нормализатор написания, а не инструмент
рефакторинга.`,
    zh: `键序永不改变。规范形式没有排序规则(§ 5.9),而重排键只会让评审差异
更糟,而非更好 —— 这是写法规范化工具,不是重构工具。`,
  },
  {
    id: 'formatting-canonical-relation',
    join: 'block',
    en: `For a document with no comments **and no blank lines**,
\`format_str\` equals \`emit_canonical\` of its parse. The stronger
condition is deliberate: blank lines are no more part of the \`Value\`
model than comments are, so \`emit_canonical\` drops them and
\`format_str\` does not.`,
    ru: `Для документа без комментариев **и без пустых строк** \`format_str\`
совпадает с \`emit_canonical\` его разбора. Условие намеренно сильнее
очевидного: пустые строки входят в модель \`Value\` ровно настолько же,
насколько комментарии, — то есть никак, поэтому \`emit_canonical\` их
выбрасывает, а \`format_str\` нет.`,
    zh: `对于既无注释**也无空行**的文档,\`format_str\` 等于其解析结果的
\`emit_canonical\`。这个更强的条件是刻意的:空行与注释一样都不属于
\`Value\` 模型,因此 \`emit_canonical\` 会丢弃它们,而 \`format_str\`
不会。`,
  },
  {
    id: 'formatting-cli-heading',
    join: 'block',
    en: `### \`ktav-fmt\` — the optional command-line formatter`,
    ru: `### \`ktav-fmt\` — необязательный форматтер командной строки`,
    zh: `### \`ktav-fmt\` —— 可选的命令行格式化器`,
  },
  {
    id: 'formatting-cli-optional',
    join: 'block',
    en: `Behind the \`cli\` feature, off by default. In a project that already
has a toolchain the library call above plus a build hook is usually the
better answer, and editors format through \`ktav-lsp\`; the binary is
for the case where neither is at hand.`,
    ru: `Живёт за feature-флагом \`cli\` и по умолчанию выключен. В проекте, где
тулчейн уже есть, библиотечный вызов выше плюс хук сборки обычно
уместнее, а редакторы форматируют через \`ktav-lsp\`; бинарь — для
случая, когда ни того, ни другого под рукой нет.`,
    zh: `位于 \`cli\` feature 之后,默认关闭。在已有工具链的项目里,上面的库
调用加一个构建钩子通常更合适,编辑器则经由 \`ktav-lsp\` 格式化;二进制
是留给两者都不可用的场景。`,
  },
  {
    id: 'formatting-cli-usage',
    join: 'block',
    common: `\`\`\`text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
\`\`\``,
  },
  {
    id: 'formatting-cli-check',
    join: 'block',
    en: `\`--check\` writes nothing and prints the path of every file that is
not already formatted, so it drops straight into CI next to
\`cargo fmt --check\`.`,
    ru: `\`--check\` ничего не пишет и печатает путь каждого файла, который ещё
не отформатирован, — так что он встаёт в CI прямо рядом с
\`cargo fmt --check\`.`,
    zh: `\`--check\` 不写入任何内容,只打印每个尚未格式化的文件路径,因此可以
直接放进 CI,紧挨着 \`cargo fmt --check\`。`,
  },
];
