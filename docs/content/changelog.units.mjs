// Source of truth for CHANGELOG.md, CHANGELOG.ru.md, CHANGELOG.zh.md.
//
// GENERATED FROM THIS FILE — never edit those artifacts by hand. Change
// the units here, then run
//   node tools/docs-gen/src/cli.mjs
// and verify with `--check`, which is what CI runs.
//
// One unit is one granular meaning: a heading, a sentence, a bullet, a
// table row. Its `join` records how it attaches to the unit before it —
// a blank line by default, a newline for `tight`, a space for `flow`,
// nothing for `none`. The map form exists because the three languages
// wrap their lines in different places. A unit is either a full
// translation set or a single `common` block for text that is identical
// in every language (code, tables, version headings).

export default [
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
  {
    id: 'v0-7-0-001',
    join: 'block',
    en: `## [0.7.0] — 2026-09-10`,
    ru: `## [0.7.0] — 2026-09-10`,
    zh: `## [0.7.0] —— 2026-09-10`,
  },
  {
    id: 'v0-7-0-002',
    join: 'block',
    en: `Implements [Ktav 0.7.0](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.md),
released the same day.`,
    ru: `Реализует [Ktav 0.7.0](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.ru.md),
выпущенный в тот же день.`,
    zh: `实现同日发布的
[Ktav 0.7.0](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.zh.md)。`,
  },
  {
    id: 'v0-7-0-003',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `\`spec-version\` metadata moves to \`0.7.0\` and the
pinned \`spec\` submodule moves to the 0.7.0 release commit, so the
conformance corpus this crate is tested against is the released one.`,
    ru: `Метаданные \`spec-version\` переходят на \`0.7.0\`,
а закреплённый submodule \`spec\` — на релизный коммит 0.7.0, так что
conformance-корпус, против которого проверяется crate, — именно выпущенный.`,
    zh: `\`spec-version\` 元数据升至 \`0.7.0\`,固定的 \`spec\` submodule 也移至 0.7.0
发布提交,因此本 crate 所对照的 conformance 语料库正是已发布的那一份。`,
  },
  {
    id: 'v0-7-0-004',
    join: 'block',
    en: `### Breaking`,
    ru: `### Breaking`,
    zh: `### 破坏性变更`,
  },
  {
    id: 'v0-7-0-005',
    join: 'block',
    en: `- **\`( … )\` stripped multi-line strings now strip trailing whitespace
  from every content line** (§ 5.6), matching what the form already did
  to leading whitespace.`,
    ru: `- **Многострочные строки формы \`( … )\` теперь срезают завершающие
  пробелы в каждой содержательной строке** (§ 5.6) — так же, как форма
  уже поступала с ведущими.`,
    zh: `- **\`( … )\` 去缩进多行字符串现在会剥除每个内容行的行尾空白**(§ 5.6),
  与它早已对行首空白所做的一致。`,
  },
  {
    id: 'v0-7-0-006',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Previously trailing whitespace survived
  byte-for-byte, which meant an editor's "trim on save" could silently
  change string content.`,
    ru: `Раньше завершающие пробелы сохранялись
  байт в байт, из-за чего «обрезка пробелов при сохранении» в редакторе
  могла молча изменить содержимое строки.`,
    zh: `此前行尾空白逐字节保留,这意味着编辑器
  的「保存时去除行尾空白」可能悄然改变字符串内容。`,
  },
  {
    id: 'v0-7-0-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`(( … ))\` is unaffected and stays verbatim on
  both edges — use it when trailing whitespace is significant.`,
    ru: `Форма \`(( … ))\` не затронута и
  остаётся дословной с обеих сторон — используйте её, когда завершающие
  пробелы значимы.`,
    zh: `\`(( … ))\` 不受影响,
  两端仍然完全逐字——当行尾空白有意义时请使用该形式。`,
  },
  {
    id: 'v0-7-0-008',
    join: 'tight',
    en: `- **A recognised escape sequence forces String classification**
  (§ 3.7 / § 5.2).`,
    ru: `- **Распознанная escape-последовательность принудительно даёт String**
  (§ 3.7 / § 5.2).`,
    zh: `- **被识别的 escape 序列强制归类为 String**(§ 3.7 / § 5.2)。`,
  },
  {
    id: 'v0-7-0-009',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `A body written as \`\\u0031\` decodes to \`1\` but stays a
  String — the escape is evidence of intent, so the decoded text is no
  longer re-classified as a number or keyword.`,
    ru: `Тело, записанное как \`\\u0031\`, декодируется в \`1\`, но
  остаётся String: escape — свидетельство намерения, поэтому
  декодированный текст больше не переклассифицируется в число или
  ключевое слово.`,
    zh: `写作
  \`\\u0031\` 的 body 解码为 \`1\`,但仍是 String:escape 是意图的证据,因此解码
  后的文本不再被重新归类为数字或关键字。`,
  },
  {
    id: 'v0-7-0-010',
    join: 'tight',
    en: `- **Whitespace is the frozen 25-code-point § 3.3 set everywhere**, with
  no delegation to a host-language Unicode predicate.`,
    ru: `- **Пробельные символы — это замороженный набор из 25 кодовых точек
  § 3.3 везде**, без делегирования Unicode-предикату языка-хозяина.`,
    zh: `- **空白是各处统一的、冻结的 § 3.3 25 码点集合**,不再委派给宿主语言的
  Unicode 判定。`,
  },
  {
    id: 'v0-7-0-011',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `Key-segment
  trimming widens from ASCII-only to that same set (§ 4), so two keys
  differing only by a non-ASCII whitespace character at a trimmed edge
  now collide as one key.`,
    ru: `  Обрезка сегментов ключа расширяется с ASCII-only до того же набора
  (§ 4), поэтому два ключа, различающиеся только не-ASCII пробельным
  символом на обрезаемом краю, теперь совпадают как один ключ.`,
    zh: `键段修剪从仅 ASCII 扩展到同一集合(§ 4),因此仅在被修剪
  边缘相差一个非 ASCII 空白字符的两个键,现在会合并为同一个键。`,
  },
  {
    id: 'v0-7-0-012',
    join: 'tight',
    en: `- **Integers outside the i64 domain are String, including through
  \`ser::to_value\`.** The parser's Integer domain has always been i64
  (§ 5, § 5.2 rule 13); the serde bridge used to produce a wider
  \`Value::Integer\` that changed variant and canonical bytes on the very
  next round-trip.`,
    ru: `- **Целые вне домена i64 — это String, в том числе через
  \`ser::to_value\`.** Integer-домен парсера всегда был i64 (§ 5,
  § 5.2 правило 13); serde-мост же создавал более широкий
  \`Value::Integer\`, который менял и вариант, и canonical-байты на
  ближайшем же roundtrip.`,
    zh: `- **i64 域之外的整数是 String,通过 \`ser::to_value\` 也是如此。** 解析器
  的 Integer 域一直是 i64(§ 5、§ 5.2 规则 13);而 serde 桥接层此前会
  产生更宽的 \`Value::Integer\`,它在下一次 roundtrip 就会改变变体与
  canonical 字节。`,
  },
  {
    id: 'v0-7-0-013',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`to_value\` now yields \`Value::String\` for a \`u64\`,
  \`i128\` or \`u128\` outside i64 — the same \`Value\` parsing that literal
  would give.`,
    ru: `Теперь \`to_value\` для \`u64\`, \`i128\` или
  \`u128\` вне i64 отдаёт \`Value::String\` — ровно тот \`Value\`, который
  дал бы разбор этого литерала.`,
    zh: `现在 \`to_value\` 对超出 i64 的 \`u64\`、\`i128\` 或 \`u128\`
  返回 \`Value::String\`——与解析同一字面量所得的 \`Value\` 完全一致。`,
  },
  {
    id: 'v0-7-0-014',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `In-range values are unchanged.`,
    ru: `Значения внутри диапазона не изменились.`,
    zh: `域内
  数值不变。`,
  },
  {
    id: 'v0-7-0-015',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-7-0-016',
    join: 'block',
    en: `- **Leading BOM handling** (§ 3.1): exactly one leading U+FEFF is
  skipped if it is the document's first code point; the canonical writer
  never emits one.`,
    ru: `- **Обработка ведущего BOM** (§ 3.1): ровно один ведущий U+FEFF
  пропускается, если это первая кодовая точка документа; канонический
  writer его никогда не выводит.`,
    zh: `- **前导 BOM 处理**(§ 3.1):若 U+FEFF 是文档的首个码点,则恰好跳过一个;
  canonical writer 永不输出 BOM。`,
  },
  {
    id: 'v0-7-0-017',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `A U+FEFF anywhere else is ordinary content.`,
    ru: `U+FEFF в любом другом месте — обычное
  содержимое.`,
    zh: `位于其他位置的 U+FEFF 是普通内容。`,
  },
  {
    id: 'v0-7-0-018',
    join: 'tight',
    en: `- **Quoted keys** (§ 5.3.3): quote-aware key and compound scanners, the
  0.7 escape table with quoted-segment validation and decoding, and the
  § 5.9.10 canonical key-form selection and re-escape recipes in every
  writer.`,
    ru: `- **Ключи в кавычках** (§ 5.3.3): quote-aware сканеры ключей и
  компаундов, таблица escape 0.7 с валидацией и декодированием
  quoted-сегментов, а также выбор канонической формы ключа и рецепты
  переэкранирования из § 5.9.10 во всех writer'ах.`,
    zh: `- **带引号的键**(§ 5.3.3):quote-aware 的键与复合值扫描器、带 quoted
  段验证与解码的 0.7 escape 表,以及所有 writer 中 § 5.9.10 的
  canonical 键形式选择与重新转义规则。`,
  },
  {
    id: 'v0-7-0-019',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The thin event path strips quoted-key delimiters.`,
    ru: `Thin event-путь
  снимает разделители quoted-ключа.`,
    zh: `thin event 路径会剥除 quoted 键的
  定界符。`,
  },
  {
    id: 'v0-7-0-020',
    join: 'tight',
    en: `- **\`\\uXXXX\` escapes** decoded in values and in keys (§ 3.7.1), with
  \`BadEscapeSequence\` diagnostics for lone and mismatched surrogates.`,
    ru: `- **Escape-последовательности \`\\uXXXX\`**, декодируемые в значениях и в
  ключах (§ 3.7.1), с диагностикой \`BadEscapeSequence\` для одиночных и
  несогласованных суррогатов.`,
    zh: `- **\`\\uXXXX\` escape** 在值与键中均可解码(§ 3.7.1),对孤立代理与不匹配
  代理给出 \`BadEscapeSequence\` 诊断。`,
  },
  {
    id: 'v0-7-0-021',
    join: 'tight',
    en: `- **Root Array documents** across the serde root serializer and both
  parsers, with the § 5.9.6 / § 5.9.12 Array-root first-item safeguards
  in every writer.`,
    ru: `- **Документы с корневым Array** во всех трёх точках: serde
  root-сериализатор и оба парсера, вместе с предохранителями первого
  элемента Array-корня из § 5.9.6 / § 5.9.12 в каждом writer'е.`,
    zh: `- **Array 作为根的文档**:serde root 序列化器与两个解析器均支持,并在每个
  writer 中带有 § 5.9.6 / § 5.9.12 的 Array 根首元素保护。`,
  },
  {
    id: 'v0-7-0-022',
    join: 'tight',
    en: `- **Writer-side representable-value rejection** (§ 5.9.0) with reason
  codes on all three writer surfaces.`,
    ru: `- **Отклонение непредставимых значений на стороне writer** (§ 5.9.0) с
  reason-кодами на всех трёх writer-поверхностях.`,
    zh: `- **writer 侧对不可表示值的拒绝**(§ 5.9.0),三个 writer 面上均带
  reason code。`,
  },
  {
    id: 'v0-7-0-023',
    join: 'tight',
    en: `- **\`InvalidUtf8\`** as a distinct error category carrying a byte offset,
  on the byte-boundary surface (§ 6.15); **\`UnterminatedQuotedKey\`**
  (§ 6.16) for an unclosed quoted key.`,
    ru: `- **\`InvalidUtf8\`** как отдельная категория ошибки с байтовым смещением
  на байтовой поверхности (§ 6.15); **\`UnterminatedQuotedKey\`** (§ 6.16)
  для незакрытого ключа в кавычках.`,
    zh: `- **\`InvalidUtf8\`** 成为携带字节偏移的独立错误类别,位于字节边界面
  (§ 6.15);**\`UnterminatedQuotedKey\`**(§ 6.16)用于未闭合的带引号键。`,
  },
  {
    id: 'v0-7-0-024',
    join: 'tight',
    en: `- **\`i128\` / \`u128\` support in both deserializers.** They serialized
  correctly before but could not be read back: serde's default methods
  reject those types outright unless overridden.`,
    ru: `- **Поддержка \`i128\` / \`u128\` в обоих десериализаторах.** Раньше они
  корректно сериализовались, но не читались обратно: дефолтные методы
  serde отвергают эти типы, пока их не переопределить.`,
    zh: `- **两个反序列化器都支持 \`i128\` / \`u128\`。** 它们此前能够正确序列化却读不
  回来:除非覆盖,serde 的默认方法会直接拒绝这两个类型。`,
  },
  {
    id: 'v0-7-0-025',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Both the owned and the
  thin path now parse the digits exactly, with no 64-bit or \`f64\` hop.`,
    ru: `Теперь и owned-,
  и thin-путь разбирают цифры точно, без промежуточного 64-битного или
  \`f64\` прыжка.`,
    zh: `现在 owned 与
  thin 两条路径都精确解析数字,不经过 64 位或 \`f64\` 中转。`,
  },
  {
    id: 'v0-7-0-026',
    join: 'tight',
    en: `- Conformance runner extended to the 0.7 corpus, including its
  \`unrepresentable/\` and \`parseable-unrepresentable/\` categories.`,
    ru: `- Conformance-runner расширен на корпус 0.7, включая его категории
  \`unrepresentable/\` и \`parseable-unrepresentable/\`.`,
    zh: `- conformance runner 扩展到 0.7 语料库,包含其 \`unrepresentable/\` 与
  \`parseable-unrepresentable/\` 类别。`,
  },
  {
    id: 'v0-7-0-027',
    join: 'block',
    en: `### Fixed`,
    ru: `### Исправлено`,
    zh: `### 修复`,
  },
  {
    id: 'v0-7-0-028',
    join: 'block',
    en: `- **\`ser::to_value\` now stores the payload the parser would store.**
  Float payloads carried a text-literal \`.0\` mantissa (\`1.0e100\` where
  the parser stores \`1e100\`), and \`f32\` went through ryu's f32
  thresholds rather than the f64 ones the parser uses (\`0.000001\` vs
  \`1e-6\`).`,
    ru: `- **\`ser::to_value\` теперь хранит тот payload, который сохранил бы
  парсер.** Float-payload нёс текстовую мантиссу с \`.0\` (\`1.0e100\` там,
  где парсер хранит \`1e100\`), а \`f32\` проходил через f32-пороги ryu
  вместо f64-порогов, которыми пользуется парсер (\`0.000001\` против
  \`1e-6\`).`,
    zh: `- **\`ser::to_value\` 现在存放解析器会存放的 payload。** Float payload 曾带
  有文本字面量式的 \`.0\` 尾数(解析器存 \`1e100\` 处它存 \`1.0e100\`),而
  \`f32\` 走的是 ryu 的 f32 阈值而非解析器所用的 f64 阈值(\`0.000001\` 对
  \`1e-6\`)。`,
  },
  {
    id: 'v0-7-0-029',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `A document built with \`to_value\` therefore stopped comparing
  equal to itself after \`render\`/\`emit_canonical\` + \`parse\`.`,
    ru: `Из-за этого документ, построенный через \`to_value\`,
  переставал сравниваться равным самому себе после
  \`render\`/\`emit_canonical\` + \`parse\`.`,
    zh: `因此用 \`to_value\` 构建的文档在 \`render\`/\`emit_canonical\` +
  \`parse\` 之后不再与自身相等。`,
  },
  {
    id: 'v0-7-0-030',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Numeric
  bits and canonical output were always correct; only the stored
  representation disagreed.`,
    ru: `Числовые биты и canonical-вывод
  всегда были верны; расходилось только хранимое представление.`,
    zh: `数值位与 canonical 输出一直是正确的,分歧
  仅在于存放的表示。`,
  },
  {
    id: 'v0-7-0-031',
    join: 'tight',
    en: `- **Typed map keys behave identically through both read APIs and both
  write APIs.** \`from_str\` and \`de::from_value\` disagreed about numeric,
  \`bool\`, unit-enum and newtype keys; \`to_string\` and \`ser::to_value\`
  produced different key *names* for the same value (\`1\` vs \`1.0\`) and
  accepted different key types.`,
    ru: `- **Типизированные ключи maps ведут себя одинаково через оба read API и
  оба write API.** \`from_str\` и \`de::from_value\` расходились на
  числовых, \`bool\`, unit-enum и newtype ключах; \`to_string\` и
  \`ser::to_value\` давали разные *имена* ключа для одного значения
  (\`1\` против \`1.0\`) и принимали разные типы ключей.`,
    zh: `- **类型化的 map 键在两个读 API 与两个写 API 上行为一致。** \`from_str\`
  与 \`de::from_value\` 在数字、\`bool\`、unit enum 与 newtype 键上互相分歧;
  \`to_string\` 与 \`ser::to_value\` 对同一个值给出不同的键*名*(\`1\` 对
  \`1.0\`)且接受不同的键类型。`,
  },
  {
    id: 'v0-7-0-032',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Both sides now share one policy.`,
    ru: `Теперь обе стороны
  используют одну политику.`,
    zh: `现在两侧共用一套策略。`,
  },
  {
    id: 'v0-7-0-033',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Keys
  wrapped in \`Some(...)\` were accepted by the writers but rejected by
  both readers — they round-trip now, and a key literally named \`null\`
  stays the string \`"null"\` rather than becoming \`None\`.`,
    ru: `Ключи, обёрнутые в \`Some(...)\`, писались
  обоими writer'ами, но отвергались обоими reader'ами — теперь они
  проходят roundtrip, а ключ с буквальным именем \`null\` остаётся
  строкой \`"null"\`, а не становится \`None\`.`,
    zh: `被 \`Some(...)\` 包裹的
  键此前可被 writer 接受却被两个 reader 拒绝——现在可以完整 roundtrip,
  而字面名为 \`null\` 的键保持为字符串 \`"null"\`,不会变成 \`None\`。`,
  },
  {
    id: 'v0-7-0-034',
    join: 'tight',
    en: `- **Inline-compound scanning**, across many edge cases surfaced by the
  0.7 conformance corpus and the review series: quote tracking keyed to
  the correct scope, quotes in a value no longer shielding a structural
  closer, mid-scalar openers treated as literal bytes, raw scalars
  terminating at any unescaped closer, per-scope raw closers and scope
  restore, comma key-context derived from the active scope, EOF after a
  whitespace skip, and a bracket in an inline-key position reported as
  \`InvalidKey\` rather than a phantom compound error.`,
    ru: `- **Сканирование inline-компаундов** во множестве краевых случаев,
  вскрытых corpus 0.7 и серией ревью: отслеживание кавычек привязано к
  правильному scope, кавычки в значении больше не заслоняют структурный
  закрыватель, открыватели в середине скаляра трактуются как обычные
  байты, сырые скаляры завершаются на любом неэкранированном
  закрывателе, пер-scope сырые закрыватели и восстановление scope,
  key-контекст запятой выводится из активного scope, EOF после
  пропуска пробелов, а скобка в позиции inline-ключа даёт \`InvalidKey\`,
  а не фантомную ошибку компаунда.`,
    zh: `- **inline 复合值扫描** 的大量边界情形,由 0.7 语料库与系列评审揭示:引号
  跟踪绑定到正确的 scope、值中的引号不再遮蔽结构性闭合符、标量中部的开启
  符视作普通字节、裸标量在任何未转义闭合符处终止、逐 scope 的裸闭合符与
  scope 恢复、逗号的键上下文取自当前 scope、空白跳过后的 EOF,以及 inline
  键位置上的方括号报告为 \`InvalidKey\` 而非幻影复合值错误。`,
  },
  {
    id: 'v0-7-0-035',
    join: 'tight',
    en: `- **§ 5.6 dedent measures the common prefix in § 3.3 code points, not
  bytes**, through a single shared prefix scan subtracted per non-blank
  line.`,
    ru: `- **Дедент § 5.6 измеряет общий префикс в кодовых точках § 3.3, а не в
  байтах**, через единственный общий скан префикса, вычитаемый из
  каждой непустой строки.`,
    zh: `- **§ 5.6 去缩进以 § 3.3 码点而非字节度量公共前缀**,通过一次共享的前缀
  扫描,并从每个非空行中扣除。`,
  },
  {
    id: 'v0-7-0-036',
    join: 'tight',
    en: `- **§ 5.3.2 dotted-key re-entry** in the thin event parser, plus
  compound child-path registration and merge frames for bare compound
  openers.`,
    ru: `- **Повторный вход в точечный ключ (§ 5.3.2)** в thin event-парсере,
  плюс регистрация дочерних путей компаунда и merge-фреймы для голых
  открывателей компаундов.`,
    zh: `- thin event 解析器中的 **§ 5.3.2 点分键重入**,以及复合值子路径注册与裸
  复合开启符的 merge frame。`,
  },
  {
    id: 'v0-7-0-037',
    join: 'tight',
    en: `- **\`i64::MIN\` is represented exactly** from a negative prefixed integer
  literal.`,
    ru: `- **\`i64::MIN\` представляется точно** из отрицательного целочисленного
  литерала с префиксом.`,
    zh: `- **\`i64::MIN\` 由带前缀的负整数字面量精确表示**。`,
  },
  {
    id: 'v0-7-0-038',
    join: 'tight',
    en: `- Empty root tuple-variant names are rejected by the serde text
  serializer.`,
    ru: `- Пустые имена корневых tuple-variant отвергаются текстовым
  serde-сериализатором.`,
    zh: `- serde 文本序列化器拒绝空的根 tuple-variant 名称。`,
  },
  {
    id: 'v0-7-0-039',
    join: 'block',
    en: `### Performance`,
    ru: `### Производительность`,
    zh: `### 性能`,
  },
  {
    id: 'v0-7-0-040',
    join: 'block',
    en: `No timing figures are claimed for this release — the work below was
driven by source-level analysis and allocation counters, not benchmarks.`,
    ru: `Никаких замеров времени для этого релиза не заявляется — работа ниже
основана на анализе исходников и счётчиках аллокаций, не на benchmark'ах.`,
    zh: `本次发布不声称任何计时数据——以下工作基于源码级分析与分配计数器,而非
benchmark。`,
  },
  {
    id: 'v0-7-0-041',
    join: 'block',
    en: `- Inline compound boundaries are computed once and reused through an
  \`InlineBounds\` memo; the three inline scanners are one shared state
  machine; \`ScopeFrame\` packs into a single byte.`,
    ru: `- Границы inline-компаундов вычисляются один раз и переиспользуются
  через memo \`InlineBounds\`; три inline-сканера сведены в один общий
  автомат; \`ScopeFrame\` упакован в один байт.`,
    zh: `- inline 复合值边界只计算一次,并通过 \`InlineBounds\` memo 复用;三个
  inline 扫描器合为一个共享状态机;\`ScopeFrame\` 压缩进单个字节。`,
  },
  {
    id: 'v0-7-0-042',
    join: 'tight',
    en: `- The thin path scans inline compounds straight into events instead of
  building an owned \`Value\` detour, stages inline arrays as flat event
  blocks, and registers child paths in one pass through a shared
  path-node index.`,
    ru: `- Thin-путь сканирует inline-компаунды сразу в события, без обходного
  построения owned \`Value\`, складывает inline-массивы плоскими блоками
  событий и регистрирует дочерние пути за один проход через общий
  индекс path-узлов.`,
    zh: `- thin 路径直接把 inline 复合值扫描为事件,不再绕行构建 owned \`Value\`,
  把 inline 数组以扁平事件块暂存,并通过共享的 path 节点索引一次性注册
  子路径。`,
  },
  {
    id: 'v0-7-0-043',
    join: 'tight',
    en: `- Decoders borrow when there is nothing to unescape.`,
    ru: `- Декодеры заимствуют, когда снимать экранирование нечего.`,
    zh: `- 无需去转义时解码器采用借用。`,
  },
  {
    id: 'v0-7-0-044',
    join: 'tight',
    en: `- The § 5.6 prefix scan is taken lazily over an iterator, and the
  canonical multi-line body is no longer split and rejoined.`,
    ru: `- Скан префикса § 5.6 берётся лениво, поверх итератора, а канонический
  многострочный body больше не разбивается и не сшивается обратно.`,
    zh: `- § 5.6 前缀扫描改为在迭代器上惰性进行,canonical 多行 body 不再拆分后
  再拼接。`,
  },
  {
    id: 'v0-7-0-045',
    join: 'tight',
    en: `- Map key names are written straight into the inline-capable \`Scalar\`,
  including through \`collect_str\` (\`Display\` and \`fmt::Arguments\` keys);
  a short key name no longer allocates a temporary \`String\`.`,
    ru: `- Имена ключей maps пишутся прямо в inline-способный \`Scalar\`, в том
  числе через \`collect_str\` (ключи \`Display\` и \`fmt::Arguments\`);
  короткое имя ключа больше не выделяет временный \`String\`.`,
    zh: `- map 键名直接写入可 inline 的 \`Scalar\`,包括经由 \`collect_str\`
  (\`Display\` 与 \`fmt::Arguments\` 键);短键名不再分配临时 \`String\`。`,
  },
  {
    id: 'v0-7-0-046',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `On the read
  side an inline key is handed over as a slice, while an existing heap
  buffer is still moved into the target rather than copied.`,
    ru: `На стороне
  чтения inline-ключ отдаётся срезом, а уже существующий heap-буфер
  по-прежнему передаётся владением, а не копируется.`,
    zh: `读取
  侧 inline 键以切片交付,而已有的 heap 缓冲区仍以移动而非复制交给目标。`,
  },
  {
    id: 'v0-7-0-047',
    join: 'tight',
    en: `- The i64-domain check for integers is a native range comparison instead
  of formatting and re-parsing the decimal text.`,
    ru: `- Проверка i64-домена для целых — нативное сравнение диапазона вместо
  форматирования и повторного разбора десятичного текста.`,
    zh: `- 整数的 i64 域检查改为原生范围比较,不再格式化后重新解析十进制文本。`,
  },
  {
    id: 'v0-7-0-048',
    join: 'block',
    en: `### Notes`,
    ru: `### Заметки`,
    zh: `### 备注`,
  },
  {
    id: 'v0-7-0-049',
    join: 'block',
    en: `- MSRV stays \`1.71\`.`,
    ru: `- MSRV остаётся \`1.71\`.`,
    zh: `- MSRV 仍为 \`1.71\`。`,
  },
  {
    id: 'v0-7-0-050',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Dependencies are unchanged.`,
    ru: `Зависимости не изменились.`,
    zh: `依赖未变。`,
  },
  {
    id: 'v0-7-0-051',
    join: 'tight',
    en: `- Seventeen rounds of independent implementation review against spec
  0.7.0 are archived under \`docs/reviews/\`; every finding is either
  fixed here or explicitly recorded there as a profiling candidate.`,
    ru: `- Семнадцать раундов независимого ревью реализации против спеки 0.7.0
  заархивированы в \`docs/reviews/\`; каждая находка либо исправлена
  здесь, либо явно зафиксирована там как кандидат для профилирования.`,
    zh: `- 针对 spec 0.7.0 的十七轮独立实现评审归档于 \`docs/reviews/\`;每条发现要么
  已在此修复,要么在那里明确记录为待剖析的候选项。`,
  },
  {
    id: 'v0-6-4-001',
    join: 'block',
    en: `## [0.6.4] — 2026-08-23`,
    ru: `## [0.6.4] — 2026-08-23`,
    zh: `## [0.6.4] —— 2026-08-23`,
  },
  {
    id: 'v0-6-4-002',
    join: 'block',
    en: `### Fixed`,
    ru: `### Исправлено`,
    zh: `### 修复`,
  },
  {
    id: 'v0-6-4-003',
    join: 'block',
    en: `- **Strict parsing now accepts the canonical notation emitted by the
  writer for Float values.** Scientific forms such as \`1e-3\`, \`1.5e-3\`,
  \`-1e-3\`, and \`1e7\` are compared against the § 5.9.8 writer policy,
  while the stored value remains the same Ryu form used by ordinary
  \`parse()\`.`,
    ru: `- **Строгий разбор теперь принимает каноническую запись Float,
  которую выдаёт writer.** Формы \`1e-3\`, \`1.5e-3\`, \`-1e-3\` и \`1e7\`
  сравниваются с policy из § 5.9.8, а хранимое значение остаётся той
  же Ryu-формой, что и при обычном \`parse()\`.`,
    zh: `- **严格解析现在接受 writer 为 Float 输出的规范形式。** \`1e-3\`、
  \`1.5e-3\`、\`-1e-3\` 与 \`1e7\` 等科学形式按 § 5.9.8 policy 比较,
  但存储的值仍保持与普通 \`parse()\` 相同的 Ryu 形式。`,
  },
  {
    id: 'v0-6-4-004',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-6-4-005',
    join: 'block',
    en: `- The crate is released as \`0.6.4\` and pins the conformance specification
  to \`0.6.4\`.`,
    ru: `- Crate выпускается как \`0.6.4\`, а conformance-спецификация закреплена
  на \`0.6.4\`.`,
    zh: `- crate 版本为 \`0.6.4\`,conformance 规范固定为 \`0.6.4\`。`,
  },
  {
    id: 'v0-6-4-006',
    join: 'tight',
    en: `- The \`ktav-lang/spec\` submodule is pinned to the matching 0.6.4
  specification commit, including the float notation-boundary fixture.`,
    ru: `- Submodule \`ktav-lang/spec\` закреплён на соответствующем коммите
  спецификации 0.6.4, включая fixture границ float-записи.`,
    zh: `- \`ktav-lang/spec\` submodule 固定到对应的 0.6.4 规范 commit,
  包含 float 表示边界 fixture。`,
  },
  {
    id: 'v0-6-3-001',
    join: 'block',
    en: `## [0.6.3] — 2026-08-23`,
    ru: `## [0.6.3] — 2026-08-23`,
    zh: `## [0.6.3] —— 2026-08-23`,
  },
  {
    id: 'v0-6-3-002',
    join: 'block',
    en: `### Fixed`,
    ru: `### Исправлено`,
    zh: `### 修复`,
  },
  {
    id: 'v0-6-3-003',
    join: 'block',
    en: `- **Keys accept all ten § 3.7 escape sequences, not three.** § 3.7 / § 4
  define \`\\\\\`, \`\\,\`, \`\\}\`, \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\`, \`\\.\`, \`\\:\` as
  valid in a key; only \`\\\\\`, \`\\.\` and \`\\:\` actually worked.`,
    ru: `- **Ключи принимают все десять escape-последовательностей § 3.7, а не
  три.** § 3.7 / § 4 объявляют допустимыми в ключе \`\\\\\`, \`\\,\`, \`\\}\`,
  \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\`, \`\\.\`, \`\\:\`; реально работали только
  \`\\\\\`, \`\\.\` и \`\\:\`.`,
    zh: `- **键现在接受 § 3.7 全部十种转义序列，而非三种。** § 3.7 / § 4 规定
  \`\\\\\`、\`\\,\`、\`\\}\`、\`\\]\`、\`\\{\`、\`\\[\`、\`\\n\`、\`\\r\`、\`\\.\`、\`\\:\` 在键中均
  合法；但实际只有 \`\\\\\`、\`\\.\` 和 \`\\:\` 可用。`,
  },
  {
    id: 'v0-6-3-004',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`a\\,b: 1\`
  and the six other forms were rejected with \`InvalidKey\`.`,
    ru: `Запись \`a\\,b: 1\` и шесть других форм отвергались
  с \`InvalidKey\`.`,
    zh: `\`a\\,b: 1\` 及其余六种形式
  都会以 \`InvalidKey\` 被拒绝。`,
  },
  {
    id: 'v0-6-3-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The cause
  was ordering: the key was escape-decoded first, then re-validated,
  and validation blanket-rejected \`,\` \`{\` \`}\` \`[\` \`]\` \`LF\` \`CR\` in the
  decoded bytes without distinguishing a byte that arrived through a
  legitimate escape from one written raw.`,
    ru: `Причина — в порядке проверок: ключ сначала
  декодировался, а потом валидировался повторно, и валидация
  безоговорочно запрещала \`,\` \`{\` \`}\` \`[\` \`]\` \`LF\` \`CR\` в уже
  декодированных байтах, не различая байт, пришедший через легитимный
  escape, и байт, записанный сырым.`,
    zh: `根因在于顺序：键先被转义解码，随后再次
  校验，而校验会一律拒绝解码后字节中的 \`,\` \`{\` \`}\` \`[\` \`]\` \`LF\` \`CR\`，
  并不区分该字节是经由合法转义而来，还是原样写入的。`,
  },
  {
    id: 'v0-6-3-006',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Validation now runs on the
  raw (pre-decode) segment, so raw structural bytes stay rejected while
  their escaped forms pass ([#7](https://github.com/ktav-lang/rust/issues/7)).`,
    ru: `Теперь валидация работает по
  сырому (до декодирования) сегменту: сырые структурные байты
  по-прежнему запрещены, а их экранированные формы проходят
  ([#7](https://github.com/ktav-lang/rust/issues/7)).`,
    zh: `现在校验改在
  原始（解码前）的段上进行：原样的结构字节依旧被拒绝，而其转义形式
  得以通过（[#7](https://github.com/ktav-lang/rust/issues/7)）。`,
  },
  {
    id: 'v0-6-3-007',
    join: 'tight',
    en: `- **The event parser and the tree parser no longer disagree on keys.**
  \`src/thin/event_parser.rs\` carried its own copy of the key check with
  the same post-decode bug, plus a second divergence of its own: it
  omitted \`LF\`/\`CR\` from the forbidden set, so \`parse()\` and
  \`from_str::<T>()\` already accepted different key sets.`,
    ru: `- **Event-парсер и tree-парсер больше не расходятся по ключам.**
  В \`src/thin/event_parser.rs\` жила собственная копия проверки ключа с
  той же ошибкой порядка плюс собственное расхождение: в её списке
  запрещённых байтов не было \`LF\`/\`CR\`, поэтому \`parse()\` и
  \`from_str::<T>()\` уже принимали разные наборы ключей.`,
    zh: `- **事件解析器与树解析器不再对键产生分歧。**
  \`src/thin/event_parser.rs\` 自带一份键校验副本，除了同样的顺序缺陷，
  还有它自己的偏差：其禁止集合遗漏了 \`LF\`/\`CR\`，因此 \`parse()\` 与
  \`from_str::<T>()\` 早已接受不同的键集合。`,
  },
  {
    id: 'v0-6-3-008',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The duplicate
  is gone; both front ends now share one validator.`,
    ru: `Дубликат
  удалён; оба фронтенда используют один валидатор.`,
    zh: `该副本已删除；两个前端现在
  共用同一个校验器。`,
  },
  {
    id: 'v0-6-3-009',
    join: 'tight',
    en: `- **The writer emits the full escape set.** \`push_escaped_key_segment\`
  escaped only \`\\\\\`, \`.\` and \`:\`, so a \`Value\` built through the API
  (rather than parsed) with a key containing \`,\` \`{\` \`}\` \`[\` \`]\` \`LF\`
  or \`CR\` serialised to a document that did not parse back — on all
  three writer surfaces, which share the helper (\`emit_canonical\`,
  serde \`to_string\`, \`render\` / \`to_string_force_strings\`).`,
    ru: `- **Writer выводит полный набор escape.** \`push_escaped_key_segment\`
  экранировал только \`\\\\\`, \`.\` и \`:\`, поэтому \`Value\`, собранный через
  API (а не разбором), с ключом, содержащим \`,\` \`{\` \`}\` \`[\` \`]\` \`LF\`
  или \`CR\`, сериализовался в документ, который обратно не разбирался, —
  на всех трёх поверхностях записи, использующих этот хелпер
  (\`emit_canonical\`, serde \`to_string\`, \`render\` /
  \`to_string_force_strings\`).`,
    zh: `- **写入端输出完整的转义集合。** \`push_escaped_key_segment\` 只转义
  \`\\\\\`、\`.\` 和 \`:\`，因此通过 API 构造（而非解析得到）的 \`Value\`，若键
  中含有 \`,\` \`{\` \`}\` \`[\` \`]\` \`LF\` 或 \`CR\`，序列化出的文档将无法被解析
  回来 —— 三个写入面因共用该辅助函数而全部受影响（\`emit_canonical\`、
  serde \`to_string\`、\`render\` / \`to_string_force_strings\`）。`,
  },
  {
    id: 'v0-6-3-010',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`(\` and \`)\`
  are deliberately left alone: § 3.7 defines no escape for them, so a
  key containing one stays unrepresentable
  ([#5](https://github.com/ktav-lang/rust/issues/5) covers refusing
  those explicitly, on the \`0.7\` branch).`,
    ru: `\`(\` и \`)\` намеренно не трогаем: § 3.7 не
  определяет для них escape, поэтому ключ с такой скобкой остаётся
  непредставимым (явный отказ по ним — предмет
  [#5](https://github.com/ktav-lang/rust/issues/5), в ветке \`0.7\`).`,
    zh: `\`(\` 与 \`)\`
  刻意保持原状：§ 3.7 未为其定义转义，因此含有这两个字符的键仍不可
  表示（对其显式拒绝属于 [#5](https://github.com/ktav-lang/rust/issues/5)，
  在 \`0.7\` 分支处理）。`,
  },
  {
    id: 'v0-6-3-011',
    join: 'block',
    en: `Non-breaking: every input that parsed before still parses to the same
\`Value\`, and the writer output only changes for keys whose previous
output did not parse at all.`,
    ru: `Изменение не ломающее: всё, что разбиралось раньше, разбирается в тот
же \`Value\`, а вывод writer'а меняется только для тех ключей, чей
прежний вывод вообще не разбирался.`,
    zh: `本次变更不具破坏性：此前能解析的输入仍解析为相同的 \`Value\`；写入端的
输出仅在那些此前根本无法解析回来的键上发生改变。`,
  },
  {
    id: 'v0-6-3-012',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-6-3-013',
    join: 'block',
    en: `- The pinned \`ktav-lang/spec\` submodule now carries conformance
  fixtures for the seven previously-untested escapes, four negative
  fixtures pinning the boundary (raw \`{\` / \`}\` still \`InvalidKey\`;
  \`\\(\` / \`\\)\` still \`BadEscapeSequence\`), and the § 5.9.3 wording fix
  for the empty-compound root wrap
  ([spec#6](https://github.com/ktav-lang/spec/issues/6),
  [spec#4](https://github.com/ktav-lang/spec/issues/4)).`,
    ru: `- Закреплённый submodule \`ktav-lang/spec\` теперь содержит conformance-
  фикстуры для семи ранее не покрытых escape, четыре негативные
  фикстуры, закрепляющие границу (сырые \`{\` / \`}\` по-прежнему
  \`InvalidKey\`; \`\\(\` / \`\\)\` по-прежнему \`BadEscapeSequence\`), и правку
  формулировки § 5.9.3 про обёртку корня для пустых компаундов
  ([spec#6](https://github.com/ktav-lang/spec/issues/6),
  [spec#4](https://github.com/ktav-lang/spec/issues/4)).`,
    zh: `- 固定的 \`ktav-lang/spec\` 子模块现已包含：此前未覆盖的七种转义的
  一致性测试夹具、四个用于钉住边界的负向夹具（原样的 \`{\` / \`}\` 仍为
  \`InvalidKey\`；\`\\(\` / \`\\)\` 仍为 \`BadEscapeSequence\`），以及关于空复合
  根包裹的 § 5.9.3 措辞修正
  （[spec#6](https://github.com/ktav-lang/spec/issues/6)、
  [spec#4](https://github.com/ktav-lang/spec/issues/4)）。`,
  },
  {
    id: 'v0-6-3-014',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The suite goes
  from 102 valid / 31 invalid to 110 valid / 35 invalid.`,
    ru: `Набор вырос со
  102 valid / 31 invalid до 110 valid / 35 invalid.`,
    zh: `测试集从
  102 valid / 31 invalid 增至 110 valid / 35 invalid。`,
  },
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
  {
    id: 'v0-6-1-001',
    join: 'block',
    common: `## [0.6.1] — 2026-06-05`,
  },
  {
    id: 'v0-6-1-002',
    join: 'block',
    en: `- Docs: rewrite all README examples to spec 0.6 syntax (bare numbers instead of removed \`:i\`/\`:f\` markers; \`##\` comments instead of \`#\`).`,
    ru: `- Документация: все примеры в README переписаны под синтаксис спецификации 0.6 (голые числа вместо удалённых маркеров \`:i\`/\`:f\`; комментарии \`##\` вместо \`#\`).`,
    zh: `- 文档：将所有 README 示例改写为 spec 0.6 语法（裸数字替代已移除的 \`:i\`/\`:f\` 标记；\`##\` 注释替代 \`#\`）。`,
  },
  {
    id: 'v0-6-0-001',
    join: 'block',
    en: `## [0.6.0] — 2026-06-01`,
    ru: `## [0.6.0] — 2026-06-01`,
    zh: `## [0.6.0] —— 2026-06-01`,
  },
  {
    id: 'v0-6-0-002',
    join: 'block',
    en: `Implements Ktav specification 0.6.0. Adds **key escaping**: keys now
process the §3.7 escape set, and two new escapes — \`\\.\` and \`\\:\` —
allow literal dot/colon characters inside a key segment.`,
    ru: `Реализация спецификации Ktav 0.6.0. Добавлено **экранирование в
ключах**: ключи теперь обрабатывают набор escape-последовательностей
из §3.7, и два новых escape — \`\\.\` и \`\\:\` — позволяют использовать
буквальные точку/двоеточие внутри сегмента ключа.`,
    zh: `实现 Ktav 规范 0.6.0。新增 **键转义**:键现在处理 §3.7 转义集,并
新增两条转义 —— \`\\.\` 与 \`\\:\` —— 允许在键段内出现字面意义的点 / 冒号。`,
  },
  {
    id: 'v0-6-0-003',
    join: 'block',
    en: `### Breaking`,
    ru: `### Breaking`,
    zh: `### 破坏性变更`,
  },
  {
    id: 'v0-6-0-004',
    join: 'block',
    en: `- A literal backslash in a key now requires \`\\\\\`.`,
    ru: `- Буквальный символ \`\\\` в ключе теперь требует \`\\\\\`.`,
    zh: `- 键中的字面 \`\\\` 现在需要写作 \`\\\\\`。`,
  },
  {
    id: 'v0-6-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Previously the
  parser treated \`\\\` in a key as an opaque content byte (no escape
  processing).`,
    ru: `Раньше парсер
  трактовал \`\\\` в ключе как обычный байт без обработки escape.`,
    zh: `此前解析器把键中的 \`\\\` 当作
  无转义的内容字节。`,
  },
  {
    id: 'v0-6-0-006',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `Source files that contain a single \`\\\` in a key must
  double it to keep the same key bytes.`,
    ru: `  Файлы-источники с одиночными \`\\\` в ключах нужно удвоить, чтобы
  сохранить те же байты ключа.`,
    zh: `源文件中含单个 \`\\\` 的键需要改为双反斜杠以保
  持相同的键字节。`,
  },
  {
    id: 'v0-6-0-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Backslashes in values are
  unchanged.`,
    ru: `Значения не затронуты.`,
    zh: `值的处理不变。`,
  },
  {
    id: 'v0-6-0-008',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-6-0-009',
    join: 'block',
    en: `- Escape table grows from 8 to 10 sequences: the existing eight
  (\`\\\\\`, \`\\,\`, \`\\}\`, \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\`) plus the two new
  key-oriented ones — \`\\.\` (literal dot in a key segment, does NOT
  split the dotted path) and \`\\:\` (literal colon, does NOT act as the
  pair separator).`,
    ru: `- Таблица escape расширена с 8 до 10 последовательностей: к восьми
  существующим (\`\\\\\`, \`\\,\`, \`\\}\`, \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\`)
  добавлены \`\\.\` (буквальная точка в сегменте ключа — НЕ разделяет
  путь) и \`\\:\` (буквальное двоеточие — НЕ работает как разделитель
  ключ/значение).`,
    zh: `- 转义表由 8 条扩展到 10 条:原有 8 条(\`\\\\\`、\`\\,\`、\`\\}\`、\`\\]\`、
  \`\\{\`、\`\\[\`、\`\\n\`、\`\\r\`)之上新增 \`\\.\`(键段中的字面点 —— 不分割
  路径)与 \`\\:\`(字面冒号 —— 不作为键/值分隔符)。`,
  },
  {
    id: 'v0-6-0-010',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The two new escapes are valid (redundant) in value
  contexts too.`,
    ru: `Оба новых escape допустимы (избыточны) и в
  значениях.`,
    zh: `两条新转义在值
  上下文中亦合法(冗余)。`,
  },
  {
    id: 'v0-6-0-011',
    join: 'tight',
    en: `- The escape-aware key scanner now treats the **first unescaped** \`:\`
  (or \`::\`) as the pair separator and splits dotted paths on
  **unescaped** \`.\` only.`,
    ru: `- Сканер ключей учитывает экранирование: первый **неэкранированный**
  \`:\` (или \`::\`) — разделитель пары; путь разбивается только по
  **неэкранированным** \`.\`.`,
    zh: `- 键扫描器支持转义:首个 **未转义** 的 \`:\`(或 \`::\`)为对分隔符;
  点分路径仅在 **未转义** 的 \`.\` 处分割。`,
  },
  {
    id: 'v0-6-0-012',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Inline-compound keys (\`{a\\.b: 1}\`) follow
  the same rules.`,
    ru: `То же действует для inline-compound
  (\`{a\\.b: 1}\`).`,
    zh: `内联 compound
  (\`{a\\.b: 1}\`)采用同样规则。`,
  },
  {
    id: 'v0-6-0-013',
    join: 'tight',
    en: `- Render path re-escapes \`\\\`, \`.\`, \`:\` in every emitted key segment,
  guaranteeing parse → render → parse identity for keys containing
  literal dots or colons.`,
    ru: `- Рендерер экранирует обратно \`\\\`, \`.\`, \`:\` в каждом сегменте ключа —
  гарантия parse → render → parse тождественности для ключей с
  буквальными точкой/двоеточием.`,
    zh: `- 渲染器在输出每个键段时回写转义 \`\\\`、\`.\`、\`:\`,保证含字面点/冒
  号的键的 parse → render → parse 同一性。`,
  },
  {
    id: 'v0-6-0-014',
    join: 'tight',
    en: `- Zero-copy event path keeps borrowing the source slice for a key
  segment that has no \`\\\`; segments with escapes are decoded into the
  bump arena so the event's \`&'a str\` lifetime is preserved.`,
    ru: `- В zero-copy event-пути сегмент без \`\\\` остаётся заимствованным из
  исходного буфера; сегмент с escape декодируется в bump-arena —
  время жизни \`&'a str\` сохраняется.`,
    zh: `- 零拷贝事件路径下,不含 \`\\\` 的键段继续从源缓冲区借用;含转义的
  键段解码到 bump-arena —— \`&'a str\` 生命周期保持不变。`,
  },
  {
    id: 'v0-6-0-015',
    join: 'block',
    en: `### Errors`,
    ru: `### Ошибки`,
    zh: `### 错误`,
  },
  {
    id: 'v0-6-0-016',
    join: 'block',
    en: `- \`\\X\` in a key with \`X\` not in the ten-escape set is now a
  \`BadEscapeSequence\`.`,
    ru: `- \`\\X\` в ключе, где \`X\` не входит в десятку, теперь
  \`BadEscapeSequence\`.`,
    zh: `- 键中 \`\\X\`(\`X\` 不在十条转义之列)现在抛出 \`BadEscapeSequence\`。`,
  },
  {
    id: 'v0-6-0-017',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `\`\\.\` and \`\\:\` are no longer \`BadEscapeSequence\`
  in any context.`,
    ru: `\`\\.\` и \`\\:\` больше не \`BadEscapeSequence\`
  ни в одном контексте.`,
    zh: `  \`\\.\` 与 \`\\:\` 在任何上下文均不再是 \`BadEscapeSequence\`。`,
  },
  {
    id: 'v0-5-0-001',
    join: 'block',
    en: `## [0.5.0] — 2026-05-28`,
    ru: `## [0.5.0] — 2026-05-28`,
    zh: `## [0.5.0] —— 2026-05-28`,
  },
  {
    id: 'v0-5-0-002',
    join: 'block',
    en: `Implements Ktav specification 0.5.0. This is a breaking release:
the parser, serializer, and Value model are rewritten for the new
language semantics.`,
    ru: `Реализует спецификацию Ktav 0.5.0. Это ломающий релиз: парсер,
сериализатор и модель Value переписаны под новую семантику языка.`,
    zh: `实现 Ktav 规范 0.5.0。这是一次破坏性发布：解析器、序列化器与 Value
模型均按新的语言语义重写。`,
  },
  {
    id: 'v0-5-0-003',
    join: 'block',
    en: `### Breaking`,
    ru: `### Breaking`,
    zh: `### 破坏性变更`,
  },
  {
    id: 'v0-5-0-004',
    join: 'block',
    en: `- Typed markers \`:i\` and \`:f\` removed.`,
    ru: `- Типизированные маркеры \`:i\` и \`:f\` удалены.`,
    zh: `- 移除类型标记 \`:i\` 与 \`:f\`。`,
  },
  {
    id: 'v0-5-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Numbers, booleans, and \`null\`
  are inferred from the lexical form (spec §§ 3.6, 5.2).`,
    ru: `Числа, булевы значения
  и \`null\` выводятся из лексической формы (спек §§ 3.6, 5.2).`,
    zh: `数字、布尔与 \`null\` 由词法形态推断
  （规范 §§ 3.6、5.2）。`,
  },
  {
    id: 'v0-5-0-006',
    join: 'tight',
    en: `- Comments use \`##\` (line-start only).`,
    ru: `- Комментарии пишутся через \`##\` (только с начала строки).`,
    zh: `- 注释使用 \`##\`（仅限行首）。`,
  },
  {
    id: 'v0-5-0-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `A single \`#\` byte is content.`,
    ru: `Одиночный
  байт \`#\` — это содержимое.`,
    zh: `单个 \`#\` 字节属于内容。`,
  },
  {
    id: 'v0-5-0-008',
    join: 'tight',
    en: `- Bare \`port: 8080\` is now \`Integer(8080)\`, not \`String("8080")\`.`,
    ru: `- Голое \`port: 8080\` теперь \`Integer(8080)\`, а не \`String("8080")\`.`,
    zh: `- 裸写的 \`port: 8080\` 现在是 \`Integer(8080)\`，不再是 \`String("8080")\`。`,
  },
  {
    id: 'v0-5-0-009',
    join: 'tight',
    en: `  Write \`port:: 8080\` to keep a String.`,
    ru: `  Чтобы сохранить String, пишите \`port:: 8080\`.`,
    zh: `  若要保留 String，请写 \`port:: 8080\`。`,
  },
  {
    id: 'v0-5-0-010',
    join: 'tight',
    en: `- Lone \`{\` / \`[\` on the first content line opens a multi-line root
  Object / Array (spec § 5.0.1 rules 4–5).`,
    ru: `- Одиночная \`{\` / \`[\` на первой содержательной строке открывает
  многострочный корневой Object / Array (спек § 5.0.1, правила 4–5).`,
    zh: `- 首个内容行上单独的 \`{\` / \`[\` 开启多行的根 Object / Array
  （规范 § 5.0.1 规则 4–5）。`,
  },
  {
    id: 'v0-5-0-011',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `The 0.1.1 JSONL-style
  semantic is removed.`,
    ru: `  Семантика 0.1.1 в стиле JSONL удалена.`,
    zh: `0.1.1 的 JSONL 式语义已移除。`,
  },
  {
    id: 'v0-5-0-012',
    join: 'tight',
    en: `- \`Float\` Values no longer carry the textual form; canonicalised via
  \`ryu\`.`,
    ru: `- Значения \`Float\` больше не несут текстовую форму; канонизируются
  через \`ryu\`.`,
    zh: `- \`Float\` 值不再携带文本形态；改由 \`ryu\` 规范化。`,
  },
  {
    id: 'v0-5-0-013',
    join: 'tight',
    en: `- Key segments are trimmed of leading/trailing whitespace (spec § 4).`,
    ru: `- Сегменты ключа обрезаются от ведущих и хвостовых пробелов (спек § 4).`,
    zh: `- 键段的首尾空白会被裁剪（规范 § 4）。`,
  },
  {
    id: 'v0-5-0-014',
    join: 'tight',
    en: `- Line terminators are \`LF\`, \`CR\`, or \`CR LF\`; \`CR\` is never a
  content byte.`,
    ru: `- Терминаторы строк — \`LF\`, \`CR\` или \`CR LF\`; \`CR\` никогда не является
  байтом содержимого.`,
    zh: `- 行终止符为 \`LF\`、\`CR\` 或 \`CR LF\`；\`CR\` 永远不作为内容字节。`,
  },
  {
    id: 'v0-5-0-015',
    join: 'tight',
    en: `- \`ErrorKind::InlineNonEmptyCompound\` and \`InvalidTypedScalar\` are
  deprecated (\`#[doc(hidden)]\`); the parser no longer emits them.`,
    ru: `- \`ErrorKind::InlineNonEmptyCompound\` и \`InvalidTypedScalar\` объявлены
  устаревшими (\`#[doc(hidden)]\`); парсер их больше не выдаёт.`,
    zh: `- \`ErrorKind::InlineNonEmptyCompound\` 与 \`InvalidTypedScalar\` 标记为
  废弃（\`#[doc(hidden)]\`）；解析器不再产生它们。`,
  },
  {
    id: 'v0-5-0-016',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-5-0-017',
    join: 'block',
    en: `- **Inline compounds** \`{k: v, …}\` / \`[i, …]\` (spec § 5.8) with
  trailing comma, mid-value brace literal (§ 5.8.5), and nesting
  depth limit of 128.`,
    ru: `- **Инлайн-составные** \`{k: v, …}\` / \`[i, …]\` (спек § 5.8) с хвостовой
  запятой, литеральной скобкой в середине значения (§ 5.8.5) и
  ограничением глубины вложенности в 128.`,
    zh: `- **内联复合** \`{k: v, …}\` / \`[i, …]\`（规范 § 5.8），支持尾随逗号、
  值中部的花括号字面量（§ 5.8.5），嵌套深度上限 128。`,
  },
  {
    id: 'v0-5-0-018',
    join: 'tight',
    en: `- **Eight escape sequences** in inline scalars: \`\\\\\`, \`\\,\`, \`\\}\`,
  \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\` (spec § 3.7).`,
    ru: `- **Восемь escape-последовательностей** в инлайн-скалярах: \`\\\\\`, \`\\,\`,
  \`\\}\`, \`\\]\`, \`\\{\`, \`\\[\`, \`\\n\`, \`\\r\` (спек § 3.7).`,
    zh: `- 内联标量中的**八种转义序列**：\`\\\\\`、\`\\,\`、\`\\}\`、\`\\]\`、\`\\{\`、\`\\[\`、
  \`\\n\`、\`\\r\`（规范 § 3.7）。`,
  },
  {
    id: 'v0-5-0-019',
    join: 'tight',
    en: `- **Number literal grammar** — \`0x\`, \`0o\`, \`0b\`, decimal, underscore
  separators; i64 overflow falls back to String (spec § 3.6).`,
    ru: `- **Грамматика числовых литералов** — \`0x\`, \`0o\`, \`0b\`, десятичные,
  разделители-подчёркивания; переполнение i64 откатывается в String
  (спек § 3.6).`,
    zh: `- **数字字面量文法** —— \`0x\`、\`0o\`、\`0b\`、十进制、下划线分隔符；
  i64 溢出回退为 String（规范 § 3.6）。`,
  },
  {
    id: 'v0-5-0-020',
    join: 'tight',
    en: `- **\`emit_canonical()\`** — spec § 5.9 normative writer output,
  byte-deterministic across implementations.`,
    ru: `- **\`emit_canonical()\`** — нормативный вывод writer-а по спек § 5.9,
  байт-детерминированный между реализациями.`,
    zh: `- **\`emit_canonical()\`** —— 规范 § 5.9 规定的 writer 输出，跨实现
  字节确定。`,
  },
  {
    id: 'v0-5-0-021',
    join: 'tight',
    en: `- **\`src/parser/inline.rs\`** — inline-compound parser.`,
    ru: `- **\`src/parser/inline.rs\`** — парсер инлайн-составных.`,
    zh: `- **\`src/parser/inline.rs\`** —— 内联复合解析器。`,
  },
  {
    id: 'v0-5-0-022',
    join: 'tight',
    en: `- **\`src/render/canonical.rs\`** — canonical writer.`,
    ru: `- **\`src/render/canonical.rs\`** — канонический writer.`,
    zh: `- **\`src/render/canonical.rs\`** —— 规范化 writer。`,
  },
  {
    id: 'v0-5-0-023',
    join: 'tight',
    en: `- New error variants: \`UnterminatedInlineCompound\`,
  \`MalformedInlineCompound\`, \`BadEscapeSequence\`,
  \`OrphanLineAfterTopLevelInline\`.`,
    ru: `- Новые варианты ошибок: \`UnterminatedInlineCompound\`,
  \`MalformedInlineCompound\`, \`BadEscapeSequence\`,
  \`OrphanLineAfterTopLevelInline\`.`,
    zh: `- 新增错误变体：\`UnterminatedInlineCompound\`、
  \`MalformedInlineCompound\`、\`BadEscapeSequence\`、
  \`OrphanLineAfterTopLevelInline\`。`,
  },
  {
    id: 'v0-5-0-024',
    join: 'tight',
    en: `- Triple-test conformance harness (\`tests/spec_conformance.rs\`):
  93 valid + 31 invalid fixtures from \`spec/versions/0.5/tests/\`.`,
    ru: `- Тройной conformance-harness (\`tests/spec_conformance.rs\`):
  93 валидных + 31 невалидная фикстура из \`spec/versions/0.5/tests/\`.`,
    zh: `- 三重 conformance 测试台（\`tests/spec_conformance.rs\`）：取自
  \`spec/versions/0.5/tests/\` 的 93 个有效 + 31 个无效 fixture。`,
  },
  {
    id: 'v0-5-0-025',
    join: 'tight',
    en: `- Parser fast-paths: plain-decimal integers, no-underscore floats,
  ryu-reuse, first-byte fast-reject, LF-only line splitting,
  pre-sized Bump arena.`,
    ru: `- Быстрые пути парсера: простые десятичные целые, float без
  подчёркиваний, переиспользование ryu, быстрый отказ по первому
  байту, разбиение строк только по LF, преднастроенная Bump-арена.`,
    zh: `- 解析器快速路径：纯十进制整数、无下划线浮点、ryu 复用、首字节快速
  拒绝、仅按 LF 切行、预分配的 Bump arena。`,
  },
  {
    id: 'v0-5-0-026',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-5-0-027',
    join: 'block',
    en: `- License: \`MIT\` → \`MIT OR Apache-2.0\`.`,
    ru: `- Лицензия: \`MIT\` → \`MIT OR Apache-2.0\`.`,
    zh: `- 许可证：\`MIT\` → \`MIT OR Apache-2.0\`。`,
  },
  {
    id: 'v0-5-0-028',
    join: 'tight',
    en: `- Spec submodule pinned to \`v0.5.0\` (\`4d0a8aa\`).`,
    ru: `- Сабмодуль спеки закреплён на \`v0.5.0\` (\`4d0a8aa\`).`,
    zh: `- 规范 submodule 固定到 \`v0.5.0\`（\`4d0a8aa\`）。`,
  },
  {
    id: 'v0-5-0-029',
    join: 'tight',
    en: `- Doctests disabled (\`[lib] doctest = false\`); examples remain as
  \`text\` blocks in doc comments.`,
    ru: `- Doctests отключены (\`[lib] doctest = false\`); примеры остаются
  блоками \`text\` в doc-комментариях.`,
    zh: `- 关闭 doctest（\`[lib] doctest = false\`）；示例以 \`text\` 块形式保留在
  文档注释中。`,
  },
  {
    id: 'v0-3-1-001',
    join: 'block',
    en: `## [0.3.1] — 2026-05-10`,
    ru: `## [0.3.1] — 2026-05-10`,
    zh: `## [0.3.1] —— 2026-05-10`,
  },
  {
    id: 'v0-3-1-002',
    join: 'block',
    en: `Backward-compatible feature release tracking spec 0.1.1.`,
    ru: `Обратно совместимый функциональный релиз, следующий за спекой 0.1.1.`,
    zh: `向后兼容的功能发布，跟进规范 0.1.1。`,
  },
  {
    id: 'v0-3-1-003',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-3-1-004',
    join: 'block',
    en: `- **Top-level Array support** (spec § 5.0.1, added in spec 0.1.1) —
  a document whose first content line has an array-item shape
  (bare scalar, \`:: text\`, \`:i 42\`, \`:f 3.14\`, lone \`{\` / \`[\`, or a
  multi-line opener \`(\` / \`((\`) is now parsed as a root-level
  \`Value::Array\`.`,
    ru: `- **Поддержка Array на верхнем уровне** (спек § 5.0.1, добавлено в
  спеке 0.1.1) — документ, первая содержательная строка которого имеет
  форму элемента массива (голый скаляр, \`:: текст\`, \`:i 42\`, \`:f 3.14\`,
  одиночная \`{\` / \`[\` или многострочный опенер \`(\` / \`((\`), теперь
  разбирается как корневой \`Value::Array\`.`,
    zh: `- **顶层 Array 支持**（规范 § 5.0.1，于规范 0.1.1 加入）——首个内容行
  具备数组元素形态（裸标量、\`:: 文本\`、\`:i 42\`、\`:f 3.14\`、单独的
  \`{\` / \`[\`，或多行开启符 \`(\` / \`((\`）的文档，现在解析为根级
  \`Value::Array\`。`,
  },
  {
    id: 'v0-3-1-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Previously the root was always \`Value::Object\`,
  so a bare scalar at line 1 errored as \`MissingSeparator\`.`,
    ru: `Раньше корнем всегда был
  \`Value::Object\`, поэтому голый скаляр в строке 1 давал ошибку
  \`MissingSeparator\`.`,
    zh: `此前根始终是 \`Value::Object\`，因此第 1 行的裸标量
  会以 \`MissingSeparator\` 报错。`,
  },
  {
    id: 'v0-3-1-006',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  Empty / comments-only documents still default to an empty Object
  (preserves 0.3.0 behaviour).`,
    ru: `Пустые документы и документы из одних
  комментариев по-прежнему дают пустой Object (поведение 0.3.0
  сохранено).`,
    zh: `空文档与仅含注释的文档仍默认为空
  Object（保持 0.3.0 行为）。`,
  },
  {
    id: 'v0-3-1-007',
    join: 'tight',
    en: `- **\`ktav::to_string_force_strings(value)\`** — render any \`Value\`
  with every scalar coerced to a String.`,
    ru: `- **\`ktav::to_string_force_strings(value)\`** — рендер любого \`Value\` с
  приведением каждого скаляра к String.`,
    zh: `- **\`ktav::to_string_force_strings(value)\`** —— 渲染任意 \`Value\`，并将
  每个标量强制为 String。`,
  },
  {
    id: 'v0-3-1-008',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Typed integers (\`:i\`),
  typed floats (\`:f\`), booleans, and null are flattened to their
  textual form; compounds preserve their structure.`,
    ru: `Типизированные целые (\`:i\`),
  типизированные float (\`:f\`), булевы значения и null уплощаются в
  текстовую форму; составные сохраняют структуру.`,
    zh: `类型化整数（\`:i\`）、类型化浮点（\`:f\`）、
  布尔与 null 会被展平为其文本形态；复合结构保持不变。`,
  },
  {
    id: 'v0-3-1-009',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The output
  round-trips back through the parser as the same set of String
  scalars.`,
    ru: `Вывод проходит
  round-trip обратно через парсер как тот же набор String-скаляров.`,
    zh: `输出经解析器
  round-trip 后仍是同一组 String 标量。`,
  },
  {
    id: 'v0-3-1-010',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `Useful for "everything is a string" dumps for downstream
  consumers that don't understand typed markers, or for diff-
  friendly canonical text.`,
    ru: `  Полезно для дампов «всё — строка» для потребителей, не понимающих
  типизированных маркеров, или для канонического текста, удобного для
  diff-а.`,
    zh: `适用于面向不理解类型标记的
  下游消费者的「一切皆字符串」转储，或便于 diff 的规范化文本。`,
  },
  {
    id: 'v0-3-1-011',
    join: 'tight',
    en: `- New \`Render\` exit point: \`render\` accepts both Object and Array
  top-level values (top-level Arrays render as bare item-per-line,
  no \`[...]\` brackets).`,
    ru: `- Новая точка выхода \`Render\`: \`render\` принимает на верхнем уровне и
  Object, и Array (Array верхнего уровня рендерится голыми элементами
  по строке, без скобок \`[...]\`).`,
    zh: `- 新的 \`Render\` 出口：\`render\` 在顶层同时接受 Object 与 Array
  （顶层 Array 渲染为每行一个裸元素，不带 \`[...]\` 方括号）。`,
  },
  {
    id: 'v0-3-1-012',
    join: 'block',
    en: `### Compatibility`,
    ru: `### Совместимость`,
    zh: `### 兼容性`,
  },
  {
    id: 'v0-3-1-013',
    join: 'block',
    en: `Strictly additive.`,
    ru: `Строго аддитивно.`,
    zh: `严格增量。`,
  },
  {
    id: 'v0-3-1-014',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Every document valid under 0.3.0 stays valid
under 0.3.1 and produces the same \`Value\`.`,
    ru: `Любой документ, валидный по 0.3.0, остаётся валидным
по 0.3.1 и даёт тот же \`Value\`.`,
    zh: `凡在 0.3.0 下有效的文档，在 0.3.1 下依然有效并产生相同的
\`Value\`。`,
  },
  {
    id: 'v0-3-1-015',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Only inputs 0.3.0
rejected as \`MissingSeparator\` (bare-scalar first lines) are now
accepted as Arrays.`,
    ru: `Только те входы, которые 0.3.0
отвергал как \`MissingSeparator\` (первая строка — голый скаляр), теперь
принимаются как Array.`,
    zh: `只有此前被 0.3.0 以 \`MissingSeparator\` 拒绝的输入（首行为
裸标量）现在被接受为 Array。`,
  },
  {
    id: 'v0-3-1-016',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The error variants and their spans are
unchanged for the Object path.`,
    ru: `Варианты ошибок и их spans для пути Object
не изменились.`,
    zh: `Object 路径上的错误变体及其 span 未变。`,
  },
  {
    id: 'v0-3-1-017',
    join: 'block',
    en: `The parser, render, thin event-parser, and thin event-deserializer
all honour spec § 5.0.1 consistently — \`parse\`, \`parse_events\`, and
\`from_str\` agree on the root kind for any given input.`,
    ru: `Парсер, render, thin event-парсер и thin event-десериализатор
согласованно соблюдают спек § 5.0.1 — \`parse\`, \`parse_events\` и
\`from_str\` дают одинаковый вид корня для любого входа.`,
    zh: `解析器、render、thin 事件解析器与 thin 事件反序列化器一致遵循规范
§ 5.0.1 —— 对任意给定输入，\`parse\`、\`parse_events\` 与 \`from_str\` 对根
的种类判断一致。`,
  },
  {
    id: 'v0-3-0-001',
    join: 'block',
    en: `## [0.3.0] — 2026-05-08`,
    ru: `## [0.3.0] — 2026-05-08`,
    zh: `## [0.3.0] —— 2026-05-08`,
  },
  {
    id: 'v0-3-0-002',
    join: 'block',
    en: `Minor release with one breaking parser strictness change, a
diagnostic-range fix, and hot-path micro-optimisations on the
typed-deserialize path.`,
    ru: `Минорный релиз с одним ломающим ужесточением парсера, исправлением
диапазона диагностики и микрооптимизациями горячего пути
типизированной десериализации.`,
    zh: `次要发布，含一项破坏性的解析器严格化、一处诊断范围修复，以及类型化
反序列化热路径上的微优化。`,
  },
  {
    id: 'v0-3-0-003',
    join: 'block',
    en: `### Fixed`,
    ru: `### Исправлено`,
    zh: `### 修复`,
  },
  {
    id: 'v0-3-0-004',
    join: 'block',
    en: `- \`ErrorKind::DuplicateKey\` and \`ErrorKind::KeyPathConflict\` now carry
  the span of the **offending key**, not the closing \`}\` / \`]\` of the
  compound that would have been assigned to it.`,
    ru: `- \`ErrorKind::DuplicateKey\` и \`ErrorKind::KeyPathConflict\` теперь несут
  span **виновного ключа**, а не закрывающей \`}\` / \`]\` того
  составного, которое ему присваивалось.`,
    zh: `- \`ErrorKind::DuplicateKey\` 与 \`ErrorKind::KeyPathConflict\` 现在携带
  **出错键**自身的 span，而不是本应赋给它的那个复合结构的收尾
  \`}\` / \`]\`。`,
  },
  {
    id: 'v0-3-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Previously, when the
  conflict was detected on \`attach_child_value\` (e.g. \`value: { ... }\`
  duplicating an earlier \`value: ...\`), the saved span pointed at the
  closing brace because that's the position the parser had at hand at
  the moment of detection.`,
    ru: `Раньше, когда конфликт
  обнаруживался на \`attach_child_value\` (например, \`value: { ... }\`,
  дублирующее более раннее \`value: ...\`), сохранённый span указывал на
  закрывающую скобку — это была позиция, которая оказывалась у парсера
  под рукой в момент обнаружения.`,
    zh: `此前，当冲突在 \`attach_child_value\` 处被发现时
  （例如 \`value: { ... }\` 与更早的 \`value: ...\` 重复），保存的 span
  指向收尾括号——那是解析器在发现冲突的那一刻手头持有的位置。`,
  },
  {
    id: 'v0-3-0-006',
    join: 'block',
    en: `  The parser now stores the key's own span (\`pending_key_span\`) on the
  parent frame when a compound is opened, and reuses it when the
  compound closes and the value is attached.`,
    ru: `  Теперь парсер сохраняет собственный span ключа (\`pending_key_span\`)
  в родительском фрейме при открытии составного и переиспользует его,
  когда составное закрывается и значение присваивается.`,
    zh: `  现在解析器在打开复合结构时把键自身的 span（\`pending_key_span\`）
  存入父帧，并在复合结构收尾、值被挂接时复用它。`,
  },
  {
    id: 'v0-3-0-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Editors / IDEs that draw
  diagnostic underlines from \`Span\` now point at the key.`,
    ru: `Редакторы и
  IDE, рисующие диагностические подчёркивания по \`Span\`, теперь
  указывают на ключ.`,
    zh: `依据 \`Span\` 绘制
  诊断下划线的编辑器 / IDE 现在会指向键。`,
  },
  {
    id: 'v0-3-0-008',
    join: 'block',
    en: `  This is a fix for a span value, not an API change — \`ErrorKind\`
  shape is unchanged.`,
    ru: `  Это исправление значения span, а не изменение API — форма
  \`ErrorKind\` не менялась.`,
    zh: `  这是对 span 取值的修复，不是 API 变更 —— \`ErrorKind\` 的形态未变。`,
  },
  {
    id: 'v0-3-0-009',
    join: 'block',
    en: `### Changed (breaking — parser strictness)`,
    ru: `### Изменено (breaking — строгость парсера)`,
    zh: `### 变更（breaking —— 解析器严格化）`,
  },
  {
    id: 'v0-3-0-010',
    join: 'block',
    en: `- \`key: (value)\` and \`key: ((value))\` now error with
  \`ErrorKind::InlineNonEmptyCompound { body: "paren-string" }\`.`,
    ru: `- \`key: (value)\` и \`key: ((value))\` теперь дают ошибку
  \`ErrorKind::InlineNonEmptyCompound { body: "paren-string" }\`.`,
    zh: `- \`key: (value)\` 与 \`key: ((value))\` 现在以
  \`ErrorKind::InlineNonEmptyCompound { body: "paren-string" }\` 报错。`,
  },
  {
    id: 'v0-3-0-011',
    join: 'tight',
    en: `  These shapes used to be accepted as plain string scalars \`(value)\`,
  but they are visually indistinguishable from multi-line openers and
  would confuse readers.`,
    ru: `  Раньше эти формы принимались как обычные строковые скаляры
  \`(value)\`, но визуально они неотличимы от многострочных опенеров и
  сбивали бы читателя с толку.`,
    zh: `  这些形态过去被当作普通字符串标量 \`(value)\` 接受，但它们与多行开启符
  在视觉上无法区分，会让读者困惑。`,
  },
  {
    id: 'v0-3-0-012',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The raw-marker form \`key:: (value)\` remains
  valid and is the canonical way to encode such literals.`,
    ru: `Форма с raw-маркером \`key:: (value)\`
  остаётся валидной и является каноническим способом записать такой
  литерал.`,
    zh: `带 raw 标记的形式 \`key:: (value)\`
  仍然有效，并且是编码此类字面量的规范写法。`,
  },
  {
    id: 'v0-3-0-013',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The
  ktav-lsp formatter auto-rewrites the legacy form on save.`,
    ru: `Форматтер ktav-lsp автоматически переписывает legacy-форму
  при сохранении.`,
    zh: `ktav-lsp 的格式化器会在
  保存时自动改写旧形式。`,
  },
  {
    id: 'v0-3-0-014',
    join: 'block',
    en: `### Optimised (no API change)`,
    ru: `### Оптимизировано (без изменения API)`,
    zh: `### 优化（无 API 变更）`,
  },
  {
    id: 'v0-3-0-015',
    join: 'block',
    en: `- \`render::render\` pre-sizes the output \`String\` with a recursive
  \`estimate_size(value)\` to skip the doubling reallocations that
  \`push_str\` chains would otherwise trigger on multi-KiB outputs.`,
    ru: `- \`render::render\` преднастраивает размер выходного \`String\` через
  рекурсивный \`estimate_size(value)\`, чтобы обойти удваивающие
  реаллокации, которые цепочки \`push_str\` иначе вызывали бы на выводе
  в несколько KiB.`,
    zh: `- \`render::render\` 通过递归的 \`estimate_size(value)\` 预设输出 \`String\`
  的容量，从而跳过 \`push_str\` 链在数 KiB 输出上本会触发的倍增式
  重分配。`,
  },
  {
    id: 'v0-3-0-016',
    join: 'tight',
    en: `- \`EventCursor::peek\` / \`next\` use \`unsafe get_unchecked\` for
  bounds-elision on the hot path; the parser's well-formed-stream
  invariant guarantees \`pos < len()\` on every call.`,
    ru: `- \`EventCursor::peek\` / \`next\` используют \`unsafe get_unchecked\` для
  устранения проверки границ на горячем пути; инвариант
  well-formed-потока парсера гарантирует \`pos < len()\` на каждом
  вызове.`,
    zh: `- \`EventCursor::peek\` / \`next\` 使用 \`unsafe get_unchecked\` 以消除热
  路径上的边界检查；解析器的良构流不变式保证每次调用时
  \`pos < len()\`。`,
  },
  {
    id: 'v0-3-0-017',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Falls back to
  \`None\` when the invariant is violated, so malformed inputs remain
  safe — the unsafe path is a pure branch elision win.`,
    ru: `При нарушении инварианта возвращается \`None\`, поэтому
  некорректные входы остаются безопасными — unsafe-путь здесь чистый
  выигрыш на устранении ветвления.`,
    zh: `不变式被破坏时回退为 \`None\`，因此畸形输入依然安全
  —— 这里的 unsafe 路径纯粹是省去一次分支判断的收益。`,
  },
  {
    id: 'v0-3-0-018',
    join: 'tight',
    en: `- \`MapAccess::next_key_seed\` folds redundant \`peek + next\` into a
  single \`next\` (both branches consume the cursor anyway).`,
    ru: `- \`MapAccess::next_key_seed\` сворачивает избыточные \`peek + next\` в
  один \`next\` (обе ветки всё равно потребляют курсор).`,
    zh: `- \`MapAccess::next_key_seed\` 将冗余的 \`peek + next\` 合并为单次 \`next\`
  （两个分支反正都会消费游标）。`,
  },
  {
    id: 'v0-3-0-019',
    join: 'tight',
    en: `- Event \`BumpVec\` capacity hint raised from \`text.len() / 8 + 16\`
  to \`text.len() / 4 + 64\`.`,
    ru: `- Подсказка ёмкости event-\`BumpVec\` поднята с \`text.len() / 8 + 16\`
  до \`text.len() / 4 + 64\`.`,
    zh: `- 事件 \`BumpVec\` 的容量提示从 \`text.len() / 8 + 16\` 提高到
  \`text.len() / 4 + 64\`。`,
  },
  {
    id: 'v0-3-0-020',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The previous hint underestimated the
  ~1-event-per-5-bytes density on synth fixtures and triggered
  8–10 realloc-copy steps inside the bump arena on a 500 KiB doc.`,
    ru: `Прежняя подсказка недооценивала плотность
  ~1 событие на 5 байт на synth-фикстурах и вызывала 8–10 шагов
  realloc-copy внутри bump-арены на документе в 500 KiB.`,
    zh: `旧提示低估了 synth fixture 上约每 5 字节
  1 个事件的密度，在 500 KiB 文档上会在 bump arena 内触发 8–10 次
  realloc-copy。`,
  },
  {
    id: 'v0-3-0-021',
    join: 'block',
    en: `### Experiment (reverted)`,
    ru: `### Эксперимент (откачен)`,
    zh: `### 实验（已回退）`,
  },
  {
    id: 'v0-3-0-022',
    join: 'block',
    en: `- A streaming-deserializer refactor (parse on demand, no whole-doc
  \`Vec<Event>\`) was implemented and tested — all 404 tests passed,
  but parse_to_struct regressed 15–60 % vs. the existing cursor on
  this hardware.`,
    ru: `- Рефакторинг стримингового десериализатора (разбор по требованию, без
  \`Vec<Event>\` на весь документ) был реализован и протестирован — все
  404 теста прошли, но parse_to_struct регрессировал на 15–60 % против
  существующего курсора на этом железе.`,
    zh: `- 一次流式反序列化器重构（按需解析，不保留整份文档的 \`Vec<Event>\`）
  已实现并测试 —— 全部 404 个测试通过，但在本机硬件上
  parse_to_struct 相对既有游标回退了 15–60 %。`,
  },
  {
    id: 'v0-3-0-023',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Cause: the cursor walks a contiguous slice with
  one monotonic branch the predictor nails 100 %, while streaming
  interleaves parser state-machine work with deserializer work and
  blows the predictor.`,
    ru: `Причина: курсор идёт по
  непрерывному срезу с одной монотонной ветвью, которую предсказатель
  угадывает на 100 %, тогда как стриминг перемежает работу
  конечного автомата парсера с работой десериализатора и сбивает
  предсказатель.`,
    zh: `原因：游标沿连续切片
  行进，只有一条单调分支，预测器命中率 100 %；而流式方案把解析器状态
  机的工作与反序列化器的工作交错，打乱了预测器。`,
  },
  {
    id: 'v0-3-0-024',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The streaming code was removed; the
  \`EventSink<'a>\` trait introduced for the experiment survives in
  \`event.rs\` as harmless generic infrastructure (zero cost when
  used only with \`BumpVec\`).`,
    ru: `Стриминговый код удалён; трейт \`EventSink<'a>\`,
  введённый для эксперимента, остался в \`event.rs\` как безвредная
  обобщённая инфраструктура (нулевая стоимость при использовании
  только с \`BumpVec\`).`,
    zh: `流式代码已删除；为该
  实验引入的 \`EventSink<'a>\` trait 作为无害的泛型基础设施保留在
  \`event.rs\` 中（仅与 \`BumpVec\` 搭配使用时零开销）。`,
  },
  {
    id: 'v0-2-0-001',
    join: 'block',
    en: `## [0.2.0] — 2026-05-07`,
    ru: `## [0.2.0] — 2026-05-07`,
    zh: `## [0.2.0] —— 2026-05-07`,
  },
  {
    id: 'v0-2-0-002',
    join: 'block',
    en: `Minor release with two breaking output / validation changes:`,
    ru: `Минорный релиз с двумя breaking-изменениями вывода / валидации:`,
    zh: `次要发布,带两项 breaking 输出 / 校验改动:`,
  },
  {
    id: 'v0-2-0-003',
    join: 'block',
    en: `### Changed (breaking)`,
    ru: `### Изменено (breaking)`,
    zh: `### 变更(breaking)`,
  },
  {
    id: 'v0-2-0-004',
    join: 'block',
    en: `- **Multi-line strings emit indented stripped form \`( ... )\` by default**,
  not verbatim \`(( ... ))\`.`,
    ru: `- **Многострочные строки по умолчанию выводятся в форме stripped
  \`( ... )\`** с отступом, а не verbatim \`(( ... ))\`.`,
    zh: `- **多行字符串默认输出为缩进 stripped 形式 \`( ... )\`**,而非 verbatim
  \`(( ... ))\`。`,
  },
  {
    id: 'v0-2-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Verbatim is still produced as a fallback when
  the content has its own leading whitespace (which the parser-side
  dedent would clobber) or contains a sole-\`)\` line that would close the
  stripped form prematurely.`,
    ru: `Verbatim остаётся
  как fallback когда содержимое имеет собственный leading-whitespace
  (parser-side dedent его съест) или строку, тримящуюся в \`)\` (закрыла
  бы stripped преждевременно).`,
    zh: `当内容自带前导空白(会被解析侧的 dedent 吞掉)或包含
  仅为 \`)\` 的行(会提前关闭 stripped)时,fallback 到 verbatim。`,
  },
  {
    id: 'v0-2-0-006',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Code that compares \`to_string\` /
  \`render\` output byte-for-byte to a baked-in \`((...))\` literal needs to
  be updated.`,
    ru: `Код, сравнивающий вывод \`to_string\` /
  \`render\` побайтово с фиксированным \`((...))\`, нужно обновить.`,
    zh: `逐字节
  比较 \`to_string\` / \`render\` 输出与硬编码 \`((...))\` 的代码需要更新。`,
  },
  {
    id: 'v0-2-0-007',
    join: { en: 'flow', ru: 'tight', zh: 'tight' },
    en: `Round-tripping (\`parse(to_string(v)) == v\`) is unchanged.`,
    ru: `  Round-trip (\`parse(to_string(v)) == v\`) не изменился.`,
    zh: `  Round-trip(\`parse(to_string(v)) == v\`)未受影响。`,
  },
  {
    id: 'v0-2-0-008',
    join: 'block',
    en: `      // Before (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // After (0.2.0):
      // body: (
      //     line1
      //     line2
      // )`,
    ru: `      // Было (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // Стало (0.2.0):
      // body: (
      //     line1
      //     line2
      // )`,
    zh: `      // 之前 (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // 之后 (0.2.0):
      // body: (
      //     line1
      //     line2
      // )`,
  },
  {
    id: 'v0-2-0-009',
    join: 'block',
    en: `  Both \`Value\` → \`render::render(&value)\` and \`T: Serialize\` →
  \`ser::to_string(&t)\` paths are updated consistently.`,
    ru: `  Оба пути сериализации — \`Value\` → \`render::render(&value)\` и
  \`T: Serialize\` → \`ser::to_string(&t)\` — обновлены консистентно.`,
    zh: `  序列化双路径(\`Value\` → \`render::render(&value)\` 与
  \`T: Serialize\` → \`ser::to_string(&t)\`)同步更新,行为一致。`,
  },
  {
    id: 'v0-2-0-010',
    join: 'block',
    en: `- **Typed-float marker \`:f\` now accepts integer literals.** The mantissa's
  decimal point is **optional**: \`:f 42\` is valid (parsed as \`42.0\`),
  matching the JSON / TOML / YAML convention that integer literals
  coerce to floats.`,
    ru: `- **Типизированный маркер \`:f\` принимает integer-литералы.** Десятичная
  точка в мантиссе теперь **опциональна**: \`:f 42\` валидно (парсится
  как \`42.0\`) — конвенция JSON / TOML / YAML, где integer литералы
  приводятся к float.`,
    zh: `- **类型标记 \`:f\` 接受整数字面量。** 尾数中的小数点现在是**可选**:
  \`:f 42\` 合法(解析为 \`42.0\`),沿用 JSON / TOML / YAML 的惯例
  (整数字面量隐式提升为 float)。`,
  },
  {
    id: 'v0-2-0-011',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`:f 1.\` (no fractional digits) and \`:f .5\` (no
  integer part) remain invalid.`,
    ru: `\`:f 1.\` (без дробной части) и \`:f .5\` (без
  целой части) по-прежнему невалидны.`,
    zh: `\`:f 1.\`(无小数部分)与 \`:f .5\`
  (无整数部分)仍然非法。`,
  },
  {
    id: 'v0-2-0-012',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Code that depends on \`:f 42\` raising
  \`InvalidTypedScalar\` needs to be updated.`,
    ru: `Код, ожидающий
  \`InvalidTypedScalar\` для \`:f 42\`, нужно обновить.`,
    zh: `依赖 \`:f 42\` 报 \`InvalidTypedScalar\` 的
  代码需要更新。`,
  },
  {
    id: 'v0-2-0-013',
    join: 'block',
    common: `### Spec`,
  },
  {
    id: 'v0-2-0-014',
    join: 'block',
    en: `- \`spec/versions/0.1/tests\` fixture \`typed_float_without_decimal\` moved
  from \`invalid/\` to \`valid/typed_float_integer_body\` to reflect the new
  semantics.`,
    ru: `- В \`spec/versions/0.1/tests\` фикстура \`typed_float_without_decimal\`
  перенесена из \`invalid/\` в \`valid/typed_float_integer_body\` под
  новую семантику.`,
    zh: `- \`spec/versions/0.1/tests\` 中的 fixture \`typed_float_without_decimal\`
  从 \`invalid/\` 移到 \`valid/typed_float_integer_body\`,以反映新语义。`,
  },
  {
    id: 'v0-2-0-015',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `Spec submodule synced.`,
    ru: `Submodule spec синхронизирован.`,
    zh: `  spec submodule 已同步。`,
  },
  {
    id: 'v0-1-5-001',
    join: 'block',
    en: `## [0.1.5] — 2026-05-01`,
    ru: `## [0.1.5] — 2026-05-01`,
    zh: `## [0.1.5] —— 2026-05-01`,
  },
  {
    id: 'v0-1-5-002',
    join: 'block',
    en: `Major release: structured errors with byte-offset spans, public
event-based parser API, \`#[non_exhaustive]\` retroactively applied to
the error enums for forward-compatibility.`,
    ru: `Большой релиз: структурированные ошибки с byte-offset spans, публичное
event-based API парсера, retroactive \`#[non_exhaustive]\` на error-enum-ах
для forward-compatibility.`,
    zh: `主要发布:带字节偏移 span 的结构化错误、公开的事件式解析器 API、
对错误枚举追溯应用 \`#[non_exhaustive]\` 以保证前向兼容。`,
  },
  {
    id: 'v0-1-5-003',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-1-5-004',
    join: 'block',
    en: `- \`ErrorKind\` enum (10 spec-defined variants + \`Other\`) with byte-offset
  \`span: Span\` on every variant, exposing \`(line, column, kind)\` directly
  to downstream consumers without regex-parsing the formatted message.`,
    ru: `- Enum \`ErrorKind\` (10 спек-определённых вариантов + \`Other\`) с
  byte-offset \`span: Span\` на каждом варианте, выставляющий
  \`(line, column, kind)\` напрямую downstream-потребителям без regex-
  парсинга форматированного сообщения.`,
    zh: `- \`ErrorKind\` 枚举(10 个规范定义变体 + \`Other\`),每个变体携带
  字节偏移 \`span: Span\`,直接向下游消费者暴露 \`(line, column, kind)\`,
  无需通过正则解析格式化消息。`,
  },
  {
    id: 'v0-1-5-005',
    join: 'block',
    common: `      pub enum ErrorKind {
          MissingSeparatorSpace { line, column, marker, span },
          InvalidTypedScalar    { line, marker, body, span },
          DuplicateKey          { line, key, span },
          KeyPathConflict       { line, path, kind: ConflictKind, span },
          EmptyKey              { line, span },
          InvalidKey            { line, key, span },
          UnclosedCompound      { kind: CompoundKind, span },
          UnbalancedBracket     { line, expected: CompoundKind, found: char, span },
          InlineNonEmptyCompound{ line, body, span },
          MissingSeparator      { line, span },
          Other                 { line: Option<u32>, message, span },
      }`,
  },
  {
    id: 'v0-1-5-006',
    join: 'block',
    en: `- \`Error::Structured(ErrorKind)\` variant on the existing \`Error\` enum.`,
    ru: `- Вариант \`Error::Structured(ErrorKind)\` на существующем enum \`Error\`.`,
    zh: `- 现有 \`Error\` 枚举上的 \`Error::Structured(ErrorKind)\` 变体。`,
  },
  {
    id: 'v0-1-5-007',
    join: 'tight',
    en: `- \`pub struct Span { start: u32, end: u32 }\` with \`Span::new\`,
  \`Span::EMPTY\`, \`slice(input)\`, and \`line_col(input)\` (1-based line,
  0-based byte column — multi-byte UTF-8 aware via tests pinning
  Cyrillic and 🦀).`,
    ru: `- \`pub struct Span { start: u32, end: u32 }\` с \`Span::new\`,
  \`Span::EMPTY\`, \`slice(input)\`, и \`line_col(input)\` (line 1-based,
  column 0-based байтовая — multi-byte UTF-8 учтён через тесты,
  пинящие кириллицу и 🦀).`,
    zh: `- \`pub struct Span { start: u32, end: u32 }\`,提供 \`Span::new\`、
  \`Span::EMPTY\`、\`slice(input)\` 与 \`line_col(input)\`(line 从 1 起,
  column 从 0 起按字节计;多字节 UTF-8 已通过西里尔与 🦀 测试固定)。`,
  },
  {
    id: 'v0-1-5-008',
    join: 'tight',
    en: `- \`Error::line() -> Option<u32>\` and \`Error::span() -> Option<Span>\`
  convenience accessors covering every variant.`,
    ru: `- \`Error::line() -> Option<u32>\` и \`Error::span() -> Option<Span>\` —
  convenience-accessors, покрывающие каждый вариант.`,
    zh: `- \`Error::line() -> Option<u32>\` 和 \`Error::span() -> Option<Span>\` —
  覆盖每个变体的便捷访问器。`,
  },
  {
    id: 'v0-1-5-009',
    join: 'tight',
    en: `- \`pub mod thin\` — public event-based parser API:
  \`ktav::parse_events(input, callback)\` invoking the supplied
  \`FnMut(ParseEvent<'_>)\` for each event borrowed from the input.`,
    ru: `- \`pub mod thin\` — публичное event-based API парсера:
  \`ktav::parse_events(input, callback)\` вызывает поставленный
  \`FnMut(ParseEvent<'_>)\` для каждого события, заимствованного из
  input.`,
    zh: `- \`pub mod thin\` —— 公开的事件式解析器 API:
  \`ktav::parse_events(input, callback)\` 对从 input 借用的每个事件
  调用所提供的 \`FnMut(ParseEvent<'_>)\`。`,
  },
  {
    id: 'v0-1-5-010',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  \`ParseEvent\` is a \`#[non_exhaustive]\` enum with 10 variants
  (\`Null\`, \`Bool\`, \`Integer\`, \`Float\`, \`Str\`, \`Key\`, \`BeginObject\`,
  \`EndObject\`, \`BeginArray\`, \`EndArray\`).`,
    ru: `\`ParseEvent\` — \`#[non_exhaustive]\` enum с 10 вариантами
  (\`Null\`, \`Bool\`, \`Integer\`, \`Float\`, \`Str\`, \`Key\`, \`BeginObject\`,
  \`EndObject\`, \`BeginArray\`, \`EndArray\`).`,
    zh: `\`ParseEvent\` 是
  \`#[non_exhaustive]\` 枚举,有 10 个变体。`,
  },
  {
    id: 'v0-1-5-011',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The internal bumpalo arena
  stays private — the public API does not leak the arena type.`,
    ru: `Внутренняя bumpalo arena
  остаётся приватной — публичный API не утекает тип арены.`,
    zh: `内部 bumpalo arena
  保持私有 —— 公共 API 不泄露 arena 类型。`,
  },
  {
    id: 'v0-1-5-012',
    join: 'tight',
    en: `- Crate-level runnable doctest in \`src/lib.rs\` demonstrating both
  \`Error::Structured\` matching with \`Span::slice\` and the
  \`parse_events\` callback shape.`,
    ru: `- Crate-level runnable doctest в \`src/lib.rs\` демонстрирует и
  matching \`Error::Structured\` со \`Span::slice\`, и форму
  \`parse_events\` callback.`,
    zh: `- \`src/lib.rs\` 的 crate 级可运行 doctest,演示 \`Error::Structured\`
  匹配配合 \`Span::slice\` 以及 \`parse_events\` 回调形态。`,
  },
  {
    id: 'v0-1-5-013',
    join: 'tight',
    en: `- Six new top-level test files:
  \`tests/error_format.rs\` — Display-string regression net (canonical
  pinning for the 7 categories that LSP / bindings rely on);
  \`tests/structured_errors.rs\` — variant identity + (line, span) byte
  ranges per spec invalid fixture;
  \`tests/error_spans.rs\` — span byte-range semantics + \`Span::slice\`
  and \`Span::line_col\` edge cases (UTF-8 multi-byte, char-boundary
  rounding);
  \`tests/error_accessors.rs\` — every \`Error\` variant tested for
  \`line()\` / \`span()\` returning \`Some\` / \`None\` as documented;
  \`tests/non_exhaustive.rs\` — wildcard-arm reachability proof for
  \`Error\` and \`ErrorKind\`;
  \`tests/thin_public.rs\` — event sequencing, nested compounds, marker
  items, error propagation, borrow contract.`,
    ru: `- Шесть новых top-level test-файлов:
  \`tests/error_format.rs\` — регрессионная сеть по Display-строкам
  (канонический пиннинг 7 категорий, на которые опираются LSP и
  биндинги);
  \`tests/structured_errors.rs\` — идентичность варианта плюс диапазоны
  (line, span) в байтах на каждую невалидную фикстуру спеки;
  \`tests/error_spans.rs\` — семантика байтовых диапазонов span плюс
  краевые случаи \`Span::slice\` и \`Span::line_col\` (многобайтовый UTF-8,
  округление до границы символа);
  \`tests/error_accessors.rs\` — каждый вариант \`Error\` проверен на то,
  что \`line()\` / \`span()\` возвращают \`Some\` / \`None\` как задокументировано;
  \`tests/non_exhaustive.rs\` — доказательство достижимости
  wildcard-ветки для \`Error\` и \`ErrorKind\`;
  \`tests/thin_public.rs\` — последовательность событий, вложенные
  составные, marker-элементы, распространение ошибок, контракт
  заимствования.`,
    zh: `- 六个新顶层测试文件:
  \`tests/error_format.rs\` —— Display 字符串的回归网(为 LSP 与绑定所
  依赖的 7 个类别做规范化钉定);
  \`tests/structured_errors.rs\` —— 按规范的每个无效 fixture 校验变体
  身份以及 (line, span) 字节范围;
  \`tests/error_spans.rs\` —— span 字节范围语义,以及 \`Span::slice\` 与
  \`Span::line_col\` 的边界情形(多字节 UTF-8、按字符边界取整);
  \`tests/error_accessors.rs\` —— 逐一验证每个 \`Error\` 变体的
  \`line()\` / \`span()\` 是否如文档所述返回 \`Some\` / \`None\`;
  \`tests/non_exhaustive.rs\` —— \`Error\` 与 \`ErrorKind\` 的 wildcard 分支
  可达性证明;
  \`tests/thin_public.rs\` —— 事件序列、嵌套复合、marker 元素、错误传播、
  借用契约。`,
  },
  {
    id: 'v0-1-5-014',
    join: 'tight',
    en: `- Synthetic Criterion benchmarks under \`benches/\` covering parse
  perf at small_1k / medium_50k / large_500k workloads on both
  success and error paths.`,
    ru: `- Синтетические Criterion-бенчи под \`benches/\` покрывающие parse-perf
  на small_1k / medium_50k / large_500k workloads, на success и error
  путях.`,
    zh: `- \`benches/\` 下的合成 Criterion 基准,覆盖 small_1k / medium_50k /
  large_500k 负载在成功路径和错误路径上的解析性能。`,
  },
  {
    id: 'v0-1-5-015',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Baseline numbers in \`bench-baseline.md\`.`,
    ru: `Baseline-числа в \`bench-baseline.md\`.`,
    zh: `baseline 数字
  位于 \`bench-baseline.md\`。`,
  },
  {
    id: 'v0-1-5-016',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-1-5-017',
    join: 'block',
    en: `- \`#[non_exhaustive]\` retroactively applied to \`Error\`, \`ErrorKind\`,
  \`ConflictKind\`, and \`CompoundKind\`.`,
    ru: `- \`#[non_exhaustive]\` retroactively применён к \`Error\`, \`ErrorKind\`,
  \`ConflictKind\` и \`CompoundKind\`.`,
    zh: `- 对 \`Error\`、\`ErrorKind\`、\`ConflictKind\`、\`CompoundKind\` 追溯应用
  \`#[non_exhaustive]\`。`,
  },
  {
    id: 'v0-1-5-018',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Future variant additions are
  no longer breaking changes for downstream \`match\`-ers, who must
  now include a \`_ =>\` arm.`,
    ru: `Будущие добавления вариантов
  больше не являются ломающим изменением для downstream-\`match\`-еров,
  которые теперь обязаны иметь arm \`_ =>\`.`,
    zh: `未来添加变体对下游的 \`match\` 不再是破坏性
  变更 —— 调用方现在必须包含 \`_ =>\` 分支。`,
  },
  {
    id: 'v0-1-5-019',
    join: 'tight',
    en: `- The parser no longer constructs \`Error::Syntax(format!(...))\` at any
  internal call site (~37 sites refactored to \`Error::Structured\`).`,
    ru: `- Парсер больше не конструирует \`Error::Syntax(format!(...))\` ни на
  одном внутреннем call-сайте (~37 сайтов перерефакторены на
  \`Error::Structured\`).`,
    zh: `- 解析器在任何内部调用点都不再构造 \`Error::Syntax(format!(...))\`
  (约 37 个站点重构为 \`Error::Structured\`)。`,
  },
  {
    id: 'v0-1-5-020',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  A regression guard test
  (\`parser_no_longer_emits_legacy_syntax_variant\`) runs 12 invalid
  inputs and fails CI loudly if anyone reintroduces the legacy
  variant inside \`src/\`.`,
    ru: `Регрессионный guard-test
  (\`parser_no_longer_emits_legacy_syntax_variant\`) гоняет 12
  невалидных входов и громко фейлит CI, если кто-то реинтродуктит
  legacy-вариант внутри \`src/\`.`,
    zh: `回归保护测试
  (\`parser_no_longer_emits_legacy_syntax_variant\`)运行 12 个非法
  输入,若有人在 \`src/\` 内重新引入旧变体,CI 会大声失败。`,
  },
  {
    id: 'v0-1-5-021',
    join: 'tight',
    en: `- \`parser/parse_str.rs\` replaces \`str::lines()\` with a manual
  byte-walking loop maintaining a cumulative \`line_start\` counter
  so byte-offset spans can be computed at every error site without
  rescanning.`,
    ru: `- \`parser/parse_str.rs\` заменяет \`str::lines()\` на ручной
  byte-walking loop, поддерживающий cumulative-\`line_start\` счётчик —
  byte-offset spans вычисляются на каждом error-сайте без
  пересканирования.`,
    zh: `- \`parser/parse_str.rs\` 用维护累积 \`line_start\` 计数的手工字节遍历
  循环替代 \`str::lines()\`,以便在每个错误站点计算字节偏移 span 而
  无需重新扫描。`,
  },
  {
    id: 'v0-1-5-022',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`thin/event_parser.rs\` mirrors the same plumbing on
  the zero-copy path.`,
    ru: `\`thin/event_parser.rs\` зеркалит то же plumbing
  на zero-copy пути.`,
    zh: `\`thin/event_parser.rs\` 在零拷贝路径上镜像同一
  plumbing。`,
  },
  {
    id: 'v0-1-5-023',
    join: 'tight',
    en: `- \`Display for ErrorKind\` is byte-identical to the strings the parser
  previously formatted into \`Error::Syntax(...)\` for the seven
  pre-existing categories — the contract that lets every existing
  string-based caller keep working unmodified during the
  ecosystem-wide migration tracked in
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md).`,
    ru: `- \`Display for ErrorKind\` byte-identical к строкам, которые парсер
  ранее форматировал в \`Error::Syntax(...)\` для семи pre-existing
  категорий — это контракт, позволяющий каждому существующему
  string-based caller-у работать без изменений на время
  ecosystem-wide миграции, описанной в
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md).`,
    zh: `- \`Display for ErrorKind\` 与解析器先前格式化进 \`Error::Syntax(...)\`
  的字符串完全字节相同(覆盖七个先前已固定类别)—— 这是让现有
  基于字符串的调用方在
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md) 描述的生态系统
  级迁移期间保持不变工作的契约。`,
  },
  {
    id: 'v0-1-5-024',
    join: 'tight',
    en: `- Three formerly-\`Other\` shapes promoted to named \`ErrorKind\` variants:
  \`UnbalancedBracket\` (stray closer / shape mismatch),
  \`InlineNonEmptyCompound\` (\`x: {foo}\` — spec § 6.7),
  \`MissingSeparator\` (line with no \`:\`).`,
    ru: `- Три прежде-\`Other\` формы промоутированы в именованные варианты
  \`ErrorKind\`: \`UnbalancedBracket\` (lone closer / shape mismatch),
  \`InlineNonEmptyCompound\` (\`x: {foo}\` — спека § 6.7),
  \`MissingSeparator\` (строка без \`:\`).`,
    zh: `- 三个先前归入 \`Other\` 的形态升级为命名 \`ErrorKind\` 变体:
  \`UnbalancedBracket\`(孤立闭符 / 形状不匹配)、
  \`InlineNonEmptyCompound\`(\`x: {foo}\` —— 规范 § 6.7)、
  \`MissingSeparator\`(无 \`:\` 的行)。`,
  },
  {
    id: 'v0-1-5-025',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `After this promotion, \`Other\`
  contains only parser-internal invariants that no spec invalid
  fixture can trigger.`,
    ru: `После promotion-а \`Other\`
  содержит только парсер-внутренние invariants, которые ни одна
  spec invalid фикстура не триггерит.`,
    zh: `升级后,\`Other\` 仅保留没有任何
  规范非法夹具能触发的解析器内部不变量。`,
  },
  {
    id: 'v0-1-5-026',
    join: 'block',
    en: `### Performance`,
    ru: `### Производительность`,
    zh: `### 性能`,
  },
  {
    id: 'v0-1-5-027',
    join: 'block',
    en: `\`cargo bench --bench parse -- --quick\` against the 0.1.4 baseline:`,
    ru: `\`cargo bench --bench parse -- --quick\` против baseline 0.1.4:`,
    zh: `\`cargo bench --bench parse -- --quick\` 与 0.1.4 baseline 对比:`,
  },
  {
    id: 'v0-1-5-028',
    join: 'block',
    common: `|                                | 0.1.4 baseline | 0.1.5  | Δ      |
|--------------------------------|----------------|--------|--------|
| \`parse_synth/small_1k\`         | 16.1 µs        | 16.0 µs| −0.6 % |
| \`parse_synth/medium_50k\`       | 896 µs         | 663 µs | −26 %  |
| \`parse_synth/large_500k\`       | 9.49 ms        | 9.27 ms| −2.3 % |
| \`parse_synth_error/small_1k\`   | 7.5 µs         | 7.2 µs | −4.0 % |
| \`parse_synth_error/medium_50k\` | 340 µs         | 346 µs | +1.8 % |
| \`parse_synth_error/large_500k\` | 4.47 ms        | 4.50 ms| +0.7 % |`,
  },
  {
    id: 'v0-1-5-029',
    join: 'block',
    en: `Net: zero success-path regression.`,
    ru: `Итог: регрессии на success-path нет.`,
    zh: `结论:成功路径零回归。`,
  },
  {
    id: 'v0-1-5-030',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Error-path slightly faster — the
new \`Display\` impl constructs the formatted string lazily at
\`.to_string()\` time, whereas the prior \`format!(...)\` allocated a
\`String\` at every error site eagerly.`,
    ru: `Error-path немного быстрее — новая
реализация \`Display\` собирает форматированную строку лениво, в момент
\`.to_string()\`, тогда как прежний \`format!(...)\` выделял \`String\` жадно
на каждой точке ошибки.`,
    zh: `错误路径略快——新的 \`Display\` 实现在调用
\`.to_string()\` 时才惰性构造格式化字符串,而此前的 \`format!(...)\` 会在
每一处错误点即时分配一个 \`String\`。`,
  },
  {
    id: 'v0-1-5-031',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The cumulative-byte counter
that powers spans is statistically free.`,
    ru: `Счётчик накопленных байтов, на котором держатся
spans, статистически бесплатен.`,
    zh: `支撑 span 的累积字节计数器在统计上
是免费的。`,
  },
  {
    id: 'v0-1-5-032',
    join: 'block',
    en: `### Notes`,
    ru: `### Заметки`,
    zh: `### 备注`,
  },
  {
    id: 'v0-1-5-033',
    join: 'block',
    en: `- \`Error::Syntax(String)\` is preserved for backward compatibility —
  the public API stays deny-no-old-callers.`,
    ru: `- \`Error::Syntax(String)\` сохранён для обратной совместимости —
  публичный API остаётся deny-no-old-callers.`,
    zh: `- 为了向后兼容,\`Error::Syntax(String)\` 保留 —— 公共 API 仍不拒绝
  老调用方。`,
  },
  {
    id: 'v0-1-5-034',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Removal is deferred to
  ktav 1.0.`,
    ru: `Удаление отложено до
  ktav 1.0.`,
    zh: `移除推迟到 ktav 1.0。`,
  },
  {
    id: 'v0-1-5-035',
    join: 'tight',
    en: `- Test count: 332 (0.1.4) → 391 (+59) plus 1 new doctest.`,
    ru: `- Тестов: 332 (0.1.4) → 391 (+59) плюс 1 новый doctest.`,
    zh: `- 测试数:332 (0.1.4) → 391 (+59),外加 1 个新 doctest。`,
  },
  {
    id: 'v0-1-5-036',
    join: 'tight',
    en: `- The cabi/binding migration to consume \`ErrorKind\` over the FFI
  boundary is tracked separately in
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md) and ships as a
  coordinated ecosystem 0.2.0.`,
    ru: `- Миграция cabi/биндингов на потребление \`ErrorKind\` через FFI
  трекается отдельно в
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md) и идёт как
  coordinated ecosystem 0.2.0.`,
    zh: `- cabi/绑定迁移以通过 FFI 边界消费 \`ErrorKind\` 的工作单独记录在
  [\`STRUCTURED_ERRORS.md\`](../STRUCTURED_ERRORS.md),并作为协调发布
  的生态系统 0.2.0 一并交付。`,
  },
  {
    id: 'v0-1-5-037',
    join: 'block',
    en: `### SemVer note`,
    ru: `### SemVer-замечание`,
    zh: `### SemVer 说明`,
  },
  {
    id: 'v0-1-5-038',
    join: 'block',
    en: `Adding \`#[non_exhaustive]\` to a previously-unmarked enum (\`Error\`,
\`ConflictKind\`, \`CompoundKind\`) is, per the
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
a breaking change that would normally require a major bump (0.2.0).`,
    ru: `Добавление \`#[non_exhaustive]\` к ранее непомеченным enum-ам (\`Error\`,
\`ConflictKind\`, \`CompoundKind\`) — согласно
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive)
— breaking change, требующий major-bump-а (0.2.0).`,
    zh: `按
[Cargo SemVer 参考](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
为先前未标注的枚举(\`Error\`、\`ConflictKind\`、\`CompoundKind\`)添加
\`#[non_exhaustive]\` 是破坏性变更,通常需要主版本号 bump(0.2.0)。`,
  },
  {
    id: 'v0-1-5-039',
    join: { en: 'tight', ru: 'flow', zh: 'tight' },
    en: `This release ships as **0.1.5** intentionally:`,
    ru: `Этот релиз
выпускается как **0.1.5** намеренно:`,
    zh: `本次发布有意作为 **0.1.5** 推出:`,
  },
  {
    id: 'v0-1-5-040',
    join: 'block',
    en: `1. Pre-1.0 Cargo convention permits breaking changes on any bump,
   including patches.`,
    ru: `1. Pre-1.0 Cargo-конвенция допускает breaking-изменения на любом
   bump-е, включая патчи.`,
    zh: `1. Pre-1.0 的 Cargo 惯例允许在任何 bump(包括 patch)上做破坏性
   变更。`,
  },
  {
    id: 'v0-1-5-041',
    join: 'tight',
    en: `2. All known downstream consumers of \`ktav::Error\` (the six language
   bindings under \`ktav-lang/\`) call \`Err(e) => e.to_string()\` only.`,
    ru: `2. Все известные downstream-потребители \`ktav::Error\` (шесть
   языковых биндингов под \`ktav-lang/\`) делают только
   \`Err(e) => e.to_string()\`.`,
    zh: `2. \`ktav::Error\` 所有已知的下游消费者(\`ktav-lang/\` 下的六个语言
   绑定)均仅调用 \`Err(e) => e.to_string()\`。`,
  },
  {
    id: 'v0-1-5-042',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `   No exhaustive \`match err { Error::Io(_) => …, Error::Syntax(_)
   => …, Error::Message(_) => … }\` patterns exist in the ecosystem
   that this change would silently break.`,
    ru: `Exhaustive \`match err { Error::Io(_)
   => …, Error::Syntax(_) => …, Error::Message(_) => … }\` в
   экосистеме нет — ломать тихо нечего.`,
    zh: `生态系统中不存在会被
   该变更悄悄破坏的穷尽 \`match err { Error::Io(_) => …,
   Error::Syntax(_) => …, Error::Message(_) => … }\` 模式。`,
  },
  {
    id: 'v0-1-5-043',
    join: 'tight',
    en: `3. The seven canonical-category Display strings remain byte-identical
   to 0.1.4, so any hypothetical out-of-tree consumer doing string
   matching keeps working unmodified.`,
    ru: `3. Display-строки семи канонических категорий byte-identical к
   0.1.4, поэтому любой гипотетический out-of-tree-потребитель,
   делающий string matching, продолжает работать без изменений.`,
    zh: `3. 七个标准类别的 Display 字符串与 0.1.4 字节相同,因此任何在
   tree 之外做字符串匹配的假想消费者也无需修改即可继续工作。`,
  },
  {
    id: 'v0-1-5-044',
    join: 'block',
    en: `If your code does keep an exhaustive match over \`ktav::Error\` and
this release breaks it, add an \`_ => …\` arm.`,
    ru: `Если ваш код всё-таки держит exhaustive \`match\` по \`ktav::Error\` и
этот релиз его ломает — добавьте arm \`_ => …\`.`,
    zh: `如果你的代码确实保留了对 \`ktav::Error\` 的穷尽 \`match\` 而本次发布
将其破坏 —— 请添加 \`_ => …\` 分支。`,
  },
  {
    id: 'v0-1-5-045',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `That arm is now
required forever and will not need to change again as future
variants are added.`,
    ru: `Этот arm теперь
обязателен навсегда и больше не потребует изменений при добавлении
будущих вариантов.`,
    zh: `该分支从此永久必须存在,且不会
随未来变体的添加而再次需要变更。`,
  },
  {
    id: 'v0-1-4-001',
    join: 'block',
    en: `## [0.1.4] — 2026-04-26`,
    ru: `## [0.1.4] — 2026-04-26`,
    zh: `## [0.1.4] —— 2026-04-26`,
  },
  {
    id: 'v0-1-4-002',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-1-4-003',
    join: 'block',
    en: `- **\`Frame::Object\` initial capacity 4 → 8** (\`src/parser/frame.rs\`).`,
    ru: `- **\`Frame::Object\` initial capacity 4 → 8** (\`src/parser/frame.rs\`).`,
    zh: `- **\`Frame::Object\` 初始容量 4 → 8**(\`src/parser/frame.rs\`)。`,
  },
  {
    id: 'v0-1-4-004',
    join: 'tight',
    en: `  The parser's per-compound \`IndexMap\` now pre-sizes for 8 entries
  instead of 4, which eliminates the first growth/rehash for the
  typical 5–8-field config row.`,
    ru: `  Per-compound \`IndexMap\` парсера теперь pre-sizes под 8 элементов
  вместо 4 — устраняется первый growth/rehash для типичной строки
  конфига (5–8 полей).`,
    zh: `  解析器的 per-compound \`IndexMap\` 现在预分配 8 个槽位而非 4 个,
  消除了典型配置行(5–8 字段)的首次扩容/rehash。`,
  },
  {
    id: 'v0-1-4-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `This is the **untyped** parse path
  (\`ktav::parse → Value\`) — the same path every C-ABI binding
  (PHP/JS/Python/Go/Java/C#) walks through \`cabi\`, so they all see
  the speedup once they pick up 0.1.4.`,
    ru: `Это **untyped** парсинг-путь
  (\`ktav::parse → Value\`) — тот самый путь, через который идут все
  C-ABI биндинги (PHP/JS/Python/Go/Java/C#) через \`cabi\`, поэтому
  они **получат** ускорение, как только подхватят 0.1.4.`,
    zh: `这是 **untyped**
  解析路径(\`ktav::parse → Value\`)—— 也是所有 C-ABI 绑定
  (PHP/JS/Python/Go/Java/C#)通过 \`cabi\` 走的路径,因此一旦它们
  升级到 0.1.4 就会获得相同的加速。`,
  },
  {
    id: 'v0-1-4-006',
    join: 'tight',
    en: `- Net impact on the \`parse_to_value\` bench (3-run median): small
  **−30%** (18.9 µs → 13.3 µs), large **−13%** (5.04 ms → 4.4 ms),
  medium in the noise (~−3%).`,
    ru: `- Эффект на бенче \`parse_to_value\` (медиана 3 прогонов): small
  **−30%** (18.9 µs → 13.3 µs), large **−13%** (5.04 ms → 4.4 ms),
  medium в шуме (~−3%).`,
    zh: `- 在 \`parse_to_value\` 基准上的影响(3 次运行中位数):small
  **−30%**(18.9 µs → 13.3 µs)、large **−13%**(5.04 ms → 4.4 ms)、
  medium 在噪声范围内(~−3%)。`,
  },
  {
    id: 'v0-1-4-007',
    join: 'block',
    en: `One-line change; full test suite (334 cases incl. spec conformance)
unaffected.`,
    ru: `Изменение в одну строку; полный набор тестов (334 кейса вкл.
spec conformance) не затронут.`,
    zh: `单行改动;完整测试套件(334 个用例,含 spec conformance)不受影响。`,
  },
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
  {
    id: 'v0-1-1-001',
    join: 'block',
    en: `## [0.1.1] — 2026-04-26`,
    ru: `## [0.1.1] — 2026-04-26`,
    zh: `## [0.1.1] —— 2026-04-26`,
  },
  {
    id: 'v0-1-1-002',
    join: 'block',
    en: `### Changed`,
    ru: `### Изменено`,
    zh: `### 变更`,
  },
  {
    id: 'v0-1-1-003',
    join: 'block',
    en: `- **Typed-deserialization fast path** — \`from_str\` and \`from_file\` no
  longer build a \`ThinValue\` tree as an intermediate.`,
    ru: `- **Быстрый путь типизированной десериализации** — \`from_str\` и
  \`from_file\` больше не строят промежуточное дерево \`ThinValue\`.`,
    zh: `- **类型化反序列化快路径** —— \`from_str\` 与 \`from_file\` 不再构建中间
  \`ThinValue\` 树。`,
  },
  {
    id: 'v0-1-1-004',
    join: { en: 'flow', ru: 'tight', zh: 'none' },
    en: `The parser now
  emits a flat \`Vec<Event>\` directly into a bump arena, and the serde
  deserializer walks it linearly with a single cursor — one allocation
  per document instead of one per compound, and no per-node enum-
  discriminant load behind a \`Box\`-style indirection.`,
    ru: `  Парсер сразу эмитит плоский \`Vec<Event>\` в bump-арену, а serde-
  десериализатор линейно идёт по нему одним курсором — одна
  аллокация на документ вместо одной на компаунд, без подгрузки
  enum-discriminant'а через косвенность.`,
    zh: `解析器直接将事件序列(\`Vec<Event>\`)发射到 bump
  arena,serde 反序列化器以单一游标线性遍历它 —— 每个文档一次分配
  而非每个复合节点一次,且无需通过 \`Box\` 间接加载枚举判别式。`,
  },
  {
    id: 'v0-1-1-005',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `Net impact on a
  275 KB config: **−18.7%** on \`parse → struct\` (3.60 ms → 2.93 ms).`,
    ru: `На 275 KB конфиге:
  **−18.7%** на \`parse → struct\` (3.60 ms → 2.93 ms).`,
    zh: `  在 275 KB 配置上的实测:\`parse → struct\` **−18.7%**(3.60 ms →
  2.93 ms)。`,
  },
  {
    id: 'v0-1-1-006',
    join: 'tight',
    en: `- **\`fast_num\` byte-loop atoi** — the \`i8\`..\`i64\` / \`u8\`..\`u64\` paths
  in the typed deserializer skip the generic \`<T as FromStr>\` route
  and call hand-rolled \`parse_i64\` / \`parse_u64\` with a width check.`,
    ru: `- **\`fast_num\` byte-loop atoi** — пути \`i8\`..\`i64\` / \`u8\`..\`u64\` в
  типизированном десериализаторе обходят generic-маршрут через
  \`<T as FromStr>\` и используют ручные \`parse_i64\` / \`parse_u64\`
  с проверкой ширины.`,
    zh: `- **\`fast_num\` 字节循环 atoi** —— 类型化反序列化器中的 \`i8\`..\`i64\`
  / \`u8\`..\`u64\` 路径绕过通用的 \`<T as FromStr>\` 路线,改为调用手写
  的 \`parse_i64\` / \`parse_u64\` 并附带宽度检查。`,
  },
  {
    id: 'v0-1-1-007',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  Floats stay on \`f64::from_str\`.`,
    ru: `Float-пути остаются на \`f64::from_str\`.`,
    zh: `浮点路径仍走
  \`f64::from_str\`。`,
  },
  {
    id: 'v0-1-1-008',
    join: 'block',
    en: `### Added`,
    ru: `### Добавлено`,
    zh: `### 新增`,
  },
  {
    id: 'v0-1-1-009',
    join: 'block',
    en: `- \`Event\` token enum and \`EventCursor\` walker (\`thin/event*.rs\`),
  internal — not exposed in the public surface.`,
    ru: `- Внутренние \`Event\` enum и \`EventCursor\` walker (\`thin/event*.rs\`).`,
    zh: `- 内部 \`Event\` 枚举与 \`EventCursor\` 遍历器(\`thin/event*.rs\`)。`,
  },
  {
    id: 'v0-1-1-010',
    join: 'block',
    en: `### Removed`,
    ru: `### Удалено`,
    zh: `### 移除`,
  },
  {
    id: 'v0-1-1-011',
    join: 'block',
    en: `- \`ThinValue\` enum and its \`ThinDeserializer\` (replaced by the event
  stream — both were \`pub(crate)\`, so no breakage at the public API).`,
    ru: `- Enum \`ThinValue\` и его \`ThinDeserializer\` (заменены event-stream'ом
  — оба были \`pub(crate)\`, публичный API не сломан).`,
    zh: `- \`ThinValue\` 枚举及其 \`ThinDeserializer\`(已被事件流取代;两者均为
  \`pub(crate)\`,公开 API 不受影响)。`,
  },
  {
    id: 'v0-1-1-012',
    join: 'block',
    en: `### Behavior change`,
    ru: `### Изменение поведения`,
    zh: `### 行为变化`,
  },
  {
    id: 'v0-1-1-013',
    join: 'block',
    en: `- **Interleaved dotted-key prefixes are now rejected as a conflict**.
  A document like \`a.x: 1\\nb.y: 2\\na.z: 3\` (synthetic \`a\` opened, then
  closed by \`b.\`, then re-opened by \`a.z\`) used to silently merge into
  one \`a\` object via the tree-builder. The event-stream tokenizer
  cannot do that without buffering the whole document, so it now
  surfaces a clear conflict error suggesting the user group lines with
  the same prefix together. Documents with grouped dotted keys (the
  canonical pattern) are unaffected — every spec-conformance fixture
  still passes.`,
    ru: `- **Чередование dotted-key префиксов теперь отклоняется как
  conflict.** Документ вида \`a.x: 1\\nb.y: 2\\na.z: 3\` (synthetic \`a\`
  открыт, закрыт через \`b.\`, попытка переоткрыть через \`a.z\`)
  раньше тихо сливался в один объект \`a\` через tree-builder.
  Event-stream-токенизатор не может это сделать без буферизации
  всего документа, поэтому теперь возвращает понятный conflict-
  ошибку с предложением сгруппировать строки с одним префиксом
  вместе. Документы со сгруппированными dotted-ключами (канонический
  паттерн) не затронуты — все spec-conformance фикстуры зелёные.`,
    zh: `- **dotted-key 前缀的交错使用现在会被拒绝为 conflict。** 形如
  \`a.x: 1\\nb.y: 2\\na.z: 3\` 的文档(合成对象 \`a\` 被打开,被 \`b.\`
  关闭,然后被 \`a.z\` 尝试重新打开)以前会通过 tree-builder 静默
  合并为单一 \`a\` 对象。事件流标记器在不缓冲整个文档的情况下无法
  做到这一点,因此现在会返回清晰的 conflict 错误,提示用户将相同
  前缀的行分组在一起。使用分组 dotted-key(规范模式)的文档不受
  影响 —— 所有 spec-conformance 用例仍然通过。`,
  },
  {
    id: 'v0-1-0-001',
    join: 'block',
    en: `## [0.1.0] — 2026-04-22`,
    ru: `## [0.1.0] — 2026-04-22`,
    zh: `## [0.1.0] —— 2026-04-22`,
  },
  {
    id: 'v0-1-0-002',
    join: 'block',
    en: `Initial release.`,
    ru: `Первый релиз.`,
    zh: `首次发布。`,
  },
  {
    id: 'v0-1-0-003',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Implements [Ktav spec 0.1.0](https://github.com/ktav-lang/spec/blob/main/versions/0.1/spec.md).`,
    ru: `Реализует [Ktav spec 0.1.0](https://github.com/ktav-lang/spec/blob/main/versions/0.1/spec.md).`,
    zh: `实现 [Ktav spec 0.1.0](https://github.com/ktav-lang/spec/blob/main/versions/0.1/spec.md)。`,
  },
  {
    id: 'v0-1-0-004',
    join: 'block',
    common: `### Added`,
  },
  {
    id: 'v0-1-0-005',
    join: 'block',
    en: `- **Parser** — turns Ktav text into a \`Value\` (owned) or a \`ThinValue\`
  (zero-copy view over the input buffer).`,
    ru: `- **Parser** — превращает текст Ktav в \`Value\` (владеющий) или
  \`ThinValue\` (zero-copy view поверх входного буфера).`,
    zh: `- **Parser** —— 将 Ktav 文本转换为 \`Value\`(拥有所有权)或
  \`ThinValue\`(在输入缓冲区上的零拷贝视图)。`,
  },
  {
    id: 'v0-1-0-006',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Line-based state machine
  with dotted-key expansion, multi-line strings (stripped and
  verbatim), JSON-style keywords \`null\` / \`true\` / \`false\`, and
  typed-scalar markers \`:i\` (Integer) and \`:f\` (Float).`,
    ru: `Построчная
  state machine с разворачиванием точечных ключей, многострочными
  строками (со снятием отступа и побайтовыми), JSON-подобными
  ключевыми словами \`null\` / \`true\` / \`false\` и типизированными
  скалярными маркерами \`:i\` (Integer) и \`:f\` (Float).`,
    zh: `基于行的状态机,
  支持点分键展开、多行字符串(剥除缩进与逐字节两种)、
  JSON 风格关键字 \`null\` / \`true\` / \`false\`,以及类型化标量标记
  \`:i\`(Integer)与 \`:f\`(Float)。`,
  },
  {
    id: 'v0-1-0-007',
    join: 'tight',
    en: `- **Serializer** — two paths:
  - \`ktav::to_string\` (direct text emission, primary path).
  - \`ktav::ser::to_value\` / \`ktav::render\` (two-step for users who
    want to inspect a \`Value\` between stages).`,
    ru: `- **Serializer** — два пути:
  - \`ktav::to_string\` (прямая эмиссия текста, основной путь).
  - \`ktav::ser::to_value\` / \`ktav::render\` (двухшаговый вариант для
    тех, кто хочет осмотреть \`Value\` между стадиями).`,
    zh: `- **Serializer** —— 两条路径:
  - \`ktav::to_string\`(直接文本输出,主路径)。
  - \`ktav::ser::to_value\` / \`ktav::render\`(两步路径,便于在中间
    检视 \`Value\`)。`,
  },
  {
    id: 'v0-1-0-008',
    join: 'tight',
    en: `  Both emit \`::\` automatically for strings that would otherwise be
  mis-read by the parser, and emit \`:i\` / \`:f\` for Rust numeric
  types.`,
    ru: `  Оба автоматически эмитят \`::\` для строк, которые иначе были бы
  неверно прочитаны парсером, и эмитят \`:i\` / \`:f\` для числовых
  Rust-типов.`,
    zh: `  两者都会在字符串可能被解析器误读时自动输出 \`::\`,并为 Rust
  数值类型发出 \`:i\` / \`:f\`。`,
  },
  {
    id: 'v0-1-0-009',
    join: 'tight',
    en: `- **Deserializer** — zero-copy path via \`ThinValue<'a>\` and
  \`ThinDeserializer\`.`,
    ru: `- **Deserializer** — zero-copy путь через \`ThinValue<'a>\` и
  \`ThinDeserializer\`.`,
    zh: `- **Deserializer** —— 通过 \`ThinValue<'a>\` 与 \`ThinDeserializer\`
  走零拷贝路径。`,
  },
  {
    id: 'v0-1-0-010',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Object keys and single-line scalar values are
  borrowed directly from the input; only multi-line strings allocate.`,
    ru: `Ключи объектов и однострочные скалярные
  значения заимствуются напрямую из входа; выделение памяти
  происходит только для многострочных строк.`,
    zh: `对象键与单行标量值直接从输入借用;只有多行
  字符串会发生分配。`,
  },
  {
    id: 'v0-1-0-011',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  Accepts both typed-marker and plain-string forms of numbers, so
  documents written without markers deserialize transparently via
  \`FromStr\`.`,
    ru: `Принимает обе формы
  чисел — с маркером и без: документы, написанные без маркеров,
  десериализуются прозрачно через \`FromStr\`.`,
    zh: `接受带标记与不带标记两种数字形式 —— 不含
  标记的旧文档仍能通过 \`FromStr\` 透明反序列化。`,
  },
  {
    id: 'v0-1-0-012',
    join: 'tight',
    en: `- **Serde integration** — \`from_str\`, \`from_file\`, \`to_string\`,
  \`to_file\` accept any \`T: Serialize\` / \`DeserializeOwned\`, including
  \`#[derive]\`-generated types, nested structs, \`Vec\`, \`Option\`,
  \`HashMap\`, and the usual externally-tagged enum forms.`,
    ru: `- **Serde integration** — \`from_str\`, \`from_file\`, \`to_string\`,
  \`to_file\` принимают любой \`T: Serialize\` / \`DeserializeOwned\`,
  включая типы, сгенерированные \`#[derive]\`, вложенные struct-ы,
  \`Vec\`, \`Option\`, \`HashMap\` и стандартные externally-tagged формы
  enum-ов.`,
    zh: `- **Serde integration** —— \`from_str\`、\`from_file\`、\`to_string\`、
  \`to_file\` 接受任何 \`T: Serialize\` / \`DeserializeOwned\`,包括
  \`#[derive]\` 生成的类型、嵌套结构体、\`Vec\`、\`Option\`、\`HashMap\`
  以及常见的 externally-tagged 枚举形式。`,
  },
  {
    id: 'v0-1-0-013',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Rust integer
  types (\`u8\`..\`u128\`, \`i8\`..\`i128\`, \`usize\`, \`isize\`) serialize with
  \`:i\`; floats (\`f32\`, \`f64\`) with \`:f\`; \`NaN\` and \`±Infinity\` are
  rejected by the serializer (not representable in Ktav 0.1.0).`,
    ru: `Целочисленные Rust-типы (\`u8\`..\`u128\`, \`i8\`..\`i128\`,
  \`usize\`, \`isize\`) сериализуются с \`:i\`; плавающие (\`f32\`, \`f64\`) —
  с \`:f\`; \`NaN\` и \`±Infinity\` отвергаются сериализатором (Ktav 0.1.0
  их не представляет).`,
    zh: `Rust 整数类型
  (\`u8\`..\`u128\`、\`i8\`..\`i128\`、\`usize\`、\`isize\`)以 \`:i\` 序列化;
  浮点(\`f32\`、\`f64\`)以 \`:f\`;\`NaN\` 与 \`±Infinity\` 被序列化器
  拒绝(Ktav 0.1.0 不表示)。`,
  },
  {
    id: 'v0-1-0-014',
    join: 'tight',
    en: `- **Raw marker \`::\`** — forces a value to be a literal String, both
  in pair position (\`key:: value\`) and as an array-item prefix
  (\`:: value\`).`,
    ru: `- **Raw-маркер \`::\`** — заставляет значение быть литеральной String,
  как в позиции пары (\`key:: value\`), так и как префикс элемента
  массива (\`:: value\`).`,
    zh: `- **Raw 标记 \`::\`** —— 强制将值视为字面量 String,既可用于键值对
  位置(\`key:: value\`),也可作为数组元素的前缀(\`:: value\`)。`,
  },
  {
    id: 'v0-1-0-015',
    join: 'tight',
    en: `- **Typed markers \`:i\` and \`:f\`** — explicit Integer / Float in pair
  position (\`port:i 8080\`, \`ratio:f 0.5\`) and as array-item prefixes
  (\`:i 42\`, \`:f 3.14\`).`,
    ru: `- **Типизированные маркеры \`:i\` и \`:f\`** — явные Integer / Float в
  позиции пары (\`port:i 8080\`, \`ratio:f 0.5\`) и как префиксы
  элементов массива (\`:i 42\`, \`:f 3.14\`).`,
    zh: `- **类型化标记 \`:i\` 与 \`:f\`** —— 在键值对位置显式声明 Integer /
  Float(\`port:i 8080\`、\`ratio:f 0.5\`),也可作为数组元素前缀
  (\`:i 42\`、\`:f 3.14\`)。`,
  },
  {
    id: 'v0-1-0-016',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Values stored as strings at the \`Value\` layer
  to preserve arbitrary precision.`,
    ru: `На уровне \`Value\` хранятся
  как строки — для сохранения произвольной точности.`,
    zh: `在 \`Value\` 层以字符串存储,以保留任意
  精度。`,
  },
  {
    id: 'v0-1-0-017',
    join: 'tight',
    en: `- **Multi-line strings** — \`( ... )\` (common-indent stripped) and
  \`(( ... ))\` (verbatim).`,
    ru: `- **Многострочные строки** — \`( ... )\` (со снятием общего отступа) и
  \`(( ... ))\` (побайтово).`,
    zh: `- **多行字符串** —— \`( ... )\`(剥除公共缩进)与 \`(( ... ))\`
  (逐字节保留)。`,
  },
  {
    id: 'v0-1-0-018',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Round-trips byte-for-byte via the verbatim
  form.`,
    ru: `Round-trip байт-в-байт через побайтовую
  форму.`,
    zh: `通过逐字节形式实现字节级 round-trip。`,
  },
  {
    id: 'v0-1-0-019',
    join: 'tight',
    en: `- **Public \`Value\` enum** — \`Null\`, \`Bool\`, \`Integer\`, \`Float\`,
  \`String\`, \`Array\`, \`Object\` (backed by \`IndexMap\` with
  \`rustc_hash::FxBuildHasher\`).`,
    ru: `- **Публичный enum \`Value\`** — \`Null\`, \`Bool\`, \`Integer\`, \`Float\`,
  \`String\`, \`Array\`, \`Object\` (на основе \`IndexMap\` с
  \`rustc_hash::FxBuildHasher\`).`,
    zh: `- **公共 \`Value\` 枚举** —— \`Null\`、\`Bool\`、\`Integer\`、\`Float\`、
  \`String\`、\`Array\`、\`Object\`(底层为 \`IndexMap\`,使用
  \`rustc_hash::FxBuildHasher\`)。`,
  },
  {
    id: 'v0-1-0-020',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`Value::as_integer\` / \`as_float\`
  accessors; analogous on \`ThinValue\`.`,
    ru: `Аксессоры \`Value::as_integer\` /
  \`as_float\`; аналогичные на \`ThinValue\`.`,
    zh: `访问器 \`Value::as_integer\` /
  \`as_float\`;\`ThinValue\` 上有对应方法。`,
  },
  {
    id: 'v0-1-0-021',
    join: 'tight',
    en: `- **Error reporting** — every syntax error carries a line number;
  deserialization errors carry a dotted path (\`upstreams.[0].port\`).`,
    ru: `- **Сообщения об ошибках** — каждая синтаксическая ошибка несёт номер
  строки; ошибки десериализации несут точечный путь
  (\`upstreams.[0].port\`).`,
    zh: `- **错误报告** —— 每个语法错误都携带行号;反序列化错误携带
  点分路径(\`upstreams.[0].port\`)。`,
  },
  {
    id: 'v0-1-0-022',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `  Typed-scalar violations surface as \`InvalidTypedScalar\` in the
  message prefix.`,
    ru: `Нарушения типизированных скаляров
  отмечаются префиксом \`InvalidTypedScalar\` в сообщении.`,
    zh: `类型化标量违规在消息前缀
  中以 \`InvalidTypedScalar\` 标示。`,
  },
  {
    id: 'v0-1-0-023',
    join: 'tight',
    en: `- **Spec conformance tests** — \`tests/spec_conformance.rs\` runs the
  language-agnostic suite from the \`ktav-lang/spec\` repository
  (resolved via \`KTAV_SPEC_DIR\` env or \`../spec\` fallback).`,
    ru: `- **Spec conformance тесты** — \`tests/spec_conformance.rs\` прогоняет
  language-agnostic набор из репозитория \`ktav-lang/spec\`
  (находится через env-переменную \`KTAV_SPEC_DIR\` или fallback
  \`../spec\`).`,
    zh: `- **Spec conformance 测试** —— \`tests/spec_conformance.rs\` 从
  \`ktav-lang/spec\` 仓库读取语言无关测试套件(通过 env
  \`KTAV_SPEC_DIR\` 或回退 \`../spec\` 解析路径)。`,
  },
  {
    id: 'v0-1-0-024',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Three
  checks: Value-equals-JSON-oracle, invalid-fixtures-rejected, and
  lossless Value-level round-trip through the renderer.`,
    ru: `Три проверки: соответствие Value JSON-оракулу,
  отвержение invalid-fixture-ов и lossless round-trip Value-уровня
  через рендерер.`,
    zh: `三项检查:
  Value 匹配 JSON oracle、invalid fixture 被拒绝、通过渲染器的
  Value 级 round-trip 无损。`,
  },
  {
    id: 'v0-1-0-025',
    join: 'block',
    en: `### Performance (criterion, 22 KB typed config, Windows release)`,
    ru: `### Performance (criterion, typed-конфиг 22 KB, Windows release)`,
    zh: `### Performance(criterion,22 KB 的 typed 配置,Windows release)`,
  },
  {
    id: 'v0-1-0-026',
    join: 'block',
    en: `- \`parse → struct\`: **275 µs** (~80 MB/s)`,
    ru: `- \`parse → struct\`: **275 µs** (~80 MB/s)`,
    zh: `- \`parse → struct\`: **275 µs**(~80 MB/s)`,
  },
  {
    id: 'v0-1-0-027',
    join: 'tight',
    en: `- \`render struct → text\`: **46 µs** (~475 MB/s)`,
    ru: `- \`render struct → text\`: **46 µs** (~475 MB/s)`,
    zh: `- \`render struct → text\`: **46 µs**(~475 MB/s)`,
  },
  {
    id: 'v0-1-0-028',
    join: 'tight',
    common: `- \`round-trip\`: **377 µs**`,
  },
  {
    id: 'v0-1-0-029',
    join: 'block',
    common: `### Dependencies`,
  },
  {
    id: 'v0-1-0-030',
    join: 'block',
    en: `- \`serde\` with \`derive\``,
    ru: `- \`serde\` с \`derive\``,
    zh: `- \`serde\`(含 \`derive\`)`,
  },
  {
    id: 'v0-1-0-031',
    join: 'tight',
    en: `- \`indexmap\` with the \`serde\` feature`,
    ru: `- \`indexmap\` с фичей \`serde\``,
    zh: `- \`indexmap\`(启用 \`serde\` 特性)`,
  },
  {
    id: 'v0-1-0-032',
    join: 'tight',
    en: `- \`rustc-hash\` (FxHash — fast and deterministic; not
  collision-resistant, which a config parser does not need)`,
    ru: `- \`rustc-hash\` (FxHash — быстрый и детерминированный; не
  устойчив к коллизиям, а парсеру конфигов это и не нужно)`,
    zh: `- \`rustc-hash\`(FxHash —— 快且确定性;不抗碰撞,而配置解析器
  并不需要抗碰撞)`,
  },
  {
    id: 'v0-1-0-033',
    join: 'block',
    common: `### MSRV`,
  },
  {
    id: 'v0-1-0-034',
    join: 'block',
    en: `\`rustc 1.70\` or newer.`,
    ru: `\`rustc 1.70\` или новее.`,
    zh: `\`rustc 1.70\` 或更新版本。`,
  },
];
