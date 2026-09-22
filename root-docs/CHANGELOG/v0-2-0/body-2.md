>>>>> lang=en
  Both `Value` → `render::render(&value)` and `T: Serialize` →
  `ser::to_string(&t)` paths are updated consistently.

- **Typed-float marker `:f` now accepts integer literals.** The mantissa's
  decimal point is **optional**: `:f 42` is valid (parsed as `42.0`),
  matching the JSON / TOML / YAML convention that integer literals
  coerce to floats. `:f 1.` (no fractional digits) and `:f .5` (no
  integer part) remain invalid. Code that depends on `:f 42` raising
  `InvalidTypedScalar` needs to be updated.

### Spec

- `spec/versions/0.1/tests` fixture `typed_float_without_decimal` moved
  from `invalid/` to `valid/typed_float_integer_body` to reflect the new
  semantics. Spec submodule synced.

>>>>> lang=ru
  Оба пути сериализации — `Value` → `render::render(&value)` и
  `T: Serialize` → `ser::to_string(&t)` — обновлены консистентно.

- **Типизированный маркер `:f` принимает integer-литералы.** Десятичная
  точка в мантиссе теперь **опциональна**: `:f 42` валидно (парсится
  как `42.0`) — конвенция JSON / TOML / YAML, где integer литералы
  приводятся к float. `:f 1.` (без дробной части) и `:f .5` (без
  целой части) по-прежнему невалидны. Код, ожидающий
  `InvalidTypedScalar` для `:f 42`, нужно обновить.

### Spec

- В `spec/versions/0.1/tests` фикстура `typed_float_without_decimal`
  перенесена из `invalid/` в `valid/typed_float_integer_body` под
  новую семантику. Submodule spec синхронизирован.

>>>>> lang=zh
  序列化双路径(`Value` → `render::render(&value)` 与
  `T: Serialize` → `ser::to_string(&t)`)同步更新,行为一致。

- **类型标记 `:f` 接受整数字面量。** 尾数中的小数点现在是**可选**:
  `:f 42` 合法(解析为 `42.0`),沿用 JSON / TOML / YAML 的惯例
  (整数字面量隐式提升为 float)。`:f 1.`(无小数部分)与 `:f .5`
  (无整数部分)仍然非法。依赖 `:f 42` 报 `InvalidTypedScalar` 的
  代码需要更新。

### Spec

- `spec/versions/0.1/tests` 中的 fixture `typed_float_without_decimal`
  从 `invalid/` 移到 `valid/typed_float_integer_body`,以反映新语义。
  spec submodule 已同步。

