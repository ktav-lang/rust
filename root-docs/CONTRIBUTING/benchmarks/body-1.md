>>>>> lang=en
## Benchmarks

Source: `benches/parse.rs` (criterion). Scenarios cover:

- `parse_to_value` — raw parse into `Value` (the owned, public tree).
- `parse_to_struct` — parse via the thin path into a typed struct.
- `render` — serialize a typed struct to text.
- `roundtrip` — parse + render.
- `multiline_dedent` — multi-line string parsing under varying line
  counts.

>>>>> lang=ru
## Benchmarks

Источник: `benches/parse.rs` (criterion). Сценарии охватывают:

- `parse_to_value` — сырой parse в `Value` (владеющее, публичное
  дерево).
- `parse_to_struct` — parse по thin-пути в типизированную структуру.
- `render` — сериализация типизированной структуры в текст.
- `roundtrip` — parse + render.
- `multiline_dedent` — разбор многострочных строк при разном
  количестве строк.

>>>>> lang=zh
## 基准测试

源文件:`benches/parse.rs`(criterion)。涵盖的场景:

- `parse_to_value` —— 原始 parse 到 `Value`(拥有所有权、公开的
  树形结构)。
- `parse_to_struct` —— 经由 thin 路径 parse 到 typed 结构体。
- `render` —— 把 typed 结构体序列化为文本。
- `roundtrip` —— parse + render。
- `multiline_dedent` —— 不同行数下的多行字符串解析。

