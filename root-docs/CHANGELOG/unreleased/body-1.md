>>>>> lang=en
## [Unreleased]

### Changed

- **C ABI 2 (0.9 preparation):** add `ktav_edit_value` and `ktav_edit_value_segments` with source/path/replacement-wire byte inputs and the existing output ownership contract. Replacement wire values may be scalars; edit failures use `EditInvalidPath`, `EditNotFound` and `EditUnsupported` in the unchanged ten-field envelope. Source parse/UTF-8 diagnostics retain their 0.9 locations and execute the versioned diagnostic fixtures through both new operations. Native loaders must reject ABI 1. This is not a package release or a binding package-version bump.

- **Breaking in 0.9.0:** `InvalidUtf8` gains `line`/`span`; `UnclosedCompound` gains the opener's `line`. Downstream constructors and exhaustive field patterns must migrate. Direct Rust errors, public envelopes and C ABI source operations share raw-byte locations; UTF-8 spans are non-empty and EOF spans cover the innermost opener through EOF. LF, lone CR, CRLF and BOM offsets are preserved, including `Span::line_col` and envelope `line_text`. The ten-field JSON schema and I/O/writer location rules are unchanged. New regression oracles live only in the draft spec `versions/0.9/tests` and run across source entry points; CI refuses to skip a missing corpus. Version 0.9.0 is in preparation, not published.

- Source editing preserves dotted/inline/block container forms, including deep array edits and empty containers. Expand only the necessary inline wrappers for multiline text, growth past `max(100, original character length)`, or at least four escapes adding 25% overhead. Preserve unrelated branches and comments; retain inline when a lossless block is unavailable. Add exact-output and semantic regressions for both path APIs, Unicode thresholds, escaped siblings and BOM/EOL/EOF.

### Added

- Lossless source edit operations `ktav::edit::{insert_value, remove_value, rename_key}` (and their `*_segments` variants): append a new Object key with an explicit `AlreadyExists` collision error and no hidden overwrites, delete a leaf/subtree/array element while keeping empty ancestors and container forms, and rename an Object member in place preserving order, value and comments. Same path grammar, inline readability thresholds, comment-loss refusals and post-edit semantic verification; invalid sources fail closed with `Unsupported`. Exposed to hosts as six additive, backward-compatible ABI 2 exports — `ktav_insert_value`, `ktav_insert_value_segments`, `ktav_remove_value` (two input buffers, no replacement), `ktav_remove_value_segments`, `ktav_rename_key` and `ktav_rename_key_segments` (UTF-8 literal decoded key); new bindings must require these symbols, and no ABI version is bumped.
- Source-editing API: `ktav::edit::edit_value` accepts an escaped dotted path, and `edit_value_segments` accepts a slice/vector of literal segments. Both replace an existing value without reformatting the document, preserve surrounding bytes and comments, and return a new `String`.
- Context-sensitive object keys and array indexes, `\.`/`\\` path escapes, minimal raw-marker transitions, unchanged-value no-ops and post-edit semantic verification. Typed `EditError` rejects malformed paths, missing targets and unsupported lossless replacements, including whole noncontiguous objects and structural changes that would discard comments.
- Exact-text regression tests and EN/RU/ZH usage examples. No syntax, existing parser/formatter API or package-version change; this feature is not part of the published 0.8.1 crate.

### Performance

- Reserve the edited document's actual output length instead of retaining removed source capacity. No-op and missing-path edits skip the positional tree after validating the original document.
- Write nested inline replacements into one buffer; avoid temporary character vectors and a second copy of parsed textual path segments. Keep semantic verification and byte-preserving behavior. Add short paired benchmarks with separate compilation and an external 15-second process limit.
- The positional pass shares structural classifiers with the parser without constructing unused values or multiline bodies. Verify the edited target and untouched tree without cloning the replacement; render borrowed block compounds without a temporary root wrapper. Preserve validation, writer error paths and byte-for-byte output, with focused regression tests and expanded paired benchmarks.
- Borrow literal path slices and unescaped textual segments instead of copying each segment. Assemble raw-marker prefixes and unchanged replacement bodies directly into the result; render verbatim replacements in one buffer without an intermediate EOL-converted string. Preserve exact output and full semantic verification; measure text and segment paths separately in short paired benchmarks.
- Locate diagnostic line text by reverse terminator search instead of rescanning the document prefix. Probe JSON strings with safe eight-byte operations, bulk-copy unescaped UTF-8, specialize escaped ASCII and retain the Unicode encoder. Allocate reopen-merge child buffers and seen-key storage lazily. Preserve exact bytes and locations; add allocation/escaping regressions and paired runtime benchmarks with one external 15-second limit for the entire process batch.

### Documentation

- Expand README examples for dotted and literal-key paths, mixed arrays, String/Integer identity, multiline unfolding, byte-identical no-op and BOM/CRLF/EOF preservation. Clarify 0.9 byte diagnostics, safe refusal, ABI 2 and standalone pure usage; these additions are not part of published 0.8.1.

>>>>> lang=ru
## [Не выпущено]

### Изменено

- **C ABI 2 (подготовка 0.9):** добавлены `ktav_edit_value` и `ktav_edit_value_segments` с byte inputs исходника, пути и replacement wire при прежнем владении output buffers. Wire-замена может быть скаляром; edit-ошибки — `EditInvalidPath`, `EditNotFound` и `EditUnsupported` в неизменном envelope из десяти полей. Source parse/UTF-8 ошибки сохраняют координаты 0.9; versioned diagnostic fixtures исполняются и через обе новые операции. Native-загрузчики обязаны отклонять ABI 1. Это не публикация пакетов и не повышение версий биндингов.

- **Breaking в 0.9.0:** в `InvalidUtf8` добавлены `line`/`span`, в `UnclosedCompound` — `line` открывающего маркера. Downstream-конструкторы и patterns, перечисляющие все поля, требуют миграции. Прямые Rust-ошибки, публичный envelope и C ABI source operations используют общие координаты исходных байтов; UTF-8 spans непустые, EOF spans охватывают самый внутренний незакрытый opener до EOF. LF, одиночный CR, CRLF и BOM учитываются без нормализации, в том числе в `Span::line_col` и envelope `line_text`. Схема JSON из десяти полей и отсутствие позиций у I/O/writer не меняются. Новые regression oracles находятся только в draft спеки `versions/0.9/tests` и проверяются через source entry points; CI запрещает пропуск отсутствующего corpus. Версия 0.9.0 готовится, но не опубликована.

- Source-edit сохраняет dotted/inline/блочную форму контейнеров, в том числе при глубоких правках массивов и опустошении. Раскрываются только необходимые inline-обёртки: при многострочном тексте, росте сверх `max(100, прежняя длина в символах)` либо не менее четырёх escapes с накладными расходами от 25%. Незатронутые ветви и комментарии сохраняются; если lossless-блок невозможен, остаётся inline. Добавлены exact-output и семантические регрессии обоих path API, Unicode-порогов, экранированных соседей и BOM/EOL/EOF.

### Добавлено

- Lossless-операции исходного редактирования `ktav::edit::{insert_value, remove_value, rename_key}` (и их `*_segments`-варианты): добавление нового ключа Object с явной ошибкой коллизии `AlreadyExists` и без скрытых перезаписей; удаление листа/поддерева/элемента массива с сохранением пустых предков и форм контейнеров; переименование члена Object на месте с сохранением порядка, значения и комментариев. Та же грамматика путей, пороги inline-читаемости, отказы при потере комментариев и семантическая верификация результата; невалидный исходник — fail closed `Unsupported`. Доступно хостам как шесть аддитивных, обратно совместимых экспортов ABI 2 — `ktav_insert_value`, `ktav_insert_value_segments`, `ktav_remove_value` (два входных буфера, без replacement), `ktav_remove_value_segments`, `ktav_rename_key` и `ktav_rename_key_segments` (UTF-8 литеральный decoded ключ); `EditAlreadyExists` отображается в `EditAlreadyExists`; новым биндингам нужны эти символы, ABI-версия не меняется.
- Rust API редактирования исходного текста: `ktav::edit::edit_value` принимает экранированный dotted-путь, `edit_value_segments` — слайс/вектор буквальных сегментов. Обе функции заменяют существующее значение без форматирования документа, сохраняют окружающие байты и комментарии и возвращают новый `String`.
- Контекстное различение ключей объектов и индексов массивов, escape-последовательности пути `\.`/`\\`, минимальное переключение raw-маркера, неизменный текст при совпадении значений и семантическая проверка результата. Типизированный `EditError` отвергает ошибочные пути, отсутствующие цели и неподдерживаемые lossless-замены, включая целые noncontiguous-объекты и структурные правки с потерей комментариев.
- Регрессионные тесты точного текста и примеры EN/RU/ZH. Синтаксис, существующий API парсера/formatter и версия пакета не меняются; функционал не входит в опубликованный crate 0.8.1.

### Производительность

- Буфер результата резервируется по фактической длине, без удержания памяти удалённого исходного значения. No-op и отсутствующий путь обходятся без позиционного дерева после проверки исходного документа.
- Вложенные inline-замены пишутся в один буфер, без временного вектора символов и повторного копирования разобранных сегментов текстового пути. Семантическая проверка и сохранение исходных байтов остаются. Добавлены короткие парные benchmarks с отдельной компиляцией и внешним лимитом 15 секунд на процесс.
- Позиционный проход использует общие с parser структурные классификаторы без построения ненужных значений и multiline-тел. Проверка цели и неизменной части дерева обходится без клонирования replacement; блочные контейнеры рендерятся по заимствованию без временной корневой обёртки. Валидация, пути writer-ошибок и точные выходные байты сохранены; добавлены регрессии и расширены парные benchmarks.
- Буквальные сегменты пути и сегменты текстового пути без escape заимствуются без копирования каждого сегмента. Префиксы raw-маркера и тела замен, не требующие преобразования, переносятся непосредственно в результат; verbatim-замены рендерятся в один буфер без промежуточной строки с преобразованными EOL. Точные выходные байты и полная семантическая проверка сохранены; короткие парные benchmarks отдельно измеряют text- и segments-пути.
- Строка диагностики находится обратным поиском терминатора без повторного прохода по префиксу документа. JSON-строки проверяются безопасными восьмибайтовыми операциями; UTF-8 без escapes переносится целиком, экранируемый ASCII обрабатывается специализированно, Unicode encoder сохраняется. Дочерние буферы и хранилища seen-ключей reopen-merge выделяются лениво. Точные байты и координаты сохранены; добавлены регрессии аллокаций/экранирования и парные runtime benchmarks с единым внешним лимитом 15 секунд на весь batch процессов.

### Документация

- Дополнены примеры README: dotted-пути и буквальные ключи, смешанные массивы, различие String/Integer, раскрытие многострочной замены, побайтовый no-op и сохранение BOM/CRLF/EOF. Уточнены байтовая диагностика 0.9, безопасные отказы, ABI 2 и самостоятельный pure-бэкенд; дополнения не относятся к опубликованному 0.8.1.

>>>>> lang=zh
## [尚未发布]

### 变更

- **C ABI 2（0.9 准备版本）：** 新增 `ktav_edit_value` 和 `ktav_edit_value_segments`，接受源文档、路径及替换 wire 值的字节输入，沿用原有输出缓冲区所有权。替换值可以是标量；编辑错误使用 `EditInvalidPath`、`EditNotFound` 和 `EditUnsupported`，仍为十字段 envelope。源解析及 UTF-8 诊断保留 0.9 坐标，两个新操作也执行版本化诊断 fixtures。原生加载器必须拒绝 ABI 1。这不发布包，也不提高绑定包版本。

- **0.9.0 破坏性变更：** `InvalidUtf8` 新增 `line`/`span`，`UnclosedCompound` 新增起始标记的 `line`。下游构造器及枚举全部字段的模式需要迁移。直接 Rust 错误、公开 envelope 和 C ABI 源操作共用原始字节坐标；UTF-8 区间非空，EOF 区间覆盖最内层未闭合起始标记至 EOF。LF、单独 CR、CRLF 及 BOM 均不经规范化，`Span::line_col` 和 envelope 的 `line_text` 同样遵守该规则。十字段 JSON 格式及 I/O/writer 无源位置的规则不变。新增回归 oracle 仅位于规范草案 `versions/0.9/tests`，由各源入口执行；CI 不允许因语料缺失而跳过。0.9.0 尚在准备中，未发布。

- 源编辑保留点分键、行内及块容器形式，包括数组深层编辑和清空容器。仅在多行文本、行长增长超过 `max(100, 原始字符长度)`，或至少四次转义带来至少 25% 开销时展开必要的行内外层容器。保留无关分支和注释；无法无损转换为块时保留行内形式。为两种路径 API、Unicode 阈值、转义邻居及 BOM/EOL/EOF 增加精确输出和语义回归测试。

### 新增

- 无损源编辑操作 `ktav::edit::{insert_value, remove_value, rename_key}`（及各自的 `*_segments` 变体）：追加新的 Object 键，冲突时报显式 `AlreadyExists` 且绝不隐蔽覆盖；删除叶子/子树/数组元素时保留空祖先和容器形式；在原位重命名 Object 成员并保留顺序、值与注释。沿用相同的路径语法、行内可读性阈值、注释丢失拒绝及编辑后语义验证；无效源以 `Unsupported` 失败闭合。通过六个增量且向后兼容的 ABI 2 导出提供给宿主——`ktav_insert_value`、`ktav_insert_value_segments`、`ktav_remove_value`（两个输入缓冲区，无替换参数）、`ktav_remove_value_segments`、`ktav_rename_key` 和 `ktav_rename_key_segments`（UTF-8 字面已解码键）；`EditAlreadyExists` 映射为 `EditAlreadyExists`；新绑定必须要求这些符号，ABI 版本不变。
- Rust 源文本编辑 API：`ktav::edit::edit_value` 接受带转义的点分路径，`edit_value_segments` 接受字面路径段切片/向量。两者均在不重新格式化文档的情况下替换已有值，保留周围字节与注释，并返回新的 `String`。
- 按上下文区分对象键与数组索引，支持路径转义 `\.`/`\\`、最小化 raw 标记调整、值不变时保留原文，以及编辑后的语义验证。类型化的 `EditError` 拒绝错误路径、不存在的目标及不支持的无损替换，包括整个不连续对象和会丢失注释的结构修改。
- 精确文本回归测试及 EN/RU/ZH 用法示例。不改变语法、现有解析器/formatter API 或包版本；此功能不包含在已发布的 0.8.1 crate 中。

### 性能

- 按编辑后实际长度预留输出缓冲区，不再保留已删除源值占用的容量。验证原文后，值不变或路径不存在时不再构建位置树。
- 将嵌套行内替换直接写入同一缓冲区，避免临时字符向量及文本路径段的二次复制。保留语义验证和原始字节保护。新增短时配对基准，编译独立进行，每个进程由外部工具限制为 15 秒。
- 位置扫描与解析器共用结构分类，不再构造无用值或多行正文。验证目标和未修改的树时不再克隆替换值；以借用方式渲染块容器，避免临时根包装。保留验证、writer 错误路径和逐字节输出，并增加专项回归测试及配对基准场景。
- 借用字面路径段及不含转义的文本路径段，不再逐段复制。将 raw 标记前缀及无需转换的替换正文直接写入结果；verbatim 替换使用单一缓冲区，不再生成转换 EOL 的中间字符串。保留精确输出与完整语义验证；短时配对基准分别测量文本路径和路径段 API。
- 通过反向查找换行符定位诊断行，不再重复扫描文档前缀。用安全的八字节运算检查 JSON 字符串，整体复制无需转义的 UTF-8，专门处理需转义的 ASCII，并保留 Unicode encoder。按需分配 reopen-merge 子缓冲区和 seen 键存储。保留精确字节与坐标；增加分配及转义回归测试，并提供配对 runtime 基准，整个进程 batch 共用一个外部 15 秒限制。

### 文档

- 扩展 README 示例：点分路径与字面键、混合数组、String/Integer 区别、多行展开、逐字节 no-op 及 BOM/CRLF/EOF 保留。明确 0.9 字节诊断、安全拒绝、ABI 2 和独立纯实现用法；这些新增功能不属于已发布的 0.8.1。

