>>>>> lang=en
## [0.3.0] — 2026-05-08

Minor release with one breaking parser strictness change, a
diagnostic-range fix, and hot-path micro-optimisations on the
typed-deserialize path.

### Fixed

- `ErrorKind::DuplicateKey` and `ErrorKind::KeyPathConflict` now carry
  the span of the **offending key**, not the closing `}` / `]` of the
  compound that would have been assigned to it. Previously, when the
  conflict was detected on `attach_child_value` (e.g. `value: { ... }`
  duplicating an earlier `value: ...`), the saved span pointed at the
  closing brace because that's the position the parser had at hand at
  the moment of detection.

  The parser now stores the key's own span (`pending_key_span`) on the
  parent frame when a compound is opened, and reuses it when the
  compound closes and the value is attached. Editors / IDEs that draw
  diagnostic underlines from `Span` now point at the key.

>>>>> lang=ru
## [0.3.0] — 2026-05-08

Минорный релиз с одним ломающим ужесточением парсера, исправлением
диапазона диагностики и микрооптимизациями горячего пути
типизированной десериализации.

### Исправлено

- `ErrorKind::DuplicateKey` и `ErrorKind::KeyPathConflict` теперь несут
  span **виновного ключа**, а не закрывающей `}` / `]` того
  составного, которое ему присваивалось. Раньше, когда конфликт
  обнаруживался на `attach_child_value` (например, `value: { ... }`,
  дублирующее более раннее `value: ...`), сохранённый span указывал на
  закрывающую скобку — это была позиция, которая оказывалась у парсера
  под рукой в момент обнаружения.

  Теперь парсер сохраняет собственный span ключа (`pending_key_span`)
  в родительском фрейме при открытии составного и переиспользует его,
  когда составное закрывается и значение присваивается. Редакторы и
  IDE, рисующие диагностические подчёркивания по `Span`, теперь
  указывают на ключ.

>>>>> lang=zh
## [0.3.0] —— 2026-05-08

次要发布，含一项破坏性的解析器严格化、一处诊断范围修复，以及类型化
反序列化热路径上的微优化。

### 修复

- `ErrorKind::DuplicateKey` 与 `ErrorKind::KeyPathConflict` 现在携带
  **出错键**自身的 span，而不是本应赋给它的那个复合结构的收尾
  `}` / `]`。此前，当冲突在 `attach_child_value` 处被发现时
  （例如 `value: { ... }` 与更早的 `value: ...` 重复），保存的 span
  指向收尾括号——那是解析器在发现冲突的那一刻手头持有的位置。

  现在解析器在打开复合结构时把键自身的 span（`pending_key_span`）
  存入父帧，并在复合结构收尾、值被挂接时复用它。依据 `Span` 绘制
  诊断下划线的编辑器 / IDE 现在会指向键。

