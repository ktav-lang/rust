>>>>> lang=en
## The C ABI for other languages

The Go, Java, PHP, C#, JS and Python bindings share the Rust core. The off-by-default `cabi` feature provides wire decoding, document operations and error encoding; one macro call in a binding's `cdylib` exports the C symbols. Direct Rust bridges such as PyO3 and N-API can use the same safe operations without another FFI layer.

```text
// crates/cabi/src/lib.rs of a binding:
ktav::declare_cabi!();
```

The **unreleased 0.9 preparation uses ABI 2**, with twelve symbols: nine document functions (`ktav_loads`, `ktav_loads_strict`, `ktav_dumps`, `ktav_dumps_force_strings`, `ktav_emit_canonical`, `ktav_format`, `ktav_canonical_from_source`, `ktav_edit_value`, `ktav_edit_value_segments`), `ktav_free`, `ktav_version` and `ktav_abi_version`. Hosts must reject an ABI-1 binary rather than silently falling back to a different backend. This does not publish or change a binding package version.

Both edit functions take three length-delimited byte inputs: Ktav source, path and replacement JSON wire value. The first path is UTF-8 dotted text; the second is a JSON array of literal decoded strings. Tagged numeric values (`$i` / `$f`) preserve scalar types and precision; the replacement may itself be a scalar. Success returns UTF-8 Ktav source, not a JSON string, using the same lossless editing and readability rules as `ktav::edit`.

Every error uses the existing ten-field JSON envelope. Source diagnostics retain the 0.9 raw-byte locations. Edit failures use `EditInvalidPath`, `EditNotFound` or `EditUnsupported`, without invented source positions: `body` carries the decimal path byte offset for `EditInvalidPath`, the reason for `EditUnsupported`, or null for `EditNotFound`. Transport JSON/UTF-8 errors outside the source use `Message`; the host surfaces `message` as its error text.

The full signatures, ownership contract and artifact naming convention (`ktav_cabi-windows-amd64.dll`, `libktav_cabi-darwin-arm64.dylib`, `libktav_cabi-linux-amd64.so`, `$KTAV_LIB_PATH`) are specified in [docs/CABI.md](docs/CABI.md).

>>>>> lang=ru
## C ABI для других языков

Биндинги Go, Java, PHP, C#, JS и Python используют общее Rust-ядро. Выключенная по умолчанию фича `cabi` предоставляет wire-декодирование, операции над документами и кодирование ошибок; один макровызов в `cdylib` биндинга экспортирует C-символы. Прямые Rust-мосты, например PyO3 и N-API, могут использовать те же безопасные операции без дополнительного FFI-слоя.

```text
// crates/cabi/src/lib.rs биндинга:
ktav::declare_cabi!();
```

**Подготовка ещё не опубликованной 0.9 использует ABI 2** с двенадцатью символами: девять функций над документами (`ktav_loads`, `ktav_loads_strict`, `ktav_dumps`, `ktav_dumps_force_strings`, `ktav_emit_canonical`, `ktav_format`, `ktav_canonical_from_source`, `ktav_edit_value`, `ktav_edit_value_segments`), `ktav_free`, `ktav_version` и `ktav_abi_version`. Хост обязан отклонить бинарник ABI 1, а не молча переключаться на другой backend. Это не публикует и не меняет версию пакета биндинга.

Обе edit-функции принимают три массива байтов с длинами: Ktav-исходник, путь и replacement в JSON wire-формате. Первый путь — UTF-8 dotted-текст, второй — JSON-массив буквальных декодированных строк. Tagged-числа (`$i` / `$f`) сохраняют типы скаляров и точность; сама замена может быть скаляром. Успех возвращает Ktav-исходник UTF-8, не JSON-строку, с теми же правилами lossless-редактирования и читаемости, что у `ktav::edit`.

Каждая ошибка использует прежний JSON-envelope из десяти полей. Диагностики исходника сохраняют координаты сырых байтов 0.9. Ошибки редактирования — `EditInvalidPath`, `EditNotFound` или `EditUnsupported`, без выдуманных source positions: `body` содержит десятичное byte offset пути для `EditInvalidPath`, причину для `EditUnsupported` либо null для `EditNotFound`. Ошибки JSON/UTF-8 транспорта вне исходника используют `Message`; текст ошибки хост берёт из `message`.

Полные сигнатуры, контракт владения и конвенция имён артефактов (`ktav_cabi-windows-amd64.dll`, `libktav_cabi-darwin-arm64.dylib`, `libktav_cabi-linux-amd64.so`, `$KTAV_LIB_PATH`) описаны в [docs/CABI.md](docs/CABI.md).

>>>>> lang=zh
## 面向其他语言的 C ABI

Go、Java、PHP、C#、JS 和 Python 绑定共用 Rust 核心。默认关闭的 `cabi` feature 提供 wire 解码、文档操作和错误编码；绑定的 `cdylib` 中一次宏调用即可导出 C 符号。PyO3 和 N-API 等直接 Rust 桥接可以调用相同的安全操作，无需增加 FFI 层。

```text
// 绑定的 crates/cabi/src/lib.rs：
ktav::declare_cabi!();
```

**尚未发布的 0.9 准备版本使用 ABI 2**，包含十二个符号：九个文档函数（`ktav_loads`、`ktav_loads_strict`、`ktav_dumps`、`ktav_dumps_force_strings`、`ktav_emit_canonical`、`ktav_format`、`ktav_canonical_from_source`、`ktav_edit_value`、`ktav_edit_value_segments`）、`ktav_free`、`ktav_version` 和 `ktav_abi_version`。宿主必须拒绝 ABI 1 二进制文件，不得静默切换后端。这不会发布或更改绑定包版本。

两个编辑函数均接受三个带长度的字节输入：Ktav 源文档、路径及 JSON wire 替换值。第一个路径为 UTF-8 点分文本；第二个为字面解码字符串的 JSON 数组。带标记的数字（`$i` / `$f`）保留标量类型和精度；替换值本身可以是标量。成功返回 UTF-8 Ktav 源文本，而非 JSON 字符串，遵守与 `ktav::edit` 相同的无损编辑和可读性规则。

所有错误仍使用十字段 JSON envelope。源诊断保留 0.9 原始字节坐标。编辑错误为 `EditInvalidPath`、`EditNotFound` 或 `EditUnsupported`，不编造源位置：`body` 对应十进制路径字节偏移、拒绝原因或 null。源文档之外的 JSON/UTF-8 传输错误使用 `Message`；宿主以 `message` 作为错误文本。

完整签名、所有权及产物命名约定（`ktav_cabi-windows-amd64.dll`、`libktav_cabi-darwin-arm64.dylib`、`libktav_cabi-linux-amd64.so`、`$KTAV_LIB_PATH`）见 [docs/CABI.md](docs/CABI.md)。

