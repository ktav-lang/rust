>>>>> lang=en
  This is a fix for a span value, not an API change — `ErrorKind`
  shape is unchanged.

### Changed (breaking — parser strictness)

- `key: (value)` and `key: ((value))` now error with
  `ErrorKind::InlineNonEmptyCompound { body: "paren-string" }`.
  These shapes used to be accepted as plain string scalars `(value)`,
  but they are visually indistinguishable from multi-line openers and
  would confuse readers. The raw-marker form `key:: (value)` remains
  valid and is the canonical way to encode such literals. The
  ktav-lsp formatter auto-rewrites the legacy form on save.

### Optimised (no API change)

>>>>> lang=ru
  Это исправление значения span, а не изменение API — форма
  `ErrorKind` не менялась.

### Изменено (breaking — строгость парсера)

- `key: (value)` и `key: ((value))` теперь дают ошибку
  `ErrorKind::InlineNonEmptyCompound { body: "paren-string" }`.
  Раньше эти формы принимались как обычные строковые скаляры
  `(value)`, но визуально они неотличимы от многострочных опенеров и
  сбивали бы читателя с толку. Форма с raw-маркером `key:: (value)`
  остаётся валидной и является каноническим способом записать такой
  литерал. Форматтер ktav-lsp автоматически переписывает legacy-форму
  при сохранении.

### Оптимизировано (без изменения API)

>>>>> lang=zh
  这是对 span 取值的修复，不是 API 变更 —— `ErrorKind` 的形态未变。

### 变更（breaking —— 解析器严格化）

- `key: (value)` 与 `key: ((value))` 现在以
  `ErrorKind::InlineNonEmptyCompound { body: "paren-string" }` 报错。
  这些形态过去被当作普通字符串标量 `(value)` 接受，但它们与多行开启符
  在视觉上无法区分，会让读者困惑。带 raw 标记的形式 `key:: (value)`
  仍然有效，并且是编码此类字面量的规范写法。ktav-lsp 的格式化器会在
  保存时自动改写旧形式。

### 优化（无 API 变更）

