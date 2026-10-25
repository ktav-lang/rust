>>>>> lang=en
### Diagnostics and safe failures in 0.9

Source coordinates are UTF-8 byte offsets, including a BOM; spans are half-open `[start, end)`, and lines are 1-based. LF, lone CR and CRLF are handled, with CRLF counted as one terminator. A Unicode character may occupy several bytes.

An unfinished `a: [\n` reports `UnclosedCompound`, line 1 and span `3..5`: the span begins at the innermost unmatched opener, not the start of its key or line. For raw bytes `61 3A 20 FF` (`a: ` followed by invalid UTF-8), validation reports `InvalidUtf8`, line 1 and a covering span `3..4`; `line_text` is null rather than fabricated by lossy decoding. UTF-8 validation precedes grammar processing.

A malformed textual path, including an empty textual path or empty textual segment, is `EditInvalidPath`; its offset is measured in the path's UTF-8 bytes. An empty segments list, missing key, out-of-range index or scalar descent is `EditNotFound`. Object numeric segments remain keys; array indexes use ASCII digits, allow leading zeros and reject signs.

Whole dotted/reopened intermediate objects may occupy several physical ranges and are `EditUnsupported`; their existing leaves can still be edited. Replacing a container must not discard interior comments. No failed call returns a partial patch or changes the supplied source. The returned text is re-parsed and semantically checked: the target must equal the replacement and the rest of the value tree must remain equal.

For bindings, invalid source is reported with the ordinary parser envelope before path validation. Malformed wire payloads are `Message` errors without invented source coordinates. The wire envelope has `error`, `reason`, `line`, `line_text`, `span`, `path`, `body`, `canonical`, `spec_section` and `message`; host accessor names are documented in the errors section. `body` carries the decimal path-byte offset for `EditInvalidPath`, null for `EditNotFound`, and the refusal reason for `EditUnsupported`.

The coordinate examples describe ordinary parse errors (`Error` / `ErrorEnvelope`). Rust source editing itself returns `EditError::{InvalidPath, NotFound, Unsupported}`: `InvalidPath { offset }` refers to the path bytes, and an invalid source is wrapped as `Unsupported`. The C ABI validates the source first and exposes the ordinary source envelope to bindings. These Rust variants are not the wire envelope field layout.

>>>>> lang=ru
### Диагностика и безопасные отказы в 0.9

Координаты исходника — байтовые смещения UTF-8 с учётом BOM; spans полуоткрыты `[start, end)`, строки нумеруются с 1. Поддерживаются LF, отдельный CR и CRLF; CRLF считается одним переводом строки. Один Unicode-символ может занимать несколько байтов.

Незавершённый `a: [\n` даёт `UnclosedCompound`, строку 1 и span `3..5`: начало — самая внутренняя незакрытая скобка, а не начало ключа или строки. Для сырых байтов `61 3A 20 FF` (`a: ` и невалидный UTF-8) валидация сообщает `InvalidUtf8`, строку 1 и покрывающий span `3..4`; `line_text` равен null, а не придуманному тексту после lossy-декодирования. Проверка UTF-8 предшествует грамматике.

Некорректный текстовый путь, включая пустой путь или пустой текстовый сегмент, даёт `EditInvalidPath`; смещение считается в байтах UTF-8 самого пути. Пустой список сегментов, отсутствующий ключ, индекс вне диапазона или спуск через скаляр дают `EditNotFound`. В объекте числовой сегмент остаётся ключом; индекс массива содержит ASCII-цифры, допускает ведущие нули, но не знаки.

Целые промежуточные объекты из dotted/reopened-определений могут занимать несколько физических диапазонов и давать `EditUnsupported`; их существующие листья остаются редактируемыми. Структурная замена контейнера не должна удалять внутренние комментарии. Ошибка не возвращает частичный патч и не меняет переданный исходник. Результат повторно разбирается и семантически проверяется: цель должна равняться замене, остальное дерево значений — исходному.

В привязках ошибочный исходник даёт обычный конверт парсера до проверки пути. Некорректный wire-payload — ошибка `Message` без выдуманных координат. Wire-конверт содержит `error`, `reason`, `line`, `line_text`, `span`, `path`, `body`, `canonical`, `spec_section`, `message`; имена host-accessors описаны в разделе ошибок. В `body` находятся десятичное байтовое смещение пути для `EditInvalidPath`, null для `EditNotFound` и причина отказа для `EditUnsupported`.

Примеры координат относятся к обычным ошибкам разбора (`Error` / `ErrorEnvelope`). Сам Rust API правки возвращает `EditError::{InvalidPath, NotFound, Unsupported}`: `InvalidPath { offset }` относится к байтам пути, невалидный исходник оборачивается в `Unsupported`. C ABI сначала проверяет исходник и передаёт привязкам обычный конверт исходной ошибки. Rust-варианты не являются структурой полей wire-конверта.

>>>>> lang=zh
### 0.9 的诊断与安全失败

源码坐标为包含 BOM 的 UTF-8 字节偏移，span 使用半开区间 `[start, end)`，行号从 1 开始。支持 LF、独立 CR 和 CRLF，且 CRLF 计为一个换行。一个 Unicode 字符可能占用多个字节。

未完成的 `a: [\n` 报告 `UnclosedCompound`、第 1 行和 span `3..5`：区间从最内层未闭合的起始括号开始，而非键或行的开头。原始字节 `61 3A 20 FF`（`a: ` 后接无效 UTF-8）报告 `InvalidUtf8`、第 1 行及覆盖区间 `3..4`；`line_text` 为 null，不会通过有损解码伪造文本。UTF-8 校验优先于语法处理。

非法文本路径（包括空路径或空文本段）报告 `EditInvalidPath`，偏移按路径本身的 UTF-8 字节计算。空路径段列表、缺失键、索引越界或穿过标量下钻报告 `EditNotFound`。对象中的数字段仍是键；数组索引仅接受 ASCII 数字，允许前导零，不允许正负号。

由 dotted/reopened 定义拼成的整个中间对象可能占据多个物理范围，因此报告 `EditUnsupported`；已有叶子值仍可修改。容器结构替换不能删除内部注释。失败调用不会返回部分补丁，也不会修改传入的源码。返回文本会重新解析并进行语义检查：目标必须等于替换值，其余值树必须与原文相等。

在绑定中，无效源码会先于路径验证报告普通解析错误信封。非法 wire 载荷是无虚构源码坐标的 `Message` 错误。wire 信封包含 `error`、`reason`、`line`、`line_text`、`span`、`path`、`body`、`canonical`、`spec_section`、`message`；宿主语言的访问器名称见错误章节。`body` 分别承载 `EditInvalidPath` 的十进制路径字节偏移、`EditNotFound` 的 null，以及 `EditUnsupported` 的拒绝原因。

坐标示例描述普通解析错误（`Error` / `ErrorEnvelope`）。Rust 编辑 API 本身返回 `EditError::{InvalidPath, NotFound, Unsupported}`：`InvalidPath { offset }` 指向路径字节，无效源码包装为 `Unsupported`。C ABI 会先验证源码，并向绑定公开普通源码错误信封。这些 Rust 变体不是 wire 信封的字段布局。

