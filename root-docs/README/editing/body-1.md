>>>>> lang=en
## Source editing (unreleased)

This API is under development and is **not included in the published 0.8.1 crate**.
Unlike `format_str`, `ktav::edit` does not reformat the document: it replaces an existing value in the original text. Comments, blank lines, key spelling, ordering, indentation, BOM, existing line endings and bytes outside the replacement remain unchanged. A raw marker is adjusted only when the new value requires it. The input is borrowed; success returns a new `String`, and an error leaves the input untouched.

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;

let source = "## top\n\na: 1\n\n## end\n";
let replacement = Value::Integer("2".into());
let expected = "## top\n\na: 2\n\n## end\n";
assert_eq!(edit_value(source, "a", &replacement)?, expected);
assert_eq!(edit_value_segments(source, &["a"], &replacement)?, expected);
let path = vec![String::from("a")];
assert_eq!(edit_value_segments(source, &path, &replacement)?, expected);
```

The textual path `key1.key2.0.key4` uses `.` as a separator. Escape a literal dot as `\.` and a backslash as `\\`; other escapes, an unfinished escape and empty textual segments are errors. For a literal key `a.b`, the Rust path argument is `r"a\.b"` or `"a\\.b"`. The slice/vector API takes decoded names (`&["a.b"]`), without path escaping. In an object, `0` means the key `"0"`; in an array, it means index zero. Array indexes contain only ASCII digits; leading zeroes are accepted, signs are not.

Only existing values can be replaced: no automatic container creation, removal or renaming. An unchanged value returns the original text, including its original numeric spelling. New values are supplied as `Value`; a numeric-looking `Value::String` stays a string, using a raw marker where necessary.

`EditError::InvalidPath` reports a malformed textual path and byte offset; `NotFound` covers missing keys, invalid/out-of-range array indexes and descent through a scalar. `Unsupported` rejects an invalid source, an unrepresentable replacement or a replacement for which this implementation cannot preserve the surrounding text. In particular, a whole object assembled from separate dotted/reopened definitions has no single replacement range, and replacing a container must not delete its internal comments. Its individual leaves remain editable. Inline values use escapes for CR and LF. Outside inline compounds, CR-containing strings and some context-dependent multiline/container replacements are rejected rather than silently rewritten. Every returned patch is parsed and checked against the intended value tree; errors never return a partially edited document.

>>>>> lang=ru
## Редактирование исходного текста (ещё не выпущено)

API находится в разработке и **не входит в опубликованный crate 0.8.1**.
В отличие от `format_str`, `ktav::edit` не форматирует документ заново: заменяет существующее значение в исходном тексте. Комментарии, пустые строки, написание ключей, порядок, отступы, BOM, существующие переводы строк и байты вне замены остаются неизменными. Raw-маркер корректируется только при необходимости для нового значения. Вход заимствуется; успех возвращает новый `String`, ошибка не меняет исходник.

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;

let source = "## top\n\na: 1\n\n## end\n";
let replacement = Value::Integer("2".into());
let expected = "## top\n\na: 2\n\n## end\n";
assert_eq!(edit_value(source, "a", &replacement)?, expected);
assert_eq!(edit_value_segments(source, &["a"], &replacement)?, expected);
let path = vec![String::from("a")];
assert_eq!(edit_value_segments(source, &path, &replacement)?, expected);
```

Текстовый путь `key1.key2.0.key4` использует `.` как разделитель. Буквальная точка экранируется как `\.`, обратный слеш — как `\\`; другие escape-последовательности, незавершённый escape и пустые текстовые сегменты ошибочны. Для буквального ключа `a.b` аргумент пути в Rust записывается как `r"a\.b"` или `"a\\.b"`. API слайса/вектора принимает декодированные имена (`&["a.b"]`), без экранирования пути. В объекте `0` означает ключ `"0"`, в массиве — индекс ноль. Индексы содержат только ASCII-цифры; ведущие нули допустимы, знаки — нет.

Поддерживается только замена существующих значений: без автоматического создания контейнеров, удаления и переименования. Совпадающее значение возвращает исходный текст, включая первоначальное написание чисел. Новые значения передаются как `Value`; похожий на число `Value::String` остаётся строкой, при необходимости с raw-маркером.

`EditError::InvalidPath` сообщает об ошибочном текстовом пути и смещении в байтах; `NotFound` — об отсутствующем ключе, неверном/выходящем за границы индексе или спуске через скаляр. `Unsupported` отвергает невалидный исходник, непредставимую замену либо замену, для которой реализация не может сохранить окружающий текст. В частности, у целого объекта из отдельных dotted/reopened-определений нет единого диапазона замены, а замена контейнера не должна удалять внутренние комментарии. Отдельные листья такого объекта остаются редактируемыми. В inline-значениях CR и LF передаются escape-последовательностями. Вне inline-контейнеров строки с CR и некоторые контекстно-зависимые многострочные/контейнерные замены отвергаются, а не переписываются молча. Каждый возвращаемый результат разбирается и сверяется с ожидаемым деревом значений; ошибка не возвращает частично изменённый документ.

>>>>> lang=zh
## 源文本编辑（尚未发布）

此 API 正在开发中，**不包含在已发布的 0.8.1 crate 中**。
与 `format_str` 不同，`ktav::edit` 不会重新格式化文档，而是在原始文本中替换已有值。注释、空行、键的写法、顺序、缩进、BOM、原有换行符以及替换范围之外的字节保持不变。仅在新值需要时调整 raw 标记。输入以借用方式传入；成功时返回新的 `String`，出错时原文不变。

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;

let source = "## top\n\na: 1\n\n## end\n";
let replacement = Value::Integer("2".into());
let expected = "## top\n\na: 2\n\n## end\n";
assert_eq!(edit_value(source, "a", &replacement)?, expected);
assert_eq!(edit_value_segments(source, &["a"], &replacement)?, expected);
let path = vec![String::from("a")];
assert_eq!(edit_value_segments(source, &path, &replacement)?, expected);
```

文本路径 `key1.key2.0.key4` 使用 `.` 分隔各段。键名中的点写作 `\.`，反斜杠写作 `\\`；其他转义、未完成的转义和空文本段均为错误。对于字面键名 `a.b`，Rust 路径参数写作 `r"a\.b"` 或 `"a\\.b"`。切片/向量 API 接受已解码的名称（`&["a.b"]`），无需路径转义。在对象中，`0` 表示键 `"0"`；在数组中，它表示索引零。数组索引只能包含 ASCII 数字，允许前导零，不允许正负号。

仅支持替换已有值，不会自动创建容器，也不支持删除或重命名。若值未改变，则返回原始文本，包括原有的数字写法。新值以 `Value` 传入；看起来像数字的 `Value::String` 仍是字符串，必要时使用 raw 标记。

`EditError::InvalidPath` 报告文本路径错误及字节偏移；`NotFound` 表示键不存在、数组索引无效或越界，或试图穿过标量继续访问。`Unsupported` 拒绝无效源文档、无法表示的新值，或当前实现无法在保留周围文本的前提下完成的替换。例如，由分散的点分键或重新打开的定义构成的整个对象没有单一替换范围；替换容器也不能删除其内部注释，但其中的叶子值仍可编辑。行内值使用转义表示 CR 和 LF。在行内复合值之外，含 CR 的字符串及部分依赖上下文的多行/容器替换会被拒绝，而不会被静默改写。所有成功返回的文本都经过解析，并与预期值树比较；错误不会返回部分编辑的文档。

