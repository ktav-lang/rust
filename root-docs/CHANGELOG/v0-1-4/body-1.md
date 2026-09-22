>>>>> lang=en
## [0.1.4] — 2026-04-26

### Changed

- **`Frame::Object` initial capacity 4 → 8** (`src/parser/frame.rs`).
  The parser's per-compound `IndexMap` now pre-sizes for 8 entries
  instead of 4, which eliminates the first growth/rehash for the
  typical 5–8-field config row. This is the **untyped** parse path
  (`ktav::parse → Value`) — the same path every C-ABI binding
  (PHP/JS/Python/Go/Java/C#) walks through `cabi`, so they all see
  the speedup once they pick up 0.1.4.
- Net impact on the `parse_to_value` bench (3-run median): small
  **−30%** (18.9 µs → 13.3 µs), large **−13%** (5.04 ms → 4.4 ms),
  medium in the noise (~−3%).

One-line change; full test suite (334 cases incl. spec conformance)
unaffected.

>>>>> lang=ru
## [0.1.4] — 2026-04-26

### Изменено

- **`Frame::Object` initial capacity 4 → 8** (`src/parser/frame.rs`).
  Per-compound `IndexMap` парсера теперь pre-sizes под 8 элементов
  вместо 4 — устраняется первый growth/rehash для типичной строки
  конфига (5–8 полей). Это **untyped** парсинг-путь
  (`ktav::parse → Value`) — тот самый путь, через который идут все
  C-ABI биндинги (PHP/JS/Python/Go/Java/C#) через `cabi`, поэтому
  они **получат** ускорение, как только подхватят 0.1.4.
- Эффект на бенче `parse_to_value` (медиана 3 прогонов): small
  **−30%** (18.9 µs → 13.3 µs), large **−13%** (5.04 ms → 4.4 ms),
  medium в шуме (~−3%).

Изменение в одну строку; полный набор тестов (334 кейса вкл.
spec conformance) не затронут.

>>>>> lang=zh
## [0.1.4] —— 2026-04-26

### 变更

- **`Frame::Object` 初始容量 4 → 8**(`src/parser/frame.rs`)。
  解析器的 per-compound `IndexMap` 现在预分配 8 个槽位而非 4 个,
  消除了典型配置行(5–8 字段)的首次扩容/rehash。这是 **untyped**
  解析路径(`ktav::parse → Value`)—— 也是所有 C-ABI 绑定
  (PHP/JS/Python/Go/Java/C#)通过 `cabi` 走的路径,因此一旦它们
  升级到 0.1.4 就会获得相同的加速。
- 在 `parse_to_value` 基准上的影响(3 次运行中位数):small
  **−30%**(18.9 µs → 13.3 µs)、large **−13%**(5.04 ms → 4.4 ms)、
  medium 在噪声范围内(~−3%)。

单行改动;完整测试套件(334 个用例,含 spec conformance)不受影响。

