>>>>> lang=en
## [0.6.4] — 2026-08-23

### Fixed

- **Strict parsing now accepts the canonical notation emitted by the
  writer for Float values.** Scientific forms such as `1e-3`, `1.5e-3`,
  `-1e-3`, and `1e7` are compared against the § 5.9.8 writer policy,
  while the stored value remains the same Ryu form used by ordinary
  `parse()`.

### Changed

- The crate is released as `0.6.4` and pins the conformance specification
  to `0.6.4`.
- The `ktav-lang/spec` submodule is pinned to the matching 0.6.4
  specification commit, including the float notation-boundary fixture.

>>>>> lang=ru
## [0.6.4] — 2026-08-23

### Исправлено

- **Строгий разбор теперь принимает каноническую запись Float,
  которую выдаёт writer.** Формы `1e-3`, `1.5e-3`, `-1e-3` и `1e7`
  сравниваются с policy из § 5.9.8, а хранимое значение остаётся той
  же Ryu-формой, что и при обычном `parse()`.

### Изменено

- Crate выпускается как `0.6.4`, а conformance-спецификация закреплена
  на `0.6.4`.
- Submodule `ktav-lang/spec` закреплён на соответствующем коммите
  спецификации 0.6.4, включая fixture границ float-записи.

>>>>> lang=zh
## [0.6.4] —— 2026-08-23

### 修复

- **严格解析现在接受 writer 为 Float 输出的规范形式。** `1e-3`、
  `1.5e-3`、`-1e-3` 与 `1e7` 等科学形式按 § 5.9.8 policy 比较,
  但存储的值仍保持与普通 `parse()` 相同的 Ryu 形式。

### 变更

- crate 版本为 `0.6.4`,conformance 规范固定为 `0.6.4`。
- `ktav-lang/spec` submodule 固定到对应的 0.6.4 规范 commit,
  包含 float 表示边界 fixture。

