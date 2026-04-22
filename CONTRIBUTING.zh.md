# 为 Ktav 做贡献

**Languages:** [English](CONTRIBUTING.md) · [Русский](CONTRIBUTING.ru.md) · **简体中文**

## 核心规则

### 1. 每个 bug fix 都要附带一个回归测试

发现 bug 后,**在动手修复之前**,先写一条能复现它的测试——该测试
**在 `main` 上必须失败**,修复之后通过。两者合并进同一个 PR。

测试应放在相关测试旁边(例如 `tests/edge_cases/<topic>.rs` 或
`tests/ser/<topic>.rs`)。在测试上写一条简短注释,说明故障方式,
好让十个月之后的读者理解这个用例*为什么*重要。

原因:对于一个分发给众多用户的库来说,沉默的回归是最致命的事情。
一条有文档的绊线,长远来看成本为零。

### 2. 涉及性能的改动要给出 before/after 数字

如果 PR 触碰了以下任意位置:

- `src/parser/` / `src/thin/parser.rs` —— 解析热路径
- `src/ser/text_serializer.rs` / `src/render/` —— 序列化热路径
- `src/thin/deserializer.rs` / `src/de/` —— 反序列化热路径
- `src/value/` —— 动态 Value 类型

……请在 PR 描述里给出受影响基准的 **before/after** criterion 数字:

```
parse_to_struct/100_upstreams_typed
  before: 275 µs
  after:  198 µs
  change: -28%
```

目标不是不惜一切追求最快,而是让改动*可追责*。只要可见且有理由,
5% 的回退是可以接受的——前提是换来了清晰度或正确性。

请用便利脚本:

```
./bench.sh                # quick run (warmup 1s, measurement 2s, 20 samples)
./bench.sh full           # criterion defaults — longer, more accurate
./bench.sh parse          # filter: only parse benches
./bench.sh render         # filter: only render benches
./bench.sh "parse|render" # any criterion regex
```

Criterion 会把上一次运行的数字保存在 `target/criterion/`,并在下一次
运行时自动对比,因此输出中会看到 `change: +/- X %`。

### 3. 公共 API 变更要标注兼容性

若你改动了 `lib.rs` 中任何 `pub` 下的项,请在 PR 描述中声明它属于:

- **semver 兼容**(新增、放宽 bound、文档改动);或
- **破坏 semver**(重命名 / 删除项、改签名、收紧 bound)——在
  pre-1.0 阶段,版本号递进进入下一个 `MINOR`。

同一个 PR 里同步更新 `CHANGELOG.md`,置于 `## [Unreleased]` 之下。

### 4. 一个概念,一个 commit

Commit 应保持原子:bug fix 与其测试一起提交,新特性与其测试一起
提交。重命名单独一个 commit。一个顺带修 bug 的重构,通常应该拆成
两个 commit。

`git log --oneline` 应读起来像一份 changelog。按这个方式写。

## 获取代码

规范一致性测试套件位于 git 子模块 `spec/`
([`ktav-lang/spec`](https://github.com/ktav-lang/spec))。克隆时带上
子模块，以确保 `cargo test` 能正常运行：

```
git clone --recurse-submodules https://github.com/ktav-lang/rust
```

若已在不带 `--recurse-submodules` 的情况下克隆：

```
git submodule update --init
```

## 运行测试

```
cargo test                         # 全部测试（含规范一致性）
cargo test --test spec_conformance # 仅语言无关的一致性套件
cargo test --test edge_cases       # 单个分类
cargo test multiline               # 按名称过滤
cargo test --doc                   # 文档测试
```

测试分类:

- `src/**/tests.rs` —— 模块内部的私有单元测试。
- `tests/de/*` —— 按特性组织的反序列化测试。
- `tests/ser/*` —— 按特性组织的序列化测试。
- `tests/roundtrip/*` —— round-trip(`T → text → T`)。
- `tests/edge_cases/*` —— 组合型边界用例(括号字面量、map 中的
  关键字、深度嵌套、特殊字符串……)。
- `tests/fixtures.rs` —— 针对真实 `.conf` 文件的端到端测试。
- `tests/spec_conformance.rs` —— 来自 `ktav-lang/spec` 的语言无关
  套件（valid 固件匹配 JSON 预期值；invalid 固件被拒绝；valid 固件
  经历无损 round-trip）。

## 基准测试

源文件:`benches/parse.rs`(criterion)。涵盖的场景:

- `parse_to_value` —— 原始 parse 到 `Value`(拥有所有权、公开的
  树形结构)。
- `parse_to_struct` —— 经由 thin 路径 parse 到 typed 结构体。
- `render` —— 把 typed 结构体序列化为文本。
- `roundtrip` —— parse + render。
- `multiline_dedent` —— 不同行数下的多行字符串解析。

## 代码布局指南

分解规则:**每个文件一个导出项**。私有辅助函数与使用它的类型放
一起。目录把紧密相关的项归拢(`src/thin/` 全部属于零拷贝反序列化
路径)。

```
src/
├── lib.rs                         public entry points
├── value/                         owned Value enum (public)
├── parser/                        Value-building parser
├── render/                        Value → text
├── thin/                          zero-copy de path (ThinValue → T)
├── ser/                           T → Value (public) + T → text (direct)
├── de/                            Value → T (via ValueDeserializer)
└── error/                         Error + serde impls
```

## 哲学(什么不要做)

Ktav 的格言是「做配置的朋友,别做它的考官」。在提议新特性前,请先问:

- 它是否多加了一条读者必须记在脑子里的规则?
- 一行是否仍能脱离上下文被理解?

新规则总是昂贵的。任何未通过这两道检查的提议,一律否决。
