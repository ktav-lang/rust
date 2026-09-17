// Changelog units for 0.7.1.
export const v0_7_1 = [
  {
    id: 'v0-7-1-001',
    join: 'block',
    en: `## [0.7.1] — 2026-09-16`,
    ru: `## [0.7.1] — 2026-09-16`,
    zh: `## [0.7.1] —— 2026-09-16`,
  },
  {
    id: 'v0-7-1-002',
    join: 'block',
    en: `Implements [Ktav 0.7.1](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.md),
released 2026-09-16.`,
    ru: `Реализует [Ktav 0.7.1](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.ru.md),
выпущенный 2026-09-16.`,
    zh: `实现于 2026-09-16 发布的
[Ktav 0.7.1](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.zh.md)。`,
  },
  {
    id: 'v0-7-1-003',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `That specification change is
editorial — § 8.5 and a machine-readable corpus manifest — and asks
nothing new of a parser or a writer: every 0.7.0 document parses to the
same Value and every canonical rendering is unchanged byte for
byte.`,
    ru: `Это изменение спецификации редакционное — § 8.5 и машиночитаемый
манифест корпуса — и от парсера или писателя не требует ничего нового:
любой документ 0.7.0 разбирается в тот же Value, и любая каноническая
отрисовка не меняется ни на байт.`,
    zh: `该规范变更是编辑性的 —— § 8.5 与一份机器可读的语料库清单 —— 对解析器
或写入器并无新要求:每一份 0.7.0 文档解析所得的 Value 不变,每一次
规范化输出也逐字节不变。`,
  },
  {
    id: 'v0-7-1-004',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `What this release adds is the tooling that verification story leans
on, plus one conformance fix.`,
    ru: `Этот выпуск добавляет инструментарий, на который
опирается такая проверка, и одно исправление соответствия.`,
    zh: `本次发布新增的是那套验证所依赖的
工具,以及一处一致性修复。`,
  },
  {
    id: 'v0-7-1-004b',
    join: 'block',
    en: `\`spec-version\` metadata moves to \`0.7.1\` and the pinned \`spec\`
submodule moves to the 0.7.1 release commit, so the conformance corpus
this crate is tested against is the released one.`,
    ru: `Метаданные \`spec-version\` переходят на \`0.7.1\`, а закреплённый
submodule \`spec\` — на релизный коммит 0.7.1, так что conformance-корпус,
против которого проверяется crate, — именно выпущенный.`,
    zh: `\`spec-version\` 元数据升至 \`0.7.1\`,固定的 \`spec\` submodule 也移至
0.7.1 发布提交,因此本 crate 所对照的 conformance 语料库正是已发布的
那一份。`,
  },
  {
    id: 'v0-7-1-005',
    join: 'block',
    en: `### Added`,
    ru: `### Added`,
    zh: `### 新增`,
  },
  {
    id: 'v0-7-1-006',
    join: 'block',
    en: `- **\`ErrorEnvelope\` — one JSON object for every structured error**,
  parse-time and writer-time alike. Nine fields, always all nine, in a
  fixed order: \`error\`, \`reason\`, \`line\`, \`line_text\`, \`span\`,
  \`path\`, \`body\`, \`canonical\`, \`spec_section\`.`,
    ru: `- **\`ErrorEnvelope\` — один JSON-объект для любой структурированной
  ошибки**, как на разборе, так и на записи. Девять полей, всегда все
  девять, в фиксированном порядке: \`error\`, \`reason\`, \`line\`,
  \`line_text\`, \`span\`, \`path\`, \`body\`, \`canonical\`,
  \`spec_section\`.`,
    zh: `- **\`ErrorEnvelope\` —— 任何结构化错误都对应一个 JSON 对象**,解析期与
  写入期一视同仁。九个字段,永远是全部九个,顺序固定:\`error\`、
  \`reason\`、\`line\`、\`line_text\`、\`span\`、\`path\`、\`body\`、
  \`canonical\`、\`spec_section\`。`,
  },
  {
    id: 'v0-7-1-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Absent information is
  an explicit \`null\`, never an omitted key, so a consumer in any
  language can read every field positionally without negotiating a
  schema first.`,
    ru: `Отсутствующие сведения — явный \`null\`, а не
  пропущенный ключ, поэтому потребитель на любом языке читает каждое
  поле позиционно, без предварительного согласования схемы.`,
    zh: `缺失的信息是显式的 \`null\`,而不是省略
  键,因此任何语言的使用方都能按位置读取每个字段,无需事先协商模式。`,
  },
  {
    id: 'v0-7-1-008',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`path\` is an
  array of exact decoded key segments, never a joined string: a key
  literally named \`a.b\` is one segment and cannot be confused with a
  two-segment path.`,
    ru: `\`path\` — массив
  точных декодированных сегментов ключа, а не склеенная строка: ключ,
  буквально названный \`a.b\`, — это один сегмент, и его нельзя спутать
  с путём из двух.`,
    zh: `\`path\` 是精确解码后的键段数组,而非拼接
  字符串:字面名为 \`a.b\` 的键是一个段,不会与两段路径混淆。`,
  },
  {
    id: 'v0-7-1-009',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Build it with
  \`ErrorEnvelope::from_error(&err, source)\`, render it with
  \`to_json()\` or \`push_json()\`. No serde dependency is involved and
  every string is escaped per RFC 8259.`,
    ru: `Строится через
  \`ErrorEnvelope::from_error(&err, source)\`, печатается через
  \`to_json()\` или \`push_json()\`. Зависимость от serde не
  задействована, каждая строка экранируется по RFC 8259.`,
    zh: `用
  \`ErrorEnvelope::from_error(&err, source)\` 构建,用 \`to_json()\` 或
  \`push_json()\` 渲染。不涉及 serde 依赖,每个字符串都按 RFC 8259
  转义。`,
  },
  {
    id: 'v0-7-1-010',
    join: 'block',
    en: `- **\`format_str\` — a comment-preserving formatter.** It normalises a
  document's structural spelling to what \`emit_canonical\` produces,
  interleaved with the trivia the canonical writer drops.`,
    ru: `- **\`format_str\` — форматтер, сохраняющий комментарии.** Приводит
  структурное написание документа к тому, что выдаёт
  \`emit_canonical\`, вперемежку с тем оформлением, которое канонический
  писатель выбрасывает.`,
    zh: `- **\`format_str\` —— 保留注释的格式化器。** 它把文档的结构写法规范到
  \`emit_canonical\` 的产物,并穿插保留规范写入器会丢弃的附属内容。`,
  },
  {
    id: 'v0-7-1-011',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Every comment
  survives verbatim. Blank lines survive as a grouping hint, but a run
  of two or more collapses to one and blank padding just inside a
  bracket is dropped — which is what makes the transform a fixed
  point.`,
    ru: `Каждый комментарий сохраняется дословно. Пустые
  строки сохраняются как подсказка группировки, но серия из двух и
  более схлопывается в одну, а пустой отступ сразу внутри скобки
  выбрасывается — именно это делает преобразование неподвижной
  точкой.`,
    zh: `每条注释都逐字保留。空行作为分组提示保留,但连续
  两行及以上会折叠为一行,紧贴括号内侧的空行填充会被丢弃 —— 正是这一点
  让该变换成为不动点。`,
  },
  {
    id: 'v0-7-1-012',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Key order is never changed: canonical form has no
  sorting rule (§ 5.9), and reordering keys would make review diffs
  worse, not better.`,
    ru: `Порядок ключей не меняется
  никогда: у канонической формы нет правила сортировки (§ 5.9), а
  перестановка ключей ухудшила бы диффы на ревью, а не
  улучшила.`,
    zh: `键序永不改变:规范形式没有排序规则(§ 5.9),
  而重排键只会让评审差异更糟,而非更好。`,
  },
  {
    id: 'v0-7-1-013',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `For a document with no comments *and no
  blank lines*, \`format_str\` equals \`emit_canonical\` of its parse.`,
    ru: `Для документа без комментариев *и без пустых
  строк* \`format_str\` совпадает с \`emit_canonical\` его разбора.`,
    zh: `对于既无注释*也无空行*的文档,\`format_str\`
  等于其解析结果的 \`emit_canonical\`。`,
  },
  {
    id: 'v0-7-1-014',
    join: 'block',
    en: `- **\`ktav-fmt\` — an optional command-line formatter**, behind the
  \`cli\` feature. The repository shipped no binary at all before this
  release.`,
    ru: `- **\`ktav-fmt\` — необязательный форматтер командной строки**, за
  feature-флагом \`cli\`. До этого выпуска репозиторий не поставлял ни
  одного исполняемого файла.`,
    zh: `- **\`ktav-fmt\` —— 可选的命令行格式化器**,位于 \`cli\` feature 之后。
  在此版本之前,本仓库不提供任何可执行文件。`,
  },
  {
    id: 'v0-7-1-015',
    join: 'tight',
    common: `
  \`\`\`text
  cargo install ktav --features cli

  ktav-fmt <file>...           format each file in place
  ktav-fmt --stdout <file>     print the result, leave the file alone
  ktav-fmt --check <file>...   exit non-zero if a file is not formatted
  ktav-fmt -                   read one document from stdin
  \`\`\``,
  },
  {
    id: 'v0-7-1-016',
    join: 'tight',
    en: `
  Three flags, parsed by hand: a CLI argument crate is the one place a
  new runtime dependency might have been defensible, and with three
  flags it is not.`,
    ru: `
  Три флага, разобранные вручную: парсер аргументов — то единственное
  место, где новая рантайм-зависимость могла бы быть оправдана, и при
  трёх флагах она не оправдана.`,
    zh: `
  三个选项,手工解析:命令行参数库是唯一一处新增运行时依赖尚可辩护的
  地方,而只有三个选项时并不成立。`,
  },
  {
    id: 'v0-7-1-016b',
    join: 'tight',
    en: `
  It is off by default deliberately. This repository publishes to
  crates.io and attaches no prebuilt binaries, so an unconditional bin
  target would commit the default install surface to a tool only
  Rust-toolchain owners can reach — while every binding exposes the same
  formatter natively and editors go through \`ktav-lsp\`.`,
    ru: `
  Выключен по умолчанию намеренно. Репозиторий публикуется в crates.io и
  не прикладывает готовых бинарей, так что безусловная bin-цель связала
  бы поверхность установки по умолчанию инструментом, доступным только
  владельцам Rust-тулчейна, — тогда как тот же форматтер есть нативно в
  каждом биндинге, а редакторы ходят через \`ktav-lsp\`.`,
    zh: `
  默认关闭是刻意的。本仓库只发布到 crates.io,不附带预编译二进制,
  因此无条件的 bin 目标会把默认安装面绑定到一个只有 Rust 工具链持有者
  才能取用的工具 —— 而每个绑定都原生提供同一个格式化器,编辑器则经由
  \`ktav-lsp\`。`,
  },
  {
    id: 'v0-7-1-017',
    join: 'block',
    en: `### Fixed`,
    ru: `### Fixed`,
    zh: `### 修复`,
  },
  {
    id: 'v0-7-1-018',
    join: 'block',
    en: `- **Root-kind detection used a narrower whitespace set than § 3.3.** A
  first-line pair whose \`:\` separator was followed by a § 3.3
  whitespace code point other than space or tab — U+00A0, for
  instance — was not recognised as opening an Object root.`,
    ru: `- **Определение вида корня использовало более узкий набор пробелов,
  чем § 3.3.** Пара в первой строке, у которой после разделителя
  \`:\` шёл пробельный код-пойнт из § 3.3, отличный от space и tab —
  например U+00A0, — не распознавалась как открывающая объектный
  корень.`,
    zh: `- **根类型判定使用了比 § 3.3 更窄的空白集合。** 首行键值对的 \`:\`
  分隔符之后若跟随 space 与 tab 以外的 § 3.3 空白码位 —— 例如
  U+00A0 —— 不会被识别为开启对象根。`,
  },
  {
    id: 'v0-7-1-019',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `§ 3.3 freezes
  twenty-five code points and states outright that there is no separate,
  narrower "structural" whitespace concept; the predicate used two. One
  predicate feeds all three parsers, so the fix reaches every one of
  them.`,
    ru: `§ 3.3 фиксирует двадцать пять код-пойнтов и прямо
  заявляет, что отдельного, более узкого «структурного» понятия
  пробела не существует; предикат использовал два. Предикат один на все
  три парсера, поэтому исправление доходит до каждого.`,
    zh: `§ 3.3 冻结了二十五个码位,并明确指出不存在另一套更窄的
  「结构性」空白概念;而该谓词只用了两个。这一谓词为全部三个解析器
  所共用,因此修复对每一个都生效。`,
  },
  {
    id: 'v0-7-1-020',
    join: 'block',
    en: `- **Raw key validation measured escape sequences as two bytes.**
  \`\\uXXXX\` (§ 3.7.1) is six bytes and a surrogate pair twelve;
  \`is_valid_key\` and \`check_quoted_key\` now advance by the real
  escape length instead of a blind \`i += 2\`.`,
    ru: `- **Валидация сырых ключей считала escape-последовательность
  двухбайтовой.** \`\\uXXXX\` (§ 3.7.1) занимает шесть байт, а
  суррогатная пара — двенадцать; \`is_valid_key\` и
  \`check_quoted_key\` теперь сдвигаются на настоящую длину escape, а
  не вслепую на \`i += 2\`.`,
    zh: `- **原始键校验把转义序列当作两个字节。** \`\\uXXXX\`(§ 3.7.1)是
  六个字节,代理对则是十二个;\`is_valid_key\` 与 \`check_quoted_key\`
  现在按转义的真实长度前进,而不是盲目地 \`i += 2\`。`,
  },
  {
    id: 'v0-7-1-021',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Observable behavior is
  unchanged — \`u\` and the hex digits are never structural bytes — but
  the correctness no longer rests on that coincidence.`,
    ru: `Наблюдаемое поведение не изменилось — \`u\` и
  шестнадцатеричные цифры никогда не бывают структурными байтами, — но
  корректность больше не держится на этом совпадении.`,
    zh: `可观察行为未变 —— \`u\` 与十六进制数字从不是结构性
  字节 —— 但正确性不再依赖于这一巧合。`,
  },
];
