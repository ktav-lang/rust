// Changelog units for 0.8.0.
export const v0_8_0 = [
  {
    id: 'v0-8-0-001',
    join: 'block',
    en: `## [0.8.0] — 2026-09-20`,
    ru: `## [0.8.0] — 2026-09-20`,
    zh: `## [0.8.0] —— 2026-09-20`,
  },
  {
    id: 'v0-8-0-002',
    join: 'block',
    en: `Two things ship together. A decimal with a redundant leading zero no
longer infers a number — \`zip: 01234\` parses to the String
\`"01234"\`, not \`Integer(1234)\` — which changes what a document
means and is why this is a minor bump rather than a patch. Alongside it,
the \`cabi\` feature turns the six language bindings' private copies of
the C ABI shim into one macro invocation against this crate; that half
is purely additive and off by default, so a default build still pulls no
\`serde_json\`.`,
    ru: `Выходят две вещи вместе. Десятичное число с избыточным ведущим нулём
больше не выводится как число — \`zip: 01234\` разбирается в String
\`"01234"\`, а не в \`Integer(1234)\`, — это меняет смысл документа и
именно поэтому выпуск минорный, а не патч. Вместе с этим фича \`cabi\`
превращает шесть приватных копий C ABI-шима из шести языковых биндингов
в один макровызов к этому crate; эта половина чисто аддитивна и по
умолчанию выключена, так что обычная сборка по-прежнему не тянет
\`serde_json\`.`,
    zh: `本次发布同时带来两件事。带冗余前导零的十进制数不再被推断为数字 ——
\`zip: 01234\` 解析为 String \`"01234"\`,而不是 \`Integer(1234)\` ——
这改变了文档的含义,也正是本次发布为 minor 而非 patch 的原因。与之同行,
\`cabi\` feature 把六个语言绑定各自私有的 C ABI 垫片副本收敛为对本 crate
的一次宏调用;这一半是纯增量的、默认关闭,因此默认构建依然不会拉取
\`serde_json\`。`,
  },
  {
    id: 'v0-8-0-003',
    join: 'block',
    en: `### Changed`,
    ru: `### Changed`,
    zh: `### 变更`,
  },
  {
    id: 'v0-8-0-004',
    join: 'block',
    en: `- **A redundant leading zero is no longer a number
  ([spec 0.8 § 5.2](https://github.com/ktav-lang/spec/blob/main/versions/0.8/spec.md)).**
  A base-10 digit run whose first digit is \`0\` with at least one
  further digit — \`01234\`, \`-045\`, \`00\`, \`0_7\`, and a float's
  integer part in \`01.5\` and \`05e3\` — is now a String carrying the
  digits as written. \`zip: 01234\` was \`Integer(1234)\`, which
  destroyed a postcode, a phone number or a zero-padded id without
  saying so; it is now \`"01234"\`.`,
    ru: `- **Избыточный ведущий ноль больше не число
  ([spec 0.8 § 5.2](https://github.com/ktav-lang/spec/blob/main/versions/0.8/spec.ru.md)).**
  Ряд цифр по основанию 10, чья первая цифра \`0\`, при этом следует
  хотя бы ещё одна цифра — \`01234\`, \`-045\`, \`00\`, \`0_7\`, а также
  целая часть float в \`01.5\` и \`05e3\`, — теперь String, несущая
  цифры как написано. \`zip: 01234\` был \`Integer(1234)\`, что молча
  уничтожало почтовый индекс, номер телефона или дополненный нулями
  идентификатор; теперь это \`"01234"\`.`,
    zh: `- **冗余前导零不再是数字
  ([spec 0.8 § 5.2](https://github.com/ktav-lang/spec/blob/main/versions/0.8/spec.zh.md))。**
  以 \`0\` 开头且后面至少还有一位数字的十进制数字串 —— \`01234\`、
  \`-045\`、\`00\`、\`0_7\`,以及 \`01.5\` 与 \`05e3\` 中 float 的整数
  部分 —— 现在是按书写原样携带这些数字的 String。\`zip: 01234\` 曾是
  \`Integer(1234)\`,这会不声不响地毁掉邮政编码、电话号码或补零的
  标识符;现在它是 \`"01234"\`。`,
  },
  {
    id: 'v0-8-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The rule holds at every entry point —
  \`parse\`, \`parse_strict\`, \`from_str\`, the thin event parser and
  the C ABI — and it is confined to base 10: \`0\`, \`0.5\`, \`0x1A\`,
  \`0o755\`, \`0b1010\`, \`1_000_000\` and \`+7\` still infer numbers
  exactly as before. Canonical output does not move either, because the
  \`::\` marker keys off § 3.6's grammar, which 0.8 left untouched: the
  String \`"01234"\` still renders as \`zip:: 01234\`. Two consequences
  for callers: code that relied on the old coercion must parse the
  String itself, and \`parse_strict\` no longer reports
  \`LossyScalar\` for these forms, since nothing is lost any more.`,
    ru: `Правило действует в каждой точке входа —
  \`parse\`, \`parse_strict\`, \`from_str\`, тонкий event-парсер и
  C ABI — и ограничено основанием 10: \`0\`, \`0.5\`, \`0x1A\`,
  \`0o755\`, \`0b1010\`, \`1_000_000\` и \`+7\` по-прежнему выводятся в
  числа в точности как раньше. Каноническая запись тоже не меняется,
  потому что маркер \`::\` привязан к грамматике § 3.6, которую 0.8 не
  тронула: String \`"01234"\` всё так же выводится как
  \`zip:: 01234\`. Два следствия для вызывающих: код, полагавшийся на
  старое приведение, должен сам разбирать String, а \`parse_strict\`
  больше не сообщает \`LossyScalar\` для этих форм, поскольку терять
  уже нечего.`,
    zh: `该规则在每一个入口都成立 ——
  \`parse\`、\`parse_strict\`、\`from_str\`、thin event parser 与
  C ABI —— 并且仅限于十进制:\`0\`、\`0.5\`、\`0x1A\`、\`0o755\`、
  \`0b1010\`、\`1_000_000\` 与 \`+7\` 仍与此前完全一致地推断为数字。
  规范化输出同样不变,因为 \`::\` 标记绑定到 § 3.6 的语法,而 0.8 未
  触动它:String \`"01234"\` 依旧写作 \`zip:: 01234\`。对调用方有两点
  影响:依赖旧的强制转换的代码须自行解析该 String;而
  \`parse_strict\` 不再为这些形态报告 \`LossyScalar\`,因为已经没有
  任何信息会丢失。`,
  },
  {
    id: 'v0-8-0-006',
    join: 'block',
    en: `- **\`spec-version\` metadata moves to \`0.8.0\`, and the pinned
  \`spec\` submodule with it.** The suite now runs the 0.8 conformance
  corpus: 223 \`valid/\`, 74 \`invalid/\`, 5 \`unrepresentable/\`, 4
  \`parseable-unrepresentable/\` and 13 \`strict-lossy/\` fixtures —
  the exact counts § 8.5's \`manifest.json\` declares, loaded and
  checked before any fixture is enumerated. The rule above is pinned by
  \`valid/numbers/integer/leading_zero_is_string\` and
  \`valid/numbers/float/leading_zero_is_string\`, which carry both sides
  of the boundary in one document each, so every implementation is held
  to it and not just this one.`,
    ru: `- **Метаданные \`spec-version\` переходят на \`0.8.0\`, а вместе с ними
  и закреплённый submodule \`spec\`.** Набор тестов теперь идёт по
  conformance-корпусу 0.8: 223 фикстуры \`valid/\`, 74 \`invalid/\`, 5
  \`unrepresentable/\`, 4 \`parseable-unrepresentable/\` и 13
  \`strict-lossy/\` — в точности те количества, которые объявляет
  \`manifest.json\` из § 8.5, загружаемые и проверяемые до перечисления
  любой фикстуры. Правило выше закреплено фикстурами
  \`valid/numbers/integer/leading_zero_is_string\` и
  \`valid/numbers/float/leading_zero_is_string\`, каждая из которых
  несёт обе стороны границы в одном документе, так что правилу
  подчиняется каждая реализация, а не только эта.`,
    zh: `- **\`spec-version\` 元数据升至 \`0.8.0\`,固定的 \`spec\` submodule
  也随之移动。** 测试套件现在跑的是 0.8 一致性语料库:223 个
  \`valid/\`、74 个 \`invalid/\`、5 个 \`unrepresentable/\`、4 个
  \`parseable-unrepresentable/\` 与 13 个 \`strict-lossy/\` fixture ——
  正是 § 8.5 的 \`manifest.json\` 所声明的精确数量,并在枚举任何
  fixture 之前加载与校验。上述规则由
  \`valid/numbers/integer/leading_zero_is_string\` 与
  \`valid/numbers/float/leading_zero_is_string\` 钉死,两者各自在一份
  文档里同时承载边界的两侧,因此受约束的是每一个实现,而不只是这一个。`,
  },
  {
    id: 'v0-8-0-007',
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
    id: 'v0-8-0-008',
    join: 'block',
    en: `### Added`,
    ru: `### Added`,
    zh: `### 新增`,
  },
  {
    id: 'v0-8-0-009',
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
    id: 'v0-8-0-010',
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
    id: 'v0-8-0-011',
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
    id: 'v0-8-0-012',
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
    id: 'v0-8-0-013',
    join: 'block',
    en: `- **A load test for the whole surface** — a fixture cdylib whose
  body is one \`declare_cabi!()\` invocation, built and dlopened by the
  suite; all ten exported symbols resolve through \`libloading\`, and
  \`ktav_abi_version\`, \`ktav_loads\`, \`ktav_format\` and
  \`ktav_canonical_from_source\` are called for real.
  The wire round-trip runs over the conformance corpus: 223
  \`valid/\` fixtures keep their \`Value\` across \`loads\` ->
  \`dumps\`, and 223 canonical byte oracles match \`emit_canonical\`
  through the wire.`,
    ru: `- **Загрузочный тест всей поверхности** — cdylib-фикстура, чьё тело —
  один вызов \`declare_cabi!()\`; тест собирает её, открывает через
  dlopen, разрешает все десять экспортированных символов через
  \`libloading\` и по-настоящему вызывает \`ktav_abi_version\`,
  \`ktav_loads\`, \`ktav_format\` и \`ktav_canonical_from_source\`. Wire-проверка идёт по
  конформанс-корпусу: 223 фикстуры из \`valid/\` сохраняют \`Value\`
  через \`loads\` -> \`dumps\`, и 223 канонических байтовых оракула
  совпадают с \`emit_canonical\` через wire.`,
    zh: `- **整个表面的加载测试** —— 一份 body 仅有一行 \`declare_cabi!()\`
  调用的 fixture cdylib;测试套件构建它、以 dlopen 打开、经
  \`libloading\` 解析出全部十个导出符号,并真实调用
  \`ktav_abi_version\`、\`ktav_loads\`、\`ktav_format\` 与
  \`ktav_canonical_from_source\`。wire 往返
  覆盖一致性语料库:223 个 \`valid/\` fixture 在 \`loads\` ->
  \`dumps\` 之间保持 \`Value\` 不变,223 个规范字节预言机与经 wire 的
  \`emit_canonical\` 逐字节一致。`,
  },
  {
    id: 'v0-8-0-014',
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
    id: 'v0-8-0-015',
    join: 'block',
    en: `### Fixed`,
    ru: `### Fixed`,
    zh: `### 修复`,
  },
  {
    id: 'v0-8-0-016',
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
