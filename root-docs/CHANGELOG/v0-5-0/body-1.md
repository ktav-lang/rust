>>>>> lang=en
## [0.5.0] — 2026-05-28

Implements Ktav specification 0.5.0. This is a breaking release:
the parser, serializer, and Value model are rewritten for the new
language semantics.

### Breaking

- Typed markers `:i` and `:f` removed. Numbers, booleans, and `null`
  are inferred from the lexical form (spec §§ 3.6, 5.2).
- Comments use `##` (line-start only). A single `#` byte is content.
- Bare `port: 8080` is now `Integer(8080)`, not `String("8080")`.
  Write `port:: 8080` to keep a String.
- Lone `{` / `[` on the first content line opens a multi-line root
  Object / Array (spec § 5.0.1 rules 4–5). The 0.1.1 JSONL-style
  semantic is removed.
- `Float` Values no longer carry the textual form; canonicalised via
  `ryu`.
- Key segments are trimmed of leading/trailing whitespace (spec § 4).
- Line terminators are `LF`, `CR`, or `CR LF`; `CR` is never a
  content byte.
- `ErrorKind::InlineNonEmptyCompound` and `InvalidTypedScalar` are
  deprecated (`#[doc(hidden)]`); the parser no longer emits them.

### Added

>>>>> lang=ru
## [0.5.0] — 2026-05-28

Реализует спецификацию Ktav 0.5.0. Это ломающий релиз: парсер,
сериализатор и модель Value переписаны под новую семантику языка.

### Breaking

- Типизированные маркеры `:i` и `:f` удалены. Числа, булевы значения
  и `null` выводятся из лексической формы (спек §§ 3.6, 5.2).
- Комментарии пишутся через `##` (только с начала строки). Одиночный
  байт `#` — это содержимое.
- Голое `port: 8080` теперь `Integer(8080)`, а не `String("8080")`.
  Чтобы сохранить String, пишите `port:: 8080`.
- Одиночная `{` / `[` на первой содержательной строке открывает
  многострочный корневой Object / Array (спек § 5.0.1, правила 4–5).
  Семантика 0.1.1 в стиле JSONL удалена.
- Значения `Float` больше не несут текстовую форму; канонизируются
  через `ryu`.
- Сегменты ключа обрезаются от ведущих и хвостовых пробелов (спек § 4).
- Терминаторы строк — `LF`, `CR` или `CR LF`; `CR` никогда не является
  байтом содержимого.
- `ErrorKind::InlineNonEmptyCompound` и `InvalidTypedScalar` объявлены
  устаревшими (`#[doc(hidden)]`); парсер их больше не выдаёт.

### Добавлено

>>>>> lang=zh
## [0.5.0] —— 2026-05-28

实现 Ktav 规范 0.5.0。这是一次破坏性发布：解析器、序列化器与 Value
模型均按新的语言语义重写。

### 破坏性变更

- 移除类型标记 `:i` 与 `:f`。数字、布尔与 `null` 由词法形态推断
  （规范 §§ 3.6、5.2）。
- 注释使用 `##`（仅限行首）。单个 `#` 字节属于内容。
- 裸写的 `port: 8080` 现在是 `Integer(8080)`，不再是 `String("8080")`。
  若要保留 String，请写 `port:: 8080`。
- 首个内容行上单独的 `{` / `[` 开启多行的根 Object / Array
  （规范 § 5.0.1 规则 4–5）。0.1.1 的 JSONL 式语义已移除。
- `Float` 值不再携带文本形态；改由 `ryu` 规范化。
- 键段的首尾空白会被裁剪（规范 § 4）。
- 行终止符为 `LF`、`CR` 或 `CR LF`；`CR` 永远不作为内容字节。
- `ErrorKind::InlineNonEmptyCompound` 与 `InvalidTypedScalar` 标记为
  废弃（`#[doc(hidden)]`）；解析器不再产生它们。

### 新增

