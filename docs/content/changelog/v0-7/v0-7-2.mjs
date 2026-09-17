// Changelog units for 0.7.2.
export const v0_7_2 = [
  {
    id: 'v0-7-2-001',
    join: 'block',
    en: `## [0.7.2] — unreleased`,
    ru: `## [0.7.2] — unreleased`,
    zh: `## [0.7.2] —— unreleased`,
  },
  {
    id: 'v0-7-2-002',
    join: 'block',
    en: `The \`cabi\` feature turns the six language bindings' private
copies of the C ABI shim into one macro invocation against this crate.
The feature is off by default — a default build pulls no
\`serde_json\` — and the release is purely additive to the public
API.`,
    ru: `Фича \`cabi\` превращает шесть приватных копий C ABI-шима из шести
языковых биндингов в один макровызов к этому crate. Фича по умолчанию
выключена — обычная сборка не тянет \`serde_json\` — а сам выпуск чисто
аддитивен к публичному API.`,
    zh: `\`cabi\` feature 把六个语言绑定各自私有的 C ABI 垫片副本收敛为对本
crate 的一次宏调用。该 feature 默认关闭——默认构建不会拉取
\`serde_json\`——本次发布对公开 API 是纯增量的。`,
  },
  {
    id: 'v0-7-2-003',
    join: 'block',
    en: `### Added`,
    ru: `### Added`,
    zh: `### 新增`,
  },
  {
    id: 'v0-7-2-004',
    join: 'block',
    en: `- **\`ktav::cabi\` and the \`declare_cabi!\` macro** — the shareable
  half of the C ABI shim: the tagged \`WireValue\` decode (ordered
  maps, lossless integers of any size), the seven document operations
  \`loads\`, \`loads_strict\`, \`dumps\`, \`dumps_force_strings\`,
  \`emit_canonical\` and the new \`format\` and
  \`canonical_from_source\`, and envelope encoding for every failure,
  including the non-\`ktav\` ones.`,
    ru: `- **\`ktav::cabi\` и макрос \`declare_cabi!\`** — разделяемая половина
  C ABI-шима: тегированное декодирование \`WireValue\` (упорядоченные
  карты, целые любой величины без потерь), семь операций над
  документами \`loads\`, \`loads_strict\`, \`dumps\`,
  \`dumps_force_strings\`, \`emit_canonical\` и новые \`format\` и
  \`canonical_from_source\`, и кодирование любого сбоя — включая
  не-ktav-овские — в конверт.`,
    zh: `- **\`ktav::cabi\` 与 \`declare_cabi!\` 宏** —— C ABI 垫片中可共享的
  一半:带标签的 \`WireValue\` 解码(有序映射、任意大小整数无损),七项
  文档操作 \`loads\`、\`loads_strict\`、\`dumps\`、
  \`dumps_force_strings\`、\`emit_canonical\` 与新增的 \`format\` 和
  \`canonical_from_source\`,以及把每一个错误——包括非 ktav 的
  错误——编码进信封。`,
  },
  {
    id: 'v0-7-2-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The module contains no
  \`extern "C"\`: symbols defined in a dependency rlib are not
  guaranteed to survive into a downstream cdylib. The macro expands the
  ten \`#[no_mangle]\` symbols into the calling crate instead, where
  export is guaranteed by construction.`,
    ru: `В модуле нет ни одного \`extern "C"\`:
  символы, определённые в rlib-зависимости, не гарантированно
  выживают в cdylib потребителя. Макрос вместо этого разворачивает
  десять символов \`#[no_mangle]\` в вызывающий crate, где экспорт
  гарантирован по построению.`,
    zh: `模块本身不含任何 \`extern "C"\`:依赖 rlib
  中定义的符号不保证能存活到下游的 cdylib。宏改为把十个
  \`#[no_mangle]\` 符号展开进调用方的 crate,在那里导出由构造方式
  保证。`,
  },
  {
    id: 'v0-7-2-006',
    join: 'block',
    en: `- **\`ktav_abi_version()\` and \`ktav::cabi::ABI_VERSION\`** — the
  version of the exported ABI *shape* (starts at 1), so a host that
  picks up a stale native library refuses to load instead of corrupting
  memory. The crate version changes every release; the ABI shape almost
  never.`,
    ru: `- **\`ktav_abi_version()\` и \`ktav::cabi::ABI_VERSION\`** — версия
  *формы* экспортируемого ABI (начинается с 1): хост, получивший
  устаревшую нативную библиотеку, откажется от загрузки вместо порчи
  памяти. Версия crate меняется в каждом выпуске, форма ABI — почти
  никогда.`,
    zh: `- **\`ktav_abi_version()\` 与 \`ktav::cabi::ABI_VERSION\`** —— 导出
  ABI *形态* 的版本(从 1 开始):拿到过期原生库的宿主会拒绝加载而不是
  破坏内存。crate 版本每次发布都变,ABI 形态几乎从不变。`,
  },
  {
    id: 'v0-7-2-007',
    join: 'block',
    en: `- **\`ktav::cabi::library_file_name\` and \`docs/CABI.md\`** — the
  artifact naming convention that the \`php\`, \`csharp\` and
  \`golang\` loaders each derived independently
  (\`ktav_cabi-windows-amd64.dll\`,
  \`libktav_cabi-darwin-arm64.dylib\`, …, the \`$KTAV_LIB_PATH\`
  override), written down once, executable, and pinned by tests.`,
    ru: `- **\`ktav::cabi::library_file_name\` и \`docs/CABI.md\`** — конвенция
  имён артефактов, которую загрузчики \`php\`, \`csharp\` и \`golang\`
  выводили каждый по-своему (\`ktav_cabi-windows-amd64.dll\`,
  \`libktav_cabi-darwin-arm64.dylib\`, …, переопределение через
  \`$KTAV_LIB_PATH\`), записанная один раз, исполнимая и закреплённая
  тестами.`,
    zh: `- **\`ktav::cabi::library_file_name\` 与 \`docs/CABI.md\`** ——
  \`php\`、\`csharp\` 与 \`golang\` 的加载器各自独立推导的产物命名约定
  (\`ktav_cabi-windows-amd64.dll\`、
  \`libktav_cabi-darwin-arm64.dylib\`、……、\`$KTAV_LIB_PATH\` 覆盖),
  如今一次性写下来、可执行、并由测试钉死。`,
  },
  {
    id: 'v0-7-2-008',
    join: 'block',
    en: `- **A load test for the whole surface** — a fixture cdylib whose
  body is one \`declare_cabi!()\` invocation, built and dlopened by the
  suite; all ten exported symbols resolve through \`libloading\`, and
  \`ktav_abi_version\`, \`ktav_loads\`, \`ktav_format\` and
  \`ktav_canonical_from_source\` are called for real.
  The wire round-trip runs over the conformance corpus: 221
  \`valid/\` fixtures keep their \`Value\` across \`loads\` ->
  \`dumps\`, and 221 canonical byte oracles match \`emit_canonical\`
  through the wire.`,
    ru: `- **Загрузочный тест всей поверхности** — cdylib-фикстура, чьё тело —
  один вызов \`declare_cabi!()\`; тест собирает её, открывает через
  dlopen, разрешает все десять экспортированных символов через
  \`libloading\` и по-настоящему вызывает \`ktav_abi_version\`,
  \`ktav_loads\`, \`ktav_format\` и \`ktav_canonical_from_source\`. Wire-проверка идёт по
  конформанс-корпусу: 221 фикстура из \`valid/\` сохраняет \`Value\`
  через \`loads\` -> \`dumps\`, и 221 канонический байтовый оракул
  совпадает с \`emit_canonical\` через wire.`,
    zh: `- **整个表面的加载测试** —— 一份 body 仅有一行 \`declare_cabi!()\`
  调用的 fixture cdylib;测试套件构建它、以 dlopen 打开、经
  \`libloading\` 解析出全部十个导出符号,并真实调用
  \`ktav_abi_version\`、\`ktav_loads\`、\`ktav_format\` 与
  \`ktav_canonical_from_source\`。wire 往返
  覆盖一致性语料库:221 个 \`valid/\` fixture 在 \`loads\` ->
  \`dumps\` 之间保持 \`Value\` 不变,221 个规范字节预言机与经 wire 的
  \`emit_canonical\` 逐字节一致。`,
  },
  {
    id: 'v0-7-2-008d',
    join: 'block',
    en: `- **\`ErrorEnvelope.message\` — the error's \`Display\` rendering,
  verbatim.** The envelope now has ten fields; \`message\` is appended
  last, so the nine that shipped in 0.7.1 keep their positions, and it
  is the only field that is never \`null\`. It exists because the other
  nine are structured data and none of them is prose: a binding that
  wanted an exception message had to assemble one itself, and five of
  them did — Go, Java, PHP, C# and JavaScript each invented a different
  layout, and none matched what the crate and the PyO3 binding already
  printed for the same input. Hosts surface this field as-is.`,
    ru: `- **\`ErrorEnvelope.message\` — дословное \`Display\`-представление
  ошибки.** В конверте теперь десять полей; \`message\` дописано в
  конец, поэтому девять полей из 0.7.1 сохранили свои позиции, и это
  единственное поле, которое никогда не \`null\`. Оно понадобилось
  потому, что остальные девять — структурированные данные, и ни одно из
  них не является текстом: биндингу, которому нужен текст исключения,
  приходилось собирать его самому — и пятеро так и сделали. Go, Java,
  PHP, C# и JavaScript изобрели пять разных раскладок, и ни одна не
  совпадала с тем, что для того же ввода уже печатали сам crate и
  биндинг на PyO3. Host показывает это поле как есть.`,
    zh: `- **\`ErrorEnvelope.message\` —— 错误的 \`Display\` 渲染结果,逐字
  携带。** 信封现在有十个字段;\`message\` 追加在最后,因此 0.7.1 中
  发布的九个字段位置不变,而它是唯一永不为 \`null\` 的字段。之所以需要
  它,是因为其余九个都是结构化数据,没有一个是文字:想要异常消息的
  绑定只能自己拼装 —— 而且有五个确实这么做了。Go、Java、PHP、C# 与
  JavaScript 各自发明了一种不同的排版,没有一种与 crate 本身和 PyO3
  绑定对同一输入已经打印的内容一致。host 直接原样呈现该字段。`,
  },
  {
    id: 'v0-7-2-008b',
    join: 'block',
    en: `### Changed`,
    ru: `### Changed`,
    zh: `### 变更`,
  },
  {
    id: 'v0-7-2-008c',
    join: 'block',
    en: `- **\`ktav_version()\` now reports this crate's version, not the
  binding's.** Each binding's private shim used to return its own
  package version; the shared macro cannot know it. The number a host
  reads therefore changes meaning, and constants such as a binding's
  \`LIB_VERSION\` become stale without any error.

  This is the honest reading — the native library *is* \`ktav\` — but
  it removes the check those constants were performing. Use
  \`ktav_abi_version()\` for that instead: it answers "is this library
  the shape I was built against", which is the question a loader
  actually has, and it does not move on every release.`,
    ru: `- **\`ktav_version()\` теперь сообщает версию этого crate-а, а не
  биндинга.** Приватный шим каждого биндинга возвращал собственную
  версию пакета; общий макрос знать её не может. Смысл числа, которое
  читает host, тем самым меняется, а константы вроде \`LIB_VERSION\` в
  биндинге устаревают без единой ошибки.

  Это честное прочтение — нативная библиотека и **есть** \`ktav\`, — но
  оно убирает проверку, которую эти константы выполняли. Для неё
  используйте \`ktav_abi_version()\`: он отвечает на вопрос «та ли это
  библиотека по форме, под которую меня собрали», то есть на настоящий
  вопрос загрузчика, и не меняется каждый выпуск.`,
    zh: `- **\`ktav_version()\` 现在报告本 crate 的版本,而非绑定的版本。**
  各绑定的私有垫片过去返回自己的包版本;共享宏无从得知它。因此宿主
  读到的数字含义发生了变化,绑定中诸如 \`LIB_VERSION\` 的常量会在没有
  任何报错的情况下过时。

  这是更诚实的读法 —— 原生库**就是** \`ktav\` —— 但它移除了那些常量
  原本承担的检查。请改用 \`ktav_abi_version()\`:它回答「这个库是否
  与我构建时的形状一致」,这才是加载器真正的问题,而且它不会每次
  发布都变动。`,
  },
  {
    id: 'v0-7-2-009',
    join: 'block',
    en: `### Fixed`,
    ru: `### Fixed`,
    zh: `### 修复`,
  },
  {
    id: 'v0-7-2-010',
    join: 'block',
    en: `- **\`ErrorEnvelope.body\` now carries the payload of
  \`Error::Message\` and \`Error::Syntax\`.** It was always \`null\`
  for these two classes. That broke under the C ABI's uniformity rule:
  non-\`ktav\` failures travel as \`Message\`, and a caller would have
  received \`{"error":"Message"}\` plus eight nulls instead of the
  diagnostic. No shipped consumer can regress — the envelope has never
  shipped in any binding.`,
    ru: `- **\`ErrorEnvelope.body\` теперь несёт полезную нагрузку
  \`Error::Message\` и \`Error::Syntax\`.** Раньше для этих двух
  классов там всегда был \`null\`. Это ломалось на правиле
  единообразия C ABI: сбои вне \`ktav\` путешествуют как \`Message\`, и
  вместо диагностики вызывающий получил бы \`{"error":"Message"}\` и
  восемь null. Регресса у выпущенных потребителей нет — конверт ещё не
  выходил ни в одном биндинге.`,
    zh: `- **\`ErrorEnvelope.body\` 现在携带 \`Error::Message\` 与
  \`Error::Syntax\` 的载荷。** 这两个类此前恒为 \`null\`。在 C ABI 的
  统一性规则下这会坏掉:非 ktav 的失败以 \`Message\` 形态传递,调用方将
  只会看到 \`{"error":"Message"}\` 和八个 null,而不是诊断信息。已发布
  的消费方不会回归——错误信封从未随任何绑定发布过。`,
  },
];
