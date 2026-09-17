// Changelog units for 0.1.0.
export const v0_1_0 = [
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
