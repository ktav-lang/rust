>>>>> lang=en
## Source editing (unreleased)

This API is under development and is **not included in the published 0.8.1 crate**.
Unlike `format_str`, `ktav::edit` does not reformat the whole document: it replaces an existing value in the original text. Bytes outside the effective edit range remain unchanged, including comments, blank lines, key spelling, ordering, indentation, BOM and existing line endings. The range normally contains only the target; when readability requires expanding inline ancestors, it includes those wrappers. A raw marker is adjusted only when the new value requires it. The input is borrowed; success returns a new `String`, and an error leaves the input untouched.

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

### Preserving the source form

>>>>> lang=ru
## Редактирование исходного текста (ещё не выпущено)

API находится в разработке и **не входит в опубликованный crate 0.8.1**.
В отличие от `format_str`, `ktav::edit` не форматирует весь документ заново: заменяет существующее значение в исходном тексте. Байты вне фактического диапазона правки остаются неизменными, включая комментарии, пустые строки, написание ключей, порядок, отступы, BOM и существующие переводы строк. Обычно диапазон содержит только цель; если читаемость требует раскрытия inline-предков, он включает эти обёртки. Raw-маркер корректируется только при необходимости для нового значения. Вход заимствуется; успех возвращает новый `String`, ошибка не меняет исходник.

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

### Сохранение исходной формы

>>>>> lang=zh
## 源文本编辑（尚未发布）

此 API 正在开发中，**不包含在已发布的 0.8.1 crate 中**。
与 `format_str` 不同，`ktav::edit` 不会重新格式化整个文档，而是在原始文本中替换已有值。实际编辑范围之外的字节保持不变，包括注释、空行、键的写法、顺序、缩进、BOM 和原有换行符。通常只修改目标；若可读性要求展开行内祖先，编辑范围也包含这些外层容器。仅在新值需要时调整 raw 标记。输入以借用方式传入；成功时返回新的 `String`，出错时原文不变。

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

### 保留源文本形式

