>>>>> lang=en
## [Unreleased]

### Changed

- **Breaking in 0.9.0:** `InvalidUtf8` gains `line`/`span`; `UnclosedCompound` gains the opener's `line`. Downstream constructors and exhaustive field patterns must migrate. Direct Rust errors, public envelopes and C ABI source operations share raw-byte locations; UTF-8 spans are non-empty and EOF spans cover the innermost opener through EOF. LF, lone CR, CRLF and BOM offsets are preserved, including `Span::line_col` and envelope `line_text`. The ten-field JSON schema and I/O/writer location rules are unchanged. New regression oracles live only in the draft spec `versions/0.9/tests` and run across source entry points; CI refuses to skip a missing corpus. Version 0.9.0 is in preparation, not published.

### Added

- Rust-only source-editing API: `ktav::edit::edit_value` accepts an escaped dotted path, and `edit_value_segments` accepts a slice/vector of literal segments. Both replace an existing value without reformatting the document, preserve surrounding bytes and comments, and return a new `String`.
- Context-sensitive object keys and array indexes, `\.`/`\\` path escapes, minimal raw-marker transitions, unchanged-value no-ops and post-edit semantic verification. Typed `EditError` rejects malformed paths, missing targets and unsupported lossless replacements, including whole noncontiguous objects and structural changes that would discard comments.
- Exact-text regression tests and EN/RU/ZH usage examples. No syntax, existing parser/formatter API, C ABI version or package-version change; this feature is not part of the published 0.8.1 crate.

### Performance

- Reserve the edited document's actual output length instead of retaining removed source capacity. No-op and missing-path edits skip the positional tree after validating the original document.
- Write nested inline replacements into one buffer; avoid temporary character vectors and a second copy of parsed textual path segments. Keep semantic verification and byte-preserving behavior. Add short paired benchmarks with separate compilation and an external 15-second process limit.
- The positional pass shares structural classifiers with the parser without constructing unused values or multiline bodies. Verify the edited target and untouched tree without cloning the replacement; render borrowed block compounds without a temporary root wrapper. Preserve validation, writer error paths and byte-for-byte output, with focused regression tests and expanded paired benchmarks.
- Borrow literal path slices and unescaped textual segments instead of copying each segment. Assemble raw-marker prefixes and unchanged replacement bodies directly into the result; render verbatim replacements in one buffer without an intermediate EOL-converted string. Preserve exact output and full semantic verification; measure text and segment paths separately in short paired benchmarks.

>>>>> lang=ru
## [Не выпущено]

### Изменено

- **Breaking в 0.9.0:** в `InvalidUtf8` добавлены `line`/`span`, в `UnclosedCompound` — `line` открывающего маркера. Downstream-конструкторы и patterns, перечисляющие все поля, требуют миграции. Прямые Rust-ошибки, публичный envelope и C ABI source operations используют общие координаты исходных байтов; UTF-8 spans непустые, EOF spans охватывают самый внутренний незакрытый opener до EOF. LF, одиночный CR, CRLF и BOM учитываются без нормализации, в том числе в `Span::line_col` и envelope `line_text`. Схема JSON из десяти полей и отсутствие позиций у I/O/writer не меняются. Новые regression oracles находятся только в draft спеки `versions/0.9/tests` и проверяются через source entry points; CI запрещает пропуск отсутствующего corpus. Версия 0.9.0 готовится, но не опубликована.

### Добавлено

- Rust API редактирования исходного текста: `ktav::edit::edit_value` принимает экранированный dotted-путь, `edit_value_segments` — слайс/вектор буквальных сегментов. Обе функции заменяют существующее значение без форматирования документа, сохраняют окружающие байты и комментарии и возвращают новый `String`.
- Контекстное различение ключей объектов и индексов массивов, escape-последовательности пути `\.`/`\\`, минимальное переключение raw-маркера, неизменный текст при совпадении значений и семантическая проверка результата. Типизированный `EditError` отвергает ошибочные пути, отсутствующие цели и неподдерживаемые lossless-замены, включая целые noncontiguous-объекты и структурные правки с потерей комментариев.
- Регрессионные тесты точного текста и примеры EN/RU/ZH. Синтаксис, существующий API парсера/formatter, версия C ABI и версия пакета не меняются; функционал не входит в опубликованный crate 0.8.1.

### Производительность

- Буфер результата резервируется по фактической длине, без удержания памяти удалённого исходного значения. No-op и отсутствующий путь обходятся без позиционного дерева после проверки исходного документа.
- Вложенные inline-замены пишутся в один буфер, без временного вектора символов и повторного копирования разобранных сегментов текстового пути. Семантическая проверка и сохранение исходных байтов остаются. Добавлены короткие парные benchmarks с отдельной компиляцией и внешним лимитом 15 секунд на процесс.
- Позиционный проход использует общие с parser структурные классификаторы без построения ненужных значений и multiline-тел. Проверка цели и неизменной части дерева обходится без клонирования replacement; блочные контейнеры рендерятся по заимствованию без временной корневой обёртки. Валидация, пути writer-ошибок и точные выходные байты сохранены; добавлены регрессии и расширены парные benchmarks.
- Буквальные сегменты пути и сегменты текстового пути без escape заимствуются без копирования каждого сегмента. Префиксы raw-маркера и тела замен, не требующие преобразования, переносятся непосредственно в результат; verbatim-замены рендерятся в один буфер без промежуточной строки с преобразованными EOL. Точные выходные байты и полная семантическая проверка сохранены; короткие парные benchmarks отдельно измеряют text- и segments-пути.

>>>>> lang=zh
## [尚未发布]

### 变更

- **0.9.0 破坏性变更：** `InvalidUtf8` 新增 `line`/`span`，`UnclosedCompound` 新增起始标记的 `line`。下游构造器及枚举全部字段的模式需要迁移。直接 Rust 错误、公开 envelope 和 C ABI 源操作共用原始字节坐标；UTF-8 区间非空，EOF 区间覆盖最内层未闭合起始标记至 EOF。LF、单独 CR、CRLF 及 BOM 均不经规范化，`Span::line_col` 和 envelope 的 `line_text` 同样遵守该规则。十字段 JSON 格式及 I/O/writer 无源位置的规则不变。新增回归 oracle 仅位于规范草案 `versions/0.9/tests`，由各源入口执行；CI 不允许因语料缺失而跳过。0.9.0 尚在准备中，未发布。

### 新增

- Rust 源文本编辑 API：`ktav::edit::edit_value` 接受带转义的点分路径，`edit_value_segments` 接受字面路径段切片/向量。两者均在不重新格式化文档的情况下替换已有值，保留周围字节与注释，并返回新的 `String`。
- 按上下文区分对象键与数组索引，支持路径转义 `\.`/`\\`、最小化 raw 标记调整、值不变时保留原文，以及编辑后的语义验证。类型化的 `EditError` 拒绝错误路径、不存在的目标及不支持的无损替换，包括整个不连续对象和会丢失注释的结构修改。
- 精确文本回归测试及 EN/RU/ZH 用法示例。不改变语法、现有解析器/formatter API、C ABI 版本或包版本；此功能不包含在已发布的 0.8.1 crate 中。

### 性能

- 按编辑后实际长度预留输出缓冲区，不再保留已删除源值占用的容量。验证原文后，值不变或路径不存在时不再构建位置树。
- 将嵌套行内替换直接写入同一缓冲区，避免临时字符向量及文本路径段的二次复制。保留语义验证和原始字节保护。新增短时配对基准，编译独立进行，每个进程由外部工具限制为 15 秒。
- 位置扫描与解析器共用结构分类，不再构造无用值或多行正文。验证目标和未修改的树时不再克隆替换值；以借用方式渲染块容器，避免临时根包装。保留验证、writer 错误路径和逐字节输出，并增加专项回归测试及配对基准场景。
- 借用字面路径段及不含转义的文本路径段，不再逐段复制。将 raw 标记前缀及无需转换的替换正文直接写入结果；verbatim 替换使用单一缓冲区，不再生成转换 EOL 的中间字符串。保留精确输出与完整语义验证；短时配对基准分别测量文本路径和路径段 API。

