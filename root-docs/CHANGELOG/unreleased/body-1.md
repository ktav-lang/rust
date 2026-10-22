>>>>> lang=en
## [Unreleased]

### Added

- Rust-only source-editing API: `ktav::edit::edit_value` accepts an escaped dotted path, and `edit_value_segments` accepts a slice/vector of literal segments. Both replace an existing value without reformatting the document, preserve surrounding bytes and comments, and return a new `String`.
- Context-sensitive object keys and array indexes, `\.`/`\\` path escapes, minimal raw-marker transitions, unchanged-value no-ops and post-edit semantic verification. Typed `EditError` rejects malformed paths, missing targets and unsupported lossless replacements, including whole noncontiguous objects and structural changes that would discard comments.
- Exact-text regression tests and EN/RU/ZH usage examples. No syntax, existing parser/formatter API, C ABI version or package-version change; this feature is not part of the published 0.8.1 crate.

>>>>> lang=ru
## [Не выпущено]

### Добавлено

- Rust API редактирования исходного текста: `ktav::edit::edit_value` принимает экранированный dotted-путь, `edit_value_segments` — слайс/вектор буквальных сегментов. Обе функции заменяют существующее значение без форматирования документа, сохраняют окружающие байты и комментарии и возвращают новый `String`.
- Контекстное различение ключей объектов и индексов массивов, escape-последовательности пути `\.`/`\\`, минимальное переключение raw-маркера, неизменный текст при совпадении значений и семантическая проверка результата. Типизированный `EditError` отвергает ошибочные пути, отсутствующие цели и неподдерживаемые lossless-замены, включая целые noncontiguous-объекты и структурные правки с потерей комментариев.
- Регрессионные тесты точного текста и примеры EN/RU/ZH. Синтаксис, существующий API парсера/formatter, версия C ABI и версия пакета не меняются; функционал не входит в опубликованный crate 0.8.1.

>>>>> lang=zh
## [尚未发布]

### 新增

- Rust 源文本编辑 API：`ktav::edit::edit_value` 接受带转义的点分路径，`edit_value_segments` 接受字面路径段切片/向量。两者均在不重新格式化文档的情况下替换已有值，保留周围字节与注释，并返回新的 `String`。
- 按上下文区分对象键与数组索引，支持路径转义 `\.`/`\\`、最小化 raw 标记调整、值不变时保留原文，以及编辑后的语义验证。类型化的 `EditError` 拒绝错误路径、不存在的目标及不支持的无损替换，包括整个不连续对象和会丢失注释的结构修改。
- 精确文本回归测试及 EN/RU/ZH 用法示例。不改变语法、现有解析器/formatter API、C ABI 版本或包版本；此功能不包含在已发布的 0.8.1 crate 中。

