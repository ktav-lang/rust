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

Dotted keys stay dotted, inline objects/arrays stay inline, and block objects/arrays stay blocks, including when emptied. This applies to deep edits inside arrays too. Replacing a scalar with a new container uses a block outside an inline parent; replacing a whole inline container preserves its inline form when readable. A type change does not by itself reformat ancestors. No-op edits bypass all readability rules.

An edited inline line is expanded only when the replacement contains a multiline string, the line grows beyond `max(100, original line length)`, or a replacement string needs at least four inline escapes with at least 25% extra encoded characters. Length means Unicode scalar values, not UTF-8 bytes or terminal columns; the escape overhead is compared with the decoded string length. Existing long lines are not expanded unless they grow.

Expansion opens the containing inline wrapper and only the affected nested wrappers still needing expansion; untouched nested branches may stay inline. Key spelling and order are preserved. Moved scalar spellings and raw markers change only when standalone syntax needs a different encoding. New block levels use four spaces and the source EOL style; no final newline is added. If a lossless standalone encoding is unavailable, for example because a string contains CR or a line equal to `))`, the valid inline representation is retained instead. Readability never authorizes comment loss or semantic changes. These fixed rules add no configuration mode.

A short paired benchmark lives in `benches/source_edit.rs`. Build it separately with `cargo bench --bench source_edit --no-run --message-format=json`, then run the reported executable with `baseline` or `candidate` as its argument. Apply an external 15-second process timeout (for example, `caps 15 <benchmark-executable> candidate` on Windows). Each case uses a 20 ms warmup and a 300 ms sample; setup and exact-output validation are outside the measured interval. Compare alternating A/B and B/A runs on the same toolchain; short-run wall-clock results are not a CI regression gate.

Only existing values can be replaced: no automatic container creation, removal or renaming. An unchanged value returns the original text, including its original numeric spelling. New values are supplied as `Value`; a numeric-looking `Value::String` stays a string, using a raw marker where necessary.

`EditError::InvalidPath` reports a malformed textual path and byte offset; `NotFound` covers missing keys, invalid/out-of-range array indexes and descent through a scalar. `Unsupported` rejects an invalid source, an unrepresentable replacement or a replacement for which this implementation cannot preserve the surrounding text. In particular, a whole object assembled from separate dotted/reopened definitions has no single replacement range, and replacing a container must not delete its internal comments. Its individual leaves remain editable. Inline values use escapes for CR and LF. Outside inline compounds, CR-containing strings and some context-dependent multiline/container replacements are rejected rather than silently rewritten. Every returned patch is parsed and checked against the intended value tree; errors never return a partially edited document.

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

Dotted-ключи остаются dotted, однострочные объекты/массивы — однострочными, блочные — блочными, в том числе при опустошении. Это относится и к глубоким правкам внутри массивов. При замене скаляра новым контейнером вне inline-родителя используется блок; при замене целого inline-контейнера сохраняется его однострочная форма, если она читаема. Смена типа сама по себе не форматирует предков. No-op обходит все критерии читаемости.

Изменённая inline-строка раскрывается только при многострочном тексте в замене, росте длины сверх `max(100, исходная длина строки)` либо если строке в замене требуется не менее четырёх inline-экранирований с увеличением числа символов хотя бы на 25%. Длина считается в Unicode scalar values, не в байтах UTF-8 и не в колонках терминала; накладные расходы экранирования сравниваются с длиной декодированной строки. Уже длинная строка без роста не раскрывается.

Раскрывается содержащая inline-обёртка и только затронутые вложенные обёртки, которым всё ещё требуется раскрытие; нетронутые вложенные ветви могут остаться однострочными. Написание и порядок ключей сохраняются. Написание переносимых скаляров и raw-маркеры меняются только при необходимости другого кодирования вне inline. Новые уровни блока используют четыре пробела и EOL исходника; финальный перевод строки не добавляется. Если lossless-кодирование вне inline невозможно, например из-за CR или строки `))` в данных, сохраняется корректное inline-представление. Читаемость не разрешает потерю комментариев или изменение семантики. Эти фиксированные правила не добавляют режимов настройки.

Короткий парный benchmark находится в `benches/source_edit.rs`. Сначала отдельно соберите его командой `cargo bench --bench source_edit --no-run --message-format=json`, затем запустите указанный в выводе исполняемый файл с аргументом `baseline` или `candidate`. Ограничьте процесс внешним timeout 15 секунд, например `caps 15 <benchmark-executable> candidate` в Windows. Каждый сценарий использует прогрев 20 мс и замер 300 мс; подготовка и проверка точного результата не входят в измеряемый интервал. Чередуйте A/B и B/A на одном toolchain; время коротких прогонов не служит порогом регрессии в CI.

Поддерживается только замена существующих значений: без автоматического создания контейнеров, удаления и переименования. Совпадающее значение возвращает исходный текст, включая первоначальное написание чисел. Новые значения передаются как `Value`; похожий на число `Value::String` остаётся строкой, при необходимости с raw-маркером.

`EditError::InvalidPath` сообщает об ошибочном текстовом пути и смещении в байтах; `NotFound` — об отсутствующем ключе, неверном/выходящем за границы индексе или спуске через скаляр. `Unsupported` отвергает невалидный исходник, непредставимую замену либо замену, для которой реализация не может сохранить окружающий текст. В частности, у целого объекта из отдельных dotted/reopened-определений нет единого диапазона замены, а замена контейнера не должна удалять внутренние комментарии. Отдельные листья такого объекта остаются редактируемыми. В inline-значениях CR и LF передаются escape-последовательностями. Вне inline-контейнеров строки с CR и некоторые контекстно-зависимые многострочные/контейнерные замены отвергаются, а не переписываются молча. Каждый возвращаемый результат разбирается и сверяется с ожидаемым деревом значений; ошибка не возвращает частично изменённый документ.

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

点分键仍为点分键，行内对象/数组仍为行内形式，块对象/数组仍为块形式，包括清空容器时。数组深层编辑也遵守此规则。在行内父容器之外用新容器替换标量时使用块；替换整个行内容器时，只要可读就保留行内形式。类型变化本身不触发祖先格式化。值不变时跳过所有可读性规则。

仅在替换值包含多行字符串、修改后行长增长并超过 `max(100, 原始行长)`，或替换字符串需要至少四个行内转义且编码字符数增加至少 25% 时展开。长度按 Unicode 标量值计算，而非 UTF-8 字节或终端列；转义开销相对于解码后字符串长度计算。原本已很长但没有增长的行不会展开。

展开所在的行内外层容器，并仅继续展开仍需展开的已修改嵌套容器；未修改的嵌套分支可以保留行内形式。保留键的写法和顺序。仅当独立行语法需要不同编码时，才改变移位标量的写法和 raw 标记。新块层级使用四个空格及原文的 EOL 风格，不添加末尾换行。若无法无损转换为独立行，例如字符串包含 CR 或一整行 `))`，则保留有效的行内表示。可读性不能成为删除注释或改变语义的理由。这些固定规则不增加配置模式。

短时配对基准位于 `benches/source_edit.rs`。先使用 `cargo bench --bench source_edit --no-run --message-format=json` 单独编译，再以 `baseline` 或 `candidate` 参数运行输出中列出的可执行文件。用外部工具将每个进程限制为 15 秒，例如 Windows 下的 `caps 15 <benchmark-executable> candidate`。每个场景预热 20 ms、采样 300 ms；准备工作和精确输出验证不计入测量区间。应在相同工具链下交替执行 A/B 与 B/A；短时运行的耗时不作为 CI 回归阈值。

仅支持替换已有值，不会自动创建容器，也不支持删除或重命名。若值未改变，则返回原始文本，包括原有的数字写法。新值以 `Value` 传入；看起来像数字的 `Value::String` 仍是字符串，必要时使用 raw 标记。

`EditError::InvalidPath` 报告文本路径错误及字节偏移；`NotFound` 表示键不存在、数组索引无效或越界，或试图穿过标量继续访问。`Unsupported` 拒绝无效源文档、无法表示的新值，或当前实现无法在保留周围文本的前提下完成的替换。例如，由分散的点分键或重新打开的定义构成的整个对象没有单一替换范围；替换容器也不能删除其内部注释，但其中的叶子值仍可编辑。行内值使用转义表示 CR 和 LF。在行内复合值之外，含 CR 的字符串及部分依赖上下文的多行/容器替换会被拒绝，而不会被静默改写。所有成功返回的文本都经过解析，并与预期值树比较；错误不会返回部分编辑的文档。

