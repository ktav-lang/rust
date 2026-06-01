# 变更日志 —— `ktav` crate

**Languages:** [English](CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

本文件记录 `ktav` crate 的全部重要变更。格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);crate
遵循 [Semantic Versioning](https://semver.org/),并采用 Cargo 惯例:
在 1.0 之前,MINOR 递进视为破坏性变更。

格式规范自身的历史,请见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) 仓库。


## [0.6.0] —— 2026-06-01

实现 Ktav 规范 0.6.0。新增 **键转义**:键现在处理 §3.7 转义集,并
新增两条转义 —— `\.` 与 `\:` —— 允许在键段内出现字面意义的点 / 冒号。

### 破坏性变更

- 键中的字面 `\` 现在需要写作 `\\`。此前解析器把键中的 `\` 当作
  无转义的内容字节。源文件中含单个 `\` 的键需要改为双反斜杠以保
  持相同的键字节。值的处理不变。

### 新增

- 转义表由 8 条扩展到 10 条:原有 8 条(`\\`、`\,`、`\}`、`\]`、
  `\{`、`\[`、`\n`、`\r`)之上新增 `\.`(键段中的字面点 —— 不分割
  路径)与 `\:`(字面冒号 —— 不作为键/值分隔符)。两条新转义在值
  上下文中亦合法(冗余)。
- 键扫描器支持转义:首个 **未转义** 的 `:`(或 `::`)为对分隔符;
  点分路径仅在 **未转义** 的 `.` 处分割。内联 compound
  (`{a\.b: 1}`)采用同样规则。
- 渲染器在输出每个键段时回写转义 `\`、`.`、`:`,保证含字面点/冒
  号的键的 parse → render → parse 同一性。
- 零拷贝事件路径下,不含 `\` 的键段继续从源缓冲区借用;含转义的
  键段解码到 bump-arena —— `&'a str` 生命周期保持不变。

### 错误

- 键中 `\X`(`X` 不在十条转义之列)现在抛出 `BadEscapeSequence`。
  `\.` 与 `\:` 在任何上下文均不再是 `BadEscapeSequence`。


## [0.2.0] —— 2026-05-07

次要发布,带两项 breaking 输出 / 校验改动:

### 变更(breaking)

- **多行字符串默认输出为缩进 stripped 形式 `( ... )`**,而非 verbatim
  `(( ... ))`。当内容自带前导空白(会被解析侧的 dedent 吞掉)或包含
  仅为 `)` 的行(会提前关闭 stripped)时,fallback 到 verbatim。逐字节
  比较 `to_string` / `render` 输出与硬编码 `((...))` 的代码需要更新。
  Round-trip(`parse(to_string(v)) == v`)未受影响。

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


## [0.1.5] —— 2026-05-01

主要发布:带字节偏移 span 的结构化错误、公开的事件式解析器 API、
对错误枚举追溯应用 `#[non_exhaustive]` 以保证前向兼容。

### 新增

- `ErrorKind` 枚举(10 个规范定义变体 + `Other`),每个变体携带
  字节偏移 `span: Span`,直接向下游消费者暴露 `(line, column, kind)`,
  无需通过正则解析格式化消息。完整变体清单见英文版。
- 现有 `Error` 枚举上的 `Error::Structured(ErrorKind)` 变体。
- `pub struct Span { start: u32, end: u32 }`,提供 `Span::new`、
  `Span::EMPTY`、`slice(input)` 与 `line_col(input)`(line 从 1 起,
  column 从 0 起按字节计;多字节 UTF-8 已通过西里尔与 🦀 测试固定)。
- `Error::line() -> Option<u32>` 和 `Error::span() -> Option<Span>` —
  覆盖每个变体的便捷访问器。
- `pub mod thin` —— 公开的事件式解析器 API:
  `ktav::parse_events(input, callback)` 对从 input 借用的每个事件
  调用所提供的 `FnMut(ParseEvent<'_>)`。`ParseEvent` 是
  `#[non_exhaustive]` 枚举,有 10 个变体。内部 bumpalo arena
  保持私有 —— 公共 API 不泄露 arena 类型。
- `src/lib.rs` 的 crate 级可运行 doctest,演示 `Error::Structured`
  匹配配合 `Span::slice` 以及 `parse_events` 回调形态。
- 六个新顶层测试文件:`tests/error_format.rs`、
  `tests/structured_errors.rs`、`tests/error_spans.rs`、
  `tests/error_accessors.rs`、`tests/non_exhaustive.rs`、
  `tests/thin_public.rs` —— 详见英文版。
- `benches/` 下的合成 Criterion 基准,覆盖 small_1k / medium_50k /
  large_500k 负载在成功路径和错误路径上的解析性能。baseline 数字
  位于 `bench-baseline.md`。

### 变更

- 对 `Error`、`ErrorKind`、`ConflictKind`、`CompoundKind` 追溯应用
  `#[non_exhaustive]`。未来添加变体对下游的 `match` 不再是破坏性
  变更 —— 调用方现在必须包含 `_ =>` 分支。
- 解析器在任何内部调用点都不再构造 `Error::Syntax(format!(...))`
  (约 37 个站点重构为 `Error::Structured`)。回归保护测试
  (`parser_no_longer_emits_legacy_syntax_variant`)运行 12 个非法
  输入,若有人在 `src/` 内重新引入旧变体,CI 会大声失败。
- `parser/parse_str.rs` 用维护累积 `line_start` 计数的手工字节遍历
  循环替代 `str::lines()`,以便在每个错误站点计算字节偏移 span 而
  无需重新扫描。`thin/event_parser.rs` 在零拷贝路径上镜像同一
  plumbing。
- `Display for ErrorKind` 与解析器先前格式化进 `Error::Syntax(...)`
  的字符串完全字节相同(覆盖七个先前已固定类别)—— 这是让现有
  基于字符串的调用方在
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md) 描述的生态系统
  级迁移期间保持不变工作的契约。
- 三个先前归入 `Other` 的形态升级为命名 `ErrorKind` 变体:
  `UnbalancedBracket`(孤立闭符 / 形状不匹配)、
  `InlineNonEmptyCompound`(`x: {foo}` —— 规范 § 6.7)、
  `MissingSeparator`(无 `:` 的行)。升级后,`Other` 仅保留没有任何
  规范非法夹具能触发的解析器内部不变量。

### 性能

`cargo bench --bench parse -- --quick` 与 0.1.4 baseline 对比:成功
路径零回归,错误路径略快(惰性 Display 取代了即时 `format!`)。
完整表格见英文版。

### 备注

- 为了向后兼容,`Error::Syntax(String)` 保留 —— 公共 API 仍不拒绝
  老调用方。移除推迟到 ktav 1.0。
- 测试数:332 (0.1.4) → 391 (+59),外加 1 个新 doctest。
- cabi/绑定迁移以通过 FFI 边界消费 `ErrorKind` 的工作单独记录在
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md),并作为协调发布
  的生态系统 0.2.0 一并交付。

### SemVer 说明

按
[Cargo SemVer 参考](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
为先前未标注的枚举(`Error`、`ConflictKind`、`CompoundKind`)添加
`#[non_exhaustive]` 是破坏性变更,通常需要主版本号 bump(0.2.0)。
本次发布有意作为 **0.1.5** 推出:

1. Pre-1.0 的 Cargo 惯例允许在任何 bump(包括 patch)上做破坏性
   变更。
2. `ktav::Error` 所有已知的下游消费者(`ktav-lang/` 下的六个语言
   绑定)均仅调用 `Err(e) => e.to_string()`。生态系统中不存在会被
   该变更悄悄破坏的穷尽 `match err { Error::Io(_) => …,
   Error::Syntax(_) => …, Error::Message(_) => … }` 模式。
3. 七个标准类别的 Display 字符串与 0.1.4 字节相同,因此任何在
   tree 之外做字符串匹配的假想消费者也无需修改即可继续工作。

如果你的代码确实保留了对 `ktav::Error` 的穷尽 `match` 而本次发布
将其破坏 —— 请添加 `_ => …` 分支。该分支从此永久必须存在,且不会
随未来变体的添加而再次需要变更。

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

## [0.1.3] —— 2026-04-26

与已 yank 的 0.1.2 内容完全相同 —— 通过新的自动化 `Release` 工作流
(CI verify → `cargo publish`)重新发布,从而后续发布不再依赖维护
者本地机器上的手动 `cargo publish`。0.1.2 被 yank 仅用于在一个全新
版本上端到端验证流水线(crates.io 不可变,无法重新发布 0.1.2 本身)。

## [0.1.2] —— 2026-04-26

0.1.1 内容的重新发布,源码经过 `cargo fmt` 处理。0.1.1 被 yank,因为
新增文件(`benches/vs_json.rs`, `src/thin/event*.rs`,
`src/thin/fast_num.rs`)在发布前未经 rustfmt 处理,导致 CI lint 在
tag push 时失败。**功能与 0.1.1 完全一致** —— 仅空白字符不同。

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

## [0.1.0] —— 2026-04-22

首次发布。实现 [Ktav spec 0.1.0](https://github.com/ktav-lang/spec/blob/main/versions/0.1/spec.md)。

### Added

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

### Performance(criterion,22 KB 的 typed 配置,Windows release)

- `parse → struct`: **275 µs**(~80 MB/s)
- `render struct → text`: **46 µs**(~475 MB/s)
- `round-trip`: **377 µs**

### Dependencies

- `serde`(含 `derive`)
- `indexmap`(启用 `serde` 特性)
- `rustc-hash`(FxHash —— 快且确定性;不抗碰撞,而配置解析器
  并不需要抗碰撞)

### MSRV

`rustc 1.70` 或更新版本。
