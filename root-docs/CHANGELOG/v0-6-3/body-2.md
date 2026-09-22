>>>>> lang=en
Non-breaking: every input that parsed before still parses to the same
`Value`, and the writer output only changes for keys whose previous
output did not parse at all.

### Changed

- The pinned `ktav-lang/spec` submodule now carries conformance
  fixtures for the seven previously-untested escapes, four negative
  fixtures pinning the boundary (raw `{` / `}` still `InvalidKey`;
  `\(` / `\)` still `BadEscapeSequence`), and the § 5.9.3 wording fix
  for the empty-compound root wrap
  ([spec#6](https://github.com/ktav-lang/spec/issues/6),
  [spec#4](https://github.com/ktav-lang/spec/issues/4)). The suite goes
  from 102 valid / 31 invalid to 110 valid / 35 invalid.

>>>>> lang=ru
Изменение не ломающее: всё, что разбиралось раньше, разбирается в тот
же `Value`, а вывод writer'а меняется только для тех ключей, чей
прежний вывод вообще не разбирался.

### Изменено

- Закреплённый submodule `ktav-lang/spec` теперь содержит conformance-
  фикстуры для семи ранее не покрытых escape, четыре негативные
  фикстуры, закрепляющие границу (сырые `{` / `}` по-прежнему
  `InvalidKey`; `\(` / `\)` по-прежнему `BadEscapeSequence`), и правку
  формулировки § 5.9.3 про обёртку корня для пустых компаундов
  ([spec#6](https://github.com/ktav-lang/spec/issues/6),
  [spec#4](https://github.com/ktav-lang/spec/issues/4)). Набор вырос со
  102 valid / 31 invalid до 110 valid / 35 invalid.

>>>>> lang=zh
本次变更不具破坏性：此前能解析的输入仍解析为相同的 `Value`；写入端的
输出仅在那些此前根本无法解析回来的键上发生改变。

### 变更

- 固定的 `ktav-lang/spec` 子模块现已包含：此前未覆盖的七种转义的
  一致性测试夹具、四个用于钉住边界的负向夹具（原样的 `{` / `}` 仍为
  `InvalidKey`；`\(` / `\)` 仍为 `BadEscapeSequence`），以及关于空复合
  根包裹的 § 5.9.3 措辞修正
  （[spec#6](https://github.com/ktav-lang/spec/issues/6)、
  [spec#4](https://github.com/ktav-lang/spec/issues/4)）。测试集从
  102 valid / 31 invalid 增至 110 valid / 35 invalid。

