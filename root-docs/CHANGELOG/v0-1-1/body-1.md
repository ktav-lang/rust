>>>>> lang=en
## [0.1.1] — 2026-04-26

### Changed

- **Typed-deserialization fast path** — `from_str` and `from_file` no
  longer build a `ThinValue` tree as an intermediate. The parser now
  emits a flat `Vec<Event>` directly into a bump arena, and the serde
  deserializer walks it linearly with a single cursor — one allocation
  per document instead of one per compound, and no per-node enum-
  discriminant load behind a `Box`-style indirection. Net impact on a
  275 KB config: **−18.7%** on `parse → struct` (3.60 ms → 2.93 ms).
- **`fast_num` byte-loop atoi** — the `i8`..`i64` / `u8`..`u64` paths
  in the typed deserializer skip the generic `<T as FromStr>` route
  and call hand-rolled `parse_i64` / `parse_u64` with a width check.
  Floats stay on `f64::from_str`.

### Added

- `Event` token enum and `EventCursor` walker (`thin/event*.rs`),
  internal — not exposed in the public surface.

### Removed

- `ThinValue` enum and its `ThinDeserializer` (replaced by the event
  stream — both were `pub(crate)`, so no breakage at the public API).

### Behavior change

- **Interleaved dotted-key prefixes are now rejected as a conflict**.
  A document like `a.x: 1\nb.y: 2\na.z: 3` (synthetic `a` opened, then
  closed by `b.`, then re-opened by `a.z`) used to silently merge into
  one `a` object via the tree-builder. The event-stream tokenizer
  cannot do that without buffering the whole document, so it now
  surfaces a clear conflict error suggesting the user group lines with
  the same prefix together. Documents with grouped dotted keys (the
  canonical pattern) are unaffected — every spec-conformance fixture
  still passes.

>>>>> lang=ru
## [0.1.1] — 2026-04-26

### Изменено

- **Быстрый путь типизированной десериализации** — `from_str` и
  `from_file` больше не строят промежуточное дерево `ThinValue`.
  Парсер сразу эмитит плоский `Vec<Event>` в bump-арену, а serde-
  десериализатор линейно идёт по нему одним курсором — одна
  аллокация на документ вместо одной на компаунд, без подгрузки
  enum-discriminant'а через косвенность. На 275 KB конфиге:
  **−18.7%** на `parse → struct` (3.60 ms → 2.93 ms).
- **`fast_num` byte-loop atoi** — пути `i8`..`i64` / `u8`..`u64` в
  типизированном десериализаторе обходят generic-маршрут через
  `<T as FromStr>` и используют ручные `parse_i64` / `parse_u64`
  с проверкой ширины. Float-пути остаются на `f64::from_str`.

### Добавлено

- Внутренние `Event` enum и `EventCursor` walker (`thin/event*.rs`).

### Удалено

- Enum `ThinValue` и его `ThinDeserializer` (заменены event-stream'ом
  — оба были `pub(crate)`, публичный API не сломан).

### Изменение поведения

- **Чередование dotted-key префиксов теперь отклоняется как
  conflict.** Документ вида `a.x: 1\nb.y: 2\na.z: 3` (synthetic `a`
  открыт, закрыт через `b.`, попытка переоткрыть через `a.z`)
  раньше тихо сливался в один объект `a` через tree-builder.
  Event-stream-токенизатор не может это сделать без буферизации
  всего документа, поэтому теперь возвращает понятный conflict-
  ошибку с предложением сгруппировать строки с одним префиксом
  вместе. Документы со сгруппированными dotted-ключами (канонический
  паттерн) не затронуты — все spec-conformance фикстуры зелёные.

>>>>> lang=zh
## [0.1.1] —— 2026-04-26

### 变更

- **类型化反序列化快路径** —— `from_str` 与 `from_file` 不再构建中间
  `ThinValue` 树。解析器直接将事件序列(`Vec<Event>`)发射到 bump
  arena,serde 反序列化器以单一游标线性遍历它 —— 每个文档一次分配
  而非每个复合节点一次,且无需通过 `Box` 间接加载枚举判别式。
  在 275 KB 配置上的实测:`parse → struct` **−18.7%**(3.60 ms →
  2.93 ms)。
- **`fast_num` 字节循环 atoi** —— 类型化反序列化器中的 `i8`..`i64`
  / `u8`..`u64` 路径绕过通用的 `<T as FromStr>` 路线,改为调用手写
  的 `parse_i64` / `parse_u64` 并附带宽度检查。浮点路径仍走
  `f64::from_str`。

### 新增

- 内部 `Event` 枚举与 `EventCursor` 遍历器(`thin/event*.rs`)。

### 移除

- `ThinValue` 枚举及其 `ThinDeserializer`(已被事件流取代;两者均为
  `pub(crate)`,公开 API 不受影响)。

### 行为变化

- **dotted-key 前缀的交错使用现在会被拒绝为 conflict。** 形如
  `a.x: 1\nb.y: 2\na.z: 3` 的文档(合成对象 `a` 被打开,被 `b.`
  关闭,然后被 `a.z` 尝试重新打开)以前会通过 tree-builder 静默
  合并为单一 `a` 对象。事件流标记器在不缓冲整个文档的情况下无法
  做到这一点,因此现在会返回清晰的 conflict 错误,提示用户将相同
  前缀的行分组在一起。使用分组 dotted-key(规范模式)的文档不受
  影响 —— 所有 spec-conformance 用例仍然通过。

