// Changelog units for 0.7.0.
export const v0_7_0 = [
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
    id: 'v0-7-0-052',
    join: 'tight',
    en: `- The published tarball no longer carries the vendored spec
  corpus, the internal review notes or the documentation generator:
  1,715 files (1.4 MiB compressed) down to 188 (403 KiB). Nothing that
  the library needs to build was removed — run the conformance suite
  from a git checkout, where nothing is excluded.`,
    ru: `- Публикуемый tarball больше не тащит вендоренный корпус спеки,
  внутренние отчёты ревью и генератор документации: 1 715 файлов
  (1,4 MiB в сжатом виде) превратились в 188 (403 KiB). Ничего из
  того, что нужно для сборки библиотеки, не убрано — conformance-сюиту
  гоняйте из git-чекаута, где не исключено ничего.`,
    zh: `- 发布的 tarball 不再携带随仓库附带的规范语料、内部评审记录与文档
  生成器:文件数从 1,715(压缩后 1.4 MiB)降至 188(403 KiB)。构建库
  所需的内容一个都没少——conformance 套件请在 git 检出中运行,那里不
  排除任何文件。`,
  },
];
