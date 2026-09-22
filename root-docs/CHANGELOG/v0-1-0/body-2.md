>>>>> lang=en
- **Parser** — turns Ktav text into a `Value` (owned) or a `ThinValue`
  (zero-copy view over the input buffer). Line-based state machine
  with dotted-key expansion, multi-line strings (stripped and
  verbatim), JSON-style keywords `null` / `true` / `false`, and
  typed-scalar markers `:i` (Integer) and `:f` (Float).
- **Serializer** — two paths:
  - `ktav::to_string` (direct text emission, primary path).
  - `ktav::ser::to_value` / `ktav::render` (two-step for users who
    want to inspect a `Value` between stages).
  Both emit `::` automatically for strings that would otherwise be
  mis-read by the parser, and emit `:i` / `:f` for Rust numeric
  types.
- **Deserializer** — zero-copy path via `ThinValue<'a>` and
  `ThinDeserializer`. Object keys and single-line scalar values are
  borrowed directly from the input; only multi-line strings allocate.
  Accepts both typed-marker and plain-string forms of numbers, so
  documents written without markers deserialize transparently via
  `FromStr`.
- **Serde integration** — `from_str`, `from_file`, `to_string`,
  `to_file` accept any `T: Serialize` / `DeserializeOwned`, including
  `#[derive]`-generated types, nested structs, `Vec`, `Option`,
  `HashMap`, and the usual externally-tagged enum forms. Rust integer
  types (`u8`..`u128`, `i8`..`i128`, `usize`, `isize`) serialize with
  `:i`; floats (`f32`, `f64`) with `:f`; `NaN` and `±Infinity` are
  rejected by the serializer (not representable in Ktav 0.1.0).
- **Raw marker `::`** — forces a value to be a literal String, both
  in pair position (`key:: value`) and as an array-item prefix
  (`:: value`).
- **Typed markers `:i` and `:f`** — explicit Integer / Float in pair
  position (`port:i 8080`, `ratio:f 0.5`) and as array-item prefixes
  (`:i 42`, `:f 3.14`). Values stored as strings at the `Value` layer
  to preserve arbitrary precision.
- **Multi-line strings** — `( ... )` (common-indent stripped) and
  `(( ... ))` (verbatim). Round-trips byte-for-byte via the verbatim
  form.
- **Public `Value` enum** — `Null`, `Bool`, `Integer`, `Float`,
  `String`, `Array`, `Object` (backed by `IndexMap` with
  `rustc_hash::FxBuildHasher`). `Value::as_integer` / `as_float`
  accessors; analogous on `ThinValue`.
- **Error reporting** — every syntax error carries a line number;
  deserialization errors carry a dotted path (`upstreams.[0].port`).
  Typed-scalar violations surface as `InvalidTypedScalar` in the
  message prefix.
- **Spec conformance tests** — `tests/spec_conformance.rs` runs the
  language-agnostic suite from the `ktav-lang/spec` repository
  (resolved via `KTAV_SPEC_DIR` env or `../spec` fallback). Three
  checks: Value-equals-JSON-oracle, invalid-fixtures-rejected, and
  lossless Value-level round-trip through the renderer.

>>>>> lang=ru
- **Parser** — превращает текст Ktav в `Value` (владеющий) или
  `ThinValue` (zero-copy view поверх входного буфера). Построчная
  state machine с разворачиванием точечных ключей, многострочными
  строками (со снятием отступа и побайтовыми), JSON-подобными
  ключевыми словами `null` / `true` / `false` и типизированными
  скалярными маркерами `:i` (Integer) и `:f` (Float).
- **Serializer** — два пути:
  - `ktav::to_string` (прямая эмиссия текста, основной путь).
  - `ktav::ser::to_value` / `ktav::render` (двухшаговый вариант для
    тех, кто хочет осмотреть `Value` между стадиями).
  Оба автоматически эмитят `::` для строк, которые иначе были бы
  неверно прочитаны парсером, и эмитят `:i` / `:f` для числовых
  Rust-типов.
- **Deserializer** — zero-copy путь через `ThinValue<'a>` и
  `ThinDeserializer`. Ключи объектов и однострочные скалярные
  значения заимствуются напрямую из входа; выделение памяти
  происходит только для многострочных строк. Принимает обе формы
  чисел — с маркером и без: документы, написанные без маркеров,
  десериализуются прозрачно через `FromStr`.
- **Serde integration** — `from_str`, `from_file`, `to_string`,
  `to_file` принимают любой `T: Serialize` / `DeserializeOwned`,
  включая типы, сгенерированные `#[derive]`, вложенные struct-ы,
  `Vec`, `Option`, `HashMap` и стандартные externally-tagged формы
  enum-ов. Целочисленные Rust-типы (`u8`..`u128`, `i8`..`i128`,
  `usize`, `isize`) сериализуются с `:i`; плавающие (`f32`, `f64`) —
  с `:f`; `NaN` и `±Infinity` отвергаются сериализатором (Ktav 0.1.0
  их не представляет).
- **Raw-маркер `::`** — заставляет значение быть литеральной String,
  как в позиции пары (`key:: value`), так и как префикс элемента
  массива (`:: value`).
- **Типизированные маркеры `:i` и `:f`** — явные Integer / Float в
  позиции пары (`port:i 8080`, `ratio:f 0.5`) и как префиксы
  элементов массива (`:i 42`, `:f 3.14`). На уровне `Value` хранятся
  как строки — для сохранения произвольной точности.
- **Многострочные строки** — `( ... )` (со снятием общего отступа) и
  `(( ... ))` (побайтово). Round-trip байт-в-байт через побайтовую
  форму.
- **Публичный enum `Value`** — `Null`, `Bool`, `Integer`, `Float`,
  `String`, `Array`, `Object` (на основе `IndexMap` с
  `rustc_hash::FxBuildHasher`). Аксессоры `Value::as_integer` /
  `as_float`; аналогичные на `ThinValue`.
- **Сообщения об ошибках** — каждая синтаксическая ошибка несёт номер
  строки; ошибки десериализации несут точечный путь
  (`upstreams.[0].port`). Нарушения типизированных скаляров
  отмечаются префиксом `InvalidTypedScalar` в сообщении.
- **Spec conformance тесты** — `tests/spec_conformance.rs` прогоняет
  language-agnostic набор из репозитория `ktav-lang/spec`
  (находится через env-переменную `KTAV_SPEC_DIR` или fallback
  `../spec`). Три проверки: соответствие Value JSON-оракулу,
  отвержение invalid-fixture-ов и lossless round-trip Value-уровня
  через рендерер.

>>>>> lang=zh
- **Parser** —— 将 Ktav 文本转换为 `Value`(拥有所有权)或
  `ThinValue`(在输入缓冲区上的零拷贝视图)。基于行的状态机,
  支持点分键展开、多行字符串(剥除缩进与逐字节两种)、
  JSON 风格关键字 `null` / `true` / `false`,以及类型化标量标记
  `:i`(Integer)与 `:f`(Float)。
- **Serializer** —— 两条路径:
  - `ktav::to_string`(直接文本输出,主路径)。
  - `ktav::ser::to_value` / `ktav::render`(两步路径,便于在中间
    检视 `Value`)。
  两者都会在字符串可能被解析器误读时自动输出 `::`,并为 Rust
  数值类型发出 `:i` / `:f`。
- **Deserializer** —— 通过 `ThinValue<'a>` 与 `ThinDeserializer`
  走零拷贝路径。对象键与单行标量值直接从输入借用;只有多行
  字符串会发生分配。接受带标记与不带标记两种数字形式 —— 不含
  标记的旧文档仍能通过 `FromStr` 透明反序列化。
- **Serde integration** —— `from_str`、`from_file`、`to_string`、
  `to_file` 接受任何 `T: Serialize` / `DeserializeOwned`,包括
  `#[derive]` 生成的类型、嵌套结构体、`Vec`、`Option`、`HashMap`
  以及常见的 externally-tagged 枚举形式。Rust 整数类型
  (`u8`..`u128`、`i8`..`i128`、`usize`、`isize`)以 `:i` 序列化;
  浮点(`f32`、`f64`)以 `:f`;`NaN` 与 `±Infinity` 被序列化器
  拒绝(Ktav 0.1.0 不表示)。
- **Raw 标记 `::`** —— 强制将值视为字面量 String,既可用于键值对
  位置(`key:: value`),也可作为数组元素的前缀(`:: value`)。
- **类型化标记 `:i` 与 `:f`** —— 在键值对位置显式声明 Integer /
  Float(`port:i 8080`、`ratio:f 0.5`),也可作为数组元素前缀
  (`:i 42`、`:f 3.14`)。在 `Value` 层以字符串存储,以保留任意
  精度。
- **多行字符串** —— `( ... )`(剥除公共缩进)与 `(( ... ))`
  (逐字节保留)。通过逐字节形式实现字节级 round-trip。
- **公共 `Value` 枚举** —— `Null`、`Bool`、`Integer`、`Float`、
  `String`、`Array`、`Object`(底层为 `IndexMap`,使用
  `rustc_hash::FxBuildHasher`)。访问器 `Value::as_integer` /
  `as_float`;`ThinValue` 上有对应方法。
- **错误报告** —— 每个语法错误都携带行号;反序列化错误携带
  点分路径(`upstreams.[0].port`)。类型化标量违规在消息前缀
  中以 `InvalidTypedScalar` 标示。
- **Spec conformance 测试** —— `tests/spec_conformance.rs` 从
  `ktav-lang/spec` 仓库读取语言无关测试套件(通过 env
  `KTAV_SPEC_DIR` 或回退 `../spec` 解析路径)。三项检查:
  Value 匹配 JSON oracle、invalid fixture 被拒绝、通过渲染器的
  Value 级 round-trip 无损。

