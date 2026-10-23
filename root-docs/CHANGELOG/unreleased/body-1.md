>>>>> lang=en
## [Unreleased]

### Added

- Rust-only source-editing API: `ktav::edit::edit_value` accepts an escaped dotted path, and `edit_value_segments` accepts a slice/vector of literal segments. Both replace an existing value without reformatting the document, preserve surrounding bytes and comments, and return a new `String`.
- Context-sensitive object keys and array indexes, `\.`/`\\` path escapes, minimal raw-marker transitions, unchanged-value no-ops and post-edit semantic verification. Typed `EditError` rejects malformed paths, missing targets and unsupported lossless replacements, including whole noncontiguous objects and structural changes that would discard comments.
- Exact-text regression tests and EN/RU/ZH usage examples. No syntax, existing parser/formatter API, C ABI version or package-version change; this feature is not part of the published 0.8.1 crate.

### Performance

- Reserve the edited document's actual output length instead of retaining removed source capacity. No-op and missing-path edits skip the positional tree after validating the original document.
- Write nested inline replacements into one buffer; avoid temporary character vectors and a second copy of parsed textual path segments. Keep semantic verification and byte-preserving behavior. Add short paired benchmarks with separate compilation and an external 15-second process limit.

>>>>> lang=ru
## [Не выпущено]

### Добавлено

- Rust API редактирования исходного текста: `ktav::edit::edit_value` принимает экранированный dotted-путь, `edit_value_segments` — слайс/вектор буквальных сегментов. Обе функции заменяют существующее значение без форматирования документа, сохраняют окружающие байты и комментарии и возвращают новый `String`.
- Контекстное различение ключей объектов и индексов массивов, escape-последовательности пути `\.`/`\\`, минимальное переключение raw-маркера, неизменный текст при совпадении значений и семантическая проверка результата. Типизированный `EditError` отвергает ошибочные пути, отсутствующие цели и неподдерживаемые lossless-замены, включая целые noncontiguous-объекты и структурные правки с потерей комментариев.
- Регрессионные тесты точного текста и примеры EN/RU/ZH. Синтаксис, существующий API парсера/formatter, версия C ABI и версия пакета не меняются; функционал не входит в опубликованный crate 0.8.1.

### Производительность

- Буфер результата резервируется по фактической длине, без удержания памяти удалённого исходного значения. No-op и отсутствующий путь обходятся без позиционного дерева после проверки исходного документа.
- Вложенные inline-замены пишутся в один буфер, без временного вектора символов и повторного копирования разобранных сегментов текстового пути. Семантическая проверка и сохранение исходных байтов остаются. Добавлены короткие парные benchmarks с отдельной компиляцией и внешним лимитом 15 секунд на процесс.

>>>>> lang=zh
## [尚未发布]

### 新增

- Rust 源文本编辑 API：`ktav::edit::edit_value` 接受带转义的点分路径，`edit_value_segments` 接受字面路径段切片/向量。两者均在不重新格式化文档的情况下替换已有值，保留周围字节与注释，并返回新的 `String`。
- 按上下文区分对象键与数组索引，支持路径转义 `\.`/`\\`、最小化 raw 标记调整、值不变时保留原文，以及编辑后的语义验证。类型化的 `EditError` 拒绝错误路径、不存在的目标及不支持的无损替换，包括整个不连续对象和会丢失注释的结构修改。
- 精确文本回归测试及 EN/RU/ZH 用法示例。不改变语法、现有解析器/formatter API、C ABI 版本或包版本；此功能不包含在已发布的 0.8.1 crate 中。

### 性能

- 按编辑后实际长度预留输出缓冲区，不再保留已删除源值占用的容量。验证原文后，值不变或路径不存在时不再构建位置树。
- 将嵌套行内替换直接写入同一缓冲区，避免临时字符向量及文本路径段的二次复制。保留语义验证和原始字节保护。新增短时配对基准，编译独立进行，每个进程由外部工具限制为 15 秒。

