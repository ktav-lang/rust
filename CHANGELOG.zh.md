# 变更日志 —— `ktav` crate

**Languages:** [English](CHANGELOG.md) · [Русский](CHANGELOG.ru.md) · **简体中文**

本文件记录 `ktav` crate 的全部重要变更。格式参照
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/);crate
遵循 [Semantic Versioning](https://semver.org/),并采用 Cargo 惯例:
在 1.0 之前,MINOR 递进视为破坏性变更。

格式规范自身的历史,请见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) 仓库。

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
