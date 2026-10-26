>>>>> lang=en
Dotted keys stay dotted, inline objects/arrays stay inline, and block objects/arrays stay blocks, including when emptied. This applies to deep edits inside arrays too. Replacing a scalar with a new container uses a block outside an inline parent; replacing a whole inline container preserves its inline form when readable. A type change does not by itself reformat ancestors. No-op edits bypass all readability rules.

An edited inline line is expanded only when the replacement contains a multiline string, the line grows beyond `max(100, original line length)`, or a replacement string needs at least four inline escapes with at least 25% extra encoded characters. Length means Unicode scalar values, not UTF-8 bytes or terminal columns; the escape overhead is compared with the decoded string length. Existing long lines are not expanded unless they grow.

Expansion opens the containing inline wrapper and only the affected nested wrappers still needing expansion; untouched nested branches may stay inline. Key spelling and order are preserved. Moved scalar spellings and raw markers change only when standalone syntax needs a different encoding. New block levels use four spaces and the source EOL style; no final newline is added. If a lossless standalone encoding is unavailable, for example because a string contains CR or a line equal to `))`, the valid inline representation is retained instead. Readability never authorizes comment loss or semantic changes. These fixed rules add no configuration mode.

A short paired benchmark lives in `benches/source_edit.rs`. Build it separately with `cargo bench --bench source_edit --no-run --message-format=json`, then run the reported executable with `baseline` or `candidate` as its argument. Apply an external 15-second process timeout (for example, `caps 15 <benchmark-executable> candidate` on Windows). Each case uses a 20 ms warmup and a 300 ms sample; setup and exact-output validation are outside the measured interval. Compare alternating A/B and B/A runs on the same toolchain; short-run wall-clock results are not a CI regression gate.

Replacement is joined by `insert_value`, `remove_value` and `rename_key` (see the section below): a new final Object key, deletion of a leaf/subtree/Array element, and an in-place member rename. An unchanged value returns the original text, including its original numeric spelling. New values are supplied as `Value`; a numeric-looking `Value::String` stays a string, using a raw marker where necessary.

`EditError::InvalidPath` reports a malformed textual path and byte offset; `NotFound` covers missing keys, invalid/out-of-range array indexes and descent through a scalar. `Unsupported` rejects an invalid source, an unrepresentable replacement or a replacement for which this implementation cannot preserve the surrounding text. In particular, a whole object assembled from separate dotted/reopened definitions has no single replacement range, and replacing a container must not delete its internal comments. Its individual leaves remain editable. Inline values use escapes for CR and LF. Outside inline compounds, CR-containing strings and some context-dependent multiline/container replacements are rejected rather than silently rewritten. Every returned patch is parsed and checked against the intended value tree; errors never return a partially edited document.

### Insertion, removal and renaming (unreleased)

`insert_value(source, path, value)` adds a new final Object key under an existing Object parent. The pair is appended as a whole line: at the end of the parent for block and implicit-root containers, as an inline segment before the closer for inline containers; the source EOL and the presence or absence of the final newline are preserved. An existing key is `EditError::AlreadyExists` — insertion never overwrites; insertion into an Array is an explicit `Unsupported`; missing intermediates and descent through a scalar are `NotFound`. Inline strings use the inline escaper (commas, brackets, backslash, line breaks, numeric-looking bodies); when the readability rules require it, the inline parent is expanded to block form; when a lossless block form is unavailable (a CR, a `))` line), the verified inline form is kept.

`remove_value(source, path)` deletes an existing leaf, a whole subtree, or one Array element (the tail shifts). Empty ancestors are not pruned: removing the only dotted leaf materializes the emptied dotted chain as a `k: {}` line at the same position, while a real block keeps its braces and an inline container keeps its inline form. Comment lines inside a removed container are refused with `Unsupported`; comments adjacent to the target are preserved. The BOM is kept as source metadata and never enters an edit range.

`rename_key(source, path, new_key)` renames a member in place, replacing only the last addressed key segment: dotted prefixes, quotes and escapes of the source key are kept byte-for-byte, and the new key is spelled with the standard bare/quoted encoding. An identical key is a byte-identical no-op; an existing sibling target is `AlreadyExists`; an Array index or an unrepresentable key is `Unsupported`. Order, value, separator bytes and comments are untouched.

All three operations have `*_segments` variants taking decoded literal segments, share the textual path grammar of `edit_value`, and run the same parse–splice–reparse–verify pipeline; an invalid source fails closed with `Unsupported`.

>>>>> lang=ru
Dotted-ключи остаются dotted, однострочные объекты/массивы — однострочными, блочные — блочными, в том числе при опустошении. Это относится и к глубоким правкам внутри массивов. При замене скаляра новым контейнером вне inline-родителя используется блок; при замене целого inline-контейнера сохраняется его однострочная форма, если она читаема. Смена типа сама по себе не форматирует предков. No-op обходит все критерии читаемости.

Изменённая inline-строка раскрывается только при многострочном тексте в замене, росте длины сверх `max(100, исходная длина строки)` либо если строке в замене требуется не менее четырёх inline-экранирований с увеличением числа символов хотя бы на 25%. Длина считается в Unicode scalar values, не в байтах UTF-8 и не в колонках терминала; накладные расходы экранирования сравниваются с длиной декодированной строки. Уже длинная строка без роста не раскрывается.

Раскрывается содержащая inline-обёртка и только затронутые вложенные обёртки, которым всё ещё требуется раскрытие; нетронутые вложенные ветви могут остаться однострочными. Написание и порядок ключей сохраняются. Написание переносимых скаляров и raw-маркеры меняются только при необходимости другого кодирования вне inline. Новые уровни блока используют четыре пробела и EOL исходника; финальный перевод строки не добавляется. Если lossless-кодирование вне inline невозможно, например из-за CR или строки `))` в данных, сохраняется корректное inline-представление. Читаемость не разрешает потерю комментариев или изменение семантики. Эти фиксированные правила не добавляют режимов настройки.

Короткий парный benchmark находится в `benches/source_edit.rs`. Сначала отдельно соберите его командой `cargo bench --bench source_edit --no-run --message-format=json`, затем запустите указанный в выводе исполняемый файл с аргументом `baseline` или `candidate`. Ограничьте процесс внешним timeout 15 секунд, например `caps 15 <benchmark-executable> candidate` в Windows. Каждый сценарий использует прогрев 20 мс и замер 300 мс; подготовка и проверка точного результата не входят в измеряемый интервал. Чередуйте A/B и B/A на одном toolchain; время коротких прогонов не служит порогом регрессии в CI.

Замена дополнена `insert_value`, `remove_value` и `rename_key` (см. раздел ниже): новый конечный ключ Object, удаление листа/поддерева/элемента массива и переименование члена на месте. Совпадающее значение возвращает исходный текст, включая первоначальное написание чисел. Новые значения передаются как `Value`; похожий на число `Value::String` остаётся строкой, при необходимости с raw-маркером.

`EditError::InvalidPath` сообщает об ошибочном текстовом пути и смещении в байтах; `NotFound` — об отсутствующем ключе, неверном/выходящем за границы индексе или спуске через скаляр. `Unsupported` отвергает невалидный исходник, непредставимую замену либо замену, для которой реализация не может сохранить окружающий текст. В частности, у целого объекта из отдельных dotted/reopened-определений нет единого диапазона замены, а замена контейнера не должна удалять внутренние комментарии. Отдельные листья такого объекта остаются редактируемыми. В inline-значениях CR и LF передаются escape-последовательностями. Вне inline-контейнеров строки с CR и некоторые контекстно-зависимые многострочные/контейнерные замены отвергаются, а не переписываются молча. Каждый возвращаемый результат разбирается и сверяется с ожидаемым деревом значений; ошибка не возвращает частично изменённый документ.

### Вставка, удаление и переименование (ещё не выпущено)

`insert_value(source, path, value)` добавляет новый конечный ключ Object под существующим Object-родителем. Пара добавляется целой строкой: в конец родителя для блочных и неявно-корневых контейнеров, inline-сегментом перед closer для inline; EOL исходника и наличие/отсутствие финального перевода строки сохраняются. Существующий ключ — `EditError::AlreadyExists`: вставка никогда не перезаписывает; вставка в Array — явный `Unsupported`; отсутствующие промежуточные уровни и спуск через скаляр — `NotFound`. Inline-строки кодируются inline-экранированием (запятые, скобки, backslash, переводы строк, числоподобные тела); когда требуют правила читаемости, inline-родитель раскрывается в блок; если lossless-блочная форма невыразима (CR, строка `))`), остаётся проверенная inline-форма.

`remove_value(source, path)` удаляет существующий лист, целое поддерево или один элемент массива (хвост сдвигается). Пустые предки не удаляются: после удаления единственного dotted-листа опустевшая dotted-цепочка материализуется строкой `k: {}` на том же месте, реальный блок сохраняет фигурные скобки, inline-контейнер — inline-форму. Комментарии внутри удаляемого контейнера — `Unsupported`; соседние комментарии сохраняются. BOM хранится как метаданные исходника и никогда не попадает в диапазон правки.

`rename_key(source, path, new_key)` переименовывает член на месте, заменяя только последний адресуемый сегмент ключа: dotted-префикс, кавычки и escapes исходного ключа сохраняются байт-в-байт, новый ключ записывается стандартным bare/quoted-кодированием. Идентичный ключ — byte-identical no-op; занятый сосед — `AlreadyExists`; индекс массива или невоспроизводимый ключ — `Unsupported`. Порядок, значение, байты разделителя и комментарии не меняются.

Все три операции имеют `*_segments`-варианты с декодированными буквальными сегментами, используют грамматику текстового пути `edit_value` и проходят тот же конвейер parse–splice–reparse–verify; невалидный исходник — fail closed `Unsupported`.

>>>>> lang=zh
点分键仍为点分键，行内对象/数组仍为行内形式，块对象/数组仍为块形式，包括清空容器时。数组深层编辑也遵守此规则。在行内父容器之外用新容器替换标量时使用块；替换整个行内容器时，只要可读就保留行内形式。类型变化本身不触发祖先格式化。值不变时跳过所有可读性规则。

仅在替换值包含多行字符串、修改后行长增长并超过 `max(100, 原始行长)`，或替换字符串需要至少四个行内转义且编码字符数增加至少 25% 时展开。长度按 Unicode 标量值计算，而非 UTF-8 字节或终端列；转义开销相对于解码后字符串长度计算。原本已很长但没有增长的行不会展开。

展开所在的行内外层容器，并仅继续展开仍需展开的已修改嵌套容器；未修改的嵌套分支可以保留行内形式。保留键的写法和顺序。仅当独立行语法需要不同编码时，才改变移位标量的写法和 raw 标记。新块层级使用四个空格及原文的 EOL 风格，不添加末尾换行。若无法无损转换为独立行，例如字符串包含 CR 或一整行 `))`，则保留有效的行内表示。可读性不能成为删除注释或改变语义的理由。这些固定规则不增加配置模式。

短时配对基准位于 `benches/source_edit.rs`。先使用 `cargo bench --bench source_edit --no-run --message-format=json` 单独编译，再以 `baseline` 或 `candidate` 参数运行输出中列出的可执行文件。用外部工具将每个进程限制为 15 秒，例如 Windows 下的 `caps 15 <benchmark-executable> candidate`。每个场景预热 20 ms、采样 300 ms；准备工作和精确输出验证不计入测量区间。应在相同工具链下交替执行 A/B 与 B/A；短时运行的耗时不作为 CI 回归阈值。

替换之外还提供 `insert_value`、`remove_value` 和 `rename_key`（见下文）：新增末级 Object 键、删除叶子/子树/数组元素，以及原位重命名成员。若值未改变，则返回原始文本，包括原有的数字写法。新值以 `Value` 传入；看起来像数字的 `Value::String` 仍是字符串，必要时使用 raw 标记。

`EditError::InvalidPath` 报告文本路径错误及字节偏移；`NotFound` 表示键不存在、数组索引无效或越界，或试图穿过标量继续访问。`Unsupported` 拒绝无效源文档、无法表示的新值，或当前实现无法在保留周围文本的前提下完成的替换。例如，由分散的点分键或重新打开的定义构成的整个对象没有单一替换范围；替换容器也不能删除其内部注释，但其中的叶子值仍可编辑。行内值使用转义表示 CR 和 LF。在行内复合值之外，含 CR 的字符串及部分依赖上下文的多行/容器替换会被拒绝，而不会被静默改写。所有成功返回的文本都经过解析，并与预期值树比较；错误不会返回部分编辑的文档。

### 插入、删除与重命名（尚未发布）

`insert_value(source, path, value)` 在已存在的 Object 父级下新增一个末级键。新键值对作为完整一行追加：块级和隐式根容器追加到父级末尾，行内容器作为 closer 前的行内段插入；保留源文档的 EOL 及末尾换行的有无。已存在的键报 `EditError::AlreadyExists`——插入绝不覆盖；对 Array 的插入是显式 `Unsupported`；缺失中间层级和穿过标量为 `NotFound`。行内字符串使用行内转义（逗号、括号、反斜杠、换行、数字样内容）；当可读性规则要求时，行内父级展开为块；若无法无损表示块形式（CR、`))` 行），则保留已验证的行内形式。

`remove_value(source, path)` 删除已存在的叶子、整个子树或一个数组元素（尾部前移）。空祖先不会被清理：删除唯一的点分叶子后，被清空的点分链会在原位置物化为 `k: {}` 行；真实块保留花括号，行内容器保留行内形式。被删容器内部的注释以 `Unsupported` 拒绝；相邻注释保留。BOM 作为源元数据保留，绝不进入编辑范围。

`rename_key(source, path, new_key)` 在原位重命名成员，仅替换最后寻址的键段：源键的点分前缀、引号和转义逐字节保留，新键采用标准 bare/quoted 编码。相同键为逐字节 no-op；同级已存在目标为 `AlreadyExists`；数组索引或不可表示的键为 `Unsupported`。顺序、值、分隔符字节与注释不变。

三个操作均提供接受字面已解码段的 `*_segments` 变体，使用与 `edit_value` 相同的文本路径语法，经过同样的 parse–splice–reparse–verify 流程；无效源以 `Unsupported` 失败闭合。

