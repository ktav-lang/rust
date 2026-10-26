# Ktav (כְּתָב)

[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

> 一种朴素的配置格式。形态上接近 JSON5,但不带引号、不用逗号,
> 以点分键表达嵌套。原生 `serde` 集成。

**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**

**演练场：** 在浏览器中互转 JSON / YAML / TOML / INI ⇄ Ktav — **[ktav-lang.github.io](https://ktav-lang.github.io/)**。

**0.9.0 正在准备中，尚未发布。** 本分支同时包含按路径无损编辑源文本
及新的源错误坐标模型。`Cargo.toml` 指向规范草案语料 `versions/0.9/tests`；
已发布的 0.8.1 保持不变。规范见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec)，迁移说明见
[`CHANGELOG.zh.md`](CHANGELOG.zh.md)。

---

## 名称

*Ktav*(希伯来语:**כְּתָב**)意为「书写、被写下来的东西」——一种
以足够稳固的形态被记录下来的东西,其意义不依赖于传递者。名字用得
很字面:一份配置文件*就是*磁盘上的 ktav,而本库把它读进来,原样
交出一个活生生的结构,不会在途中擅自脑补。

## 格言

> **做配置的朋友,别做它的考官。配置并不完美——但已是最好的那一份。**

每条规则都是局部的。每一行要么独立成立,要么只依赖于可见的括号。
没有缩进陷阱,没有忘记的引号,没有尾随逗号的算术。

## 规则

Ktav 文档是一个隐式的顶层对象。任何对象里是键值对,任何数组里是
元素。

```text
## comment              — any line starting with '##'
key: value             — scalar pair; key may be a dotted path (a.b.c)
key:: value            — scalar pair; value is ALWAYS a literal string
key: { ... }           — multi-line object; `}` closes on its own line
key: [ ... ]           — multi-line array; `]` closes on its own line
key: {}   /   key: []  — empty compound, inline
key: ( ... )           — multi-line string; common indent stripped
key: (( ... ))         — multi-line string; verbatim (no stripping)
:: value               — inside an array: literal-string item
```

整个语言就这些。没有逗号、没有引号、没有值内部的转义——唯一的
「转义」是 `::` 标记,它出现在分隔符里(用于键值对)或行首前缀里
(用于数组元素)。

## 值与特殊记号

### 字符串

任何标量的默认类型。在内部以 `Value::String` 存放。值就是 `:` 之后
经过空白修剪的内容。

```text
name: Russia
path: /etc/hosts
greeting: hello world
## `::` 强制将值解释为字面量字符串
pattern:: [a-z]+
```

### 数字

数字不加引号,并根据字面形式分类:裸整数 body 解析为
`Value::Integer`,裸小数解析为 `Value::Float`。两者存放的都是
*规范化*后的 payload,而非原始写法:`Integer` 保存规范十进制形式
(无下划线、无 `+`),`Float` 保存能还原精确 `f64` 位模式的最短
十进制形式——`+1_000` 变为 `Integer("1000")`,`1.0e+2` 变为
`Float("100.0")`。数字以冗余的 `0` 开头的十进制根本不是数字:
`zip: 01234` 是 `String("01234")`(§ 5.2),因此补零的标识符会被
逐字保留。`Value` 层的 `Integer` 覆盖 i64 范围;比 i64 更宽的
native Rust 整数类型(`u64`、`i128`、`u128`)若超出该范围,经过
`ser::to_value` 时会存为 `Value::String`——效果与直接解析同一段十进制
文本完全一致。serde 在反序列化时直接将数字解析为目标 Rust 类型
(`u16`、`i64`、`i128`、`f64`……),并在序列化时用相同的规范化格式化。

```text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
```

像 `port: abc` 这样的值在 *Ktav 层*可以正常解析(即字符串 `"abc"`),
但 `serde::deserialize` 到 `u16` 时会返回一个清晰的 `ParseError`。

### 布尔:`true` / `false`

严格小写。其它写法都是字符串。

```text
## Value::Bool(true)
on: true
## Value::Bool(false)
off: false
## Value::String("True")
capitalized: True
## Value::String("FALSE")
yelling:    FALSE
## Value::String("true")
literal:: true
```

### Null:`null`

严格小写。对应 Rust 侧的 `Option::None`,以及 unit 类型 `()`。

```text
## Value::Null
label: null
## Value::String("Null")
capitalized: Null
## Value::String("null")
literal:: null
```

序列化时,`Option::None` 会输出为 `null`。若你希望该字段干脆缺省,
可以加上 `#[serde(skip_serializing_if = "Option::is_none")]`。

### 空对象 / 空数组

**唯一**允许的内联复合值——没什么要分隔,不需要逗号。

```text
## 空对象
meta: {}
## 空数组
tags: []
```

### 与关键字同形的字符串需要 `::`

若字符串内容恰好等于关键字(`true`、`false`、`null`),或以 `{`、
`[` 开头,**序列化器会自动输出 `::`**,以保证 round-trip 无损。
写入端请用同样的方式:

```text
## 字符串 "true",而非 Bool
flag:: true
## 字符串 "null",而非 Null
noun:: null
regex:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl
```

## 复合值是多行的

非空的 `{ ... }` / `[ ... ]` **必须**跨越多行,闭合括号独占一行。
`x: { a: 1 }` 与 `x: [1, 2, 3]` 会被以清晰的错误拒绝——Ktav 没有
逗号分隔的规则,也没有针对它们的转义机制。

```text
## rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]

## accepted — multi-line form
server: {
    host: 127.0.0.1
    port: 8080
}

tags: [
    primary
    eu
    prod
]
```

## 值得知道的边角

以上示例未触及的四条规则。每一条都由规范而非本实现决定,因此任何
符合规范的解析器行为都一致。

### 引号键 —— 当键里有点号或空格时

裸键中的点号表示嵌套:`db.host: primary` 会构建 `db` → `host`。
给键加引号会关闭这一解读,点号因此成为名字的一部分(§ 5.3.3):

```ktav
db.host: primary
"db.host": literal
"a b": spaces are fine too
```

第一行产生嵌套。第二行是一个字面名为 `db.host` 的键。这正是错误
信封把 `path` 表示为段数组而非拼接字符串的原因 —— 拼接字符串无法区分
这两者。

### 转义在行内复合值与引号键中生效

`\uXXXX` 与具名转义(§ 3.7、§ 3.7.1)在分隔符本应具有结构含义的
位置被解读 —— 即 `{ }` 与 `[ ]` 内部,以及引号键内部。裸的块级值没有
需要转义的分隔符,因此其中的反斜杠是普通文本:

```ktav
inline: {greek: \u03b1, csv: a\,b}
"\u00e9": 1
literal: \u0041
```

`greek` 是 `α`,`csv` 是单个字符串 `a,b`(被转义的逗号不是
分隔符),引号键是 `é`。但 `literal` 是原封不动的八个字符
`\u0041`。被识别的转义还会强制归类为字符串:写作 `\u0031` 的值是
字符串 `1`,而不是整数。

### 恰好跳过一个前导 BOM

文档最开头的 U+FEFF 会在检查任何其他字节之前被跳过(§ 3.1)。出现
在其他位置时它是普通内容 —— 包括后续行的开头,在那里它会成为该键名字的
一部分。因此保存时添加 BOM 的编辑器不会破坏文档,而中间位置的多余 BOM
也不会悄然消失。

### 非法 UTF-8 是独立的错误,而非 I/O 失败

[`from_file`](https://docs.rs/ktav) 在解析前校验文件字节是否为
UTF-8,并以首个非法序列的字节偏移报告 [`Error::InvalidUtf8`](§ 6.15)。
文件缺失或权限问题仍然是 [`Error::Io`] —— 二者值得区分,因为一个意味着
「修文件」,另一个意味着「修路径」。字节在解析器看到之前绝不会被修复或
替换。

## 在 Rust 中使用

Ktav 原生支持 serde。任何实现了 `Serialize` / `Deserialize` 的类型
(包括 `#[derive]` 生成的)都可以开箱即用地通过 Ktav 完成
round-trip。

### 解析 —— 直接解码到类型化结构体

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Db { host: String, timeout: u32 }

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port:    u16,
    ratio:   f64,
    tls:     bool,
    tags:    Vec<String>,
    db:      Db,
}

const SRC: &str = "\
service: web
port: 8080
ratio: 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout: 30
";

let cfg: Config = ktav::from_str(SRC)?;
println!("port={} db.host={}", cfg.port, cfg.db.host);
```

### 遍历 —— 在动态 `Value` 枚举上 match

```rust
use ktav::value::Value;

let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("顶层始终是 object") };

for (k, v) in top {
    let kind = match v {
        Value::Null         => "null".into(),
        Value::Bool(b)      => format!("bool={b}"),
        Value::Integer(s)   => format!("int={s}"),
        Value::Float(s)     => format!("float={s}"),
        Value::String(s)    => format!("str={s:?}"),
        Value::Array(a)     => format!("array({})", a.len()),
        Value::Object(o)    => format!("object({})", o.len()),
    };
    println!("{k} -> {kind}");
}
```

### 构建并渲染 —— 用代码搭建文档

```rust
use ktav::value::{ObjectMap, Value};

let mut top = ObjectMap::default();
top.insert("name".into(),  Value::String("frontend".into()));
top.insert("port".into(),  Value::Integer("8443".into()));
top.insert("tls".into(),   Value::Bool(true));
top.insert("ratio".into(), Value::Float("0.95".into()));
top.insert("notes".into(), Value::Null);

let text = ktav::render::render(&Value::Object(top))?;
```

通常请走 serde 路径(`ktav::to_string(&cfg)`);仅在 schema
是动态的时候才需要直接操作 `Value`。

四个公共入口:读取用 [`from_str`](https://docs.rs/ktav) /
[`from_file`](https://docs.rs/ktav),写入用
[`to_string`](https://docs.rs/ktav) / [`to_file`](https://docs.rs/ktav)。
完整可运行示例:[`examples/basic.rs`](examples/basic.rs)。

### 检视错误 —— 给工具链使用的结构化变体

`Error::Structured(ErrorKind)` 为每次解析失败携带类型化的类别加字节
偏移 `Span`,因此编辑器和 linter 可以高亮恰好出错的范围,而不是
整行。

```rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\nport: 443\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("第 {line} 行:重复键 {key:?}");
        println!("出错字节: {:?}", span.slice(src));   // -> Some("port")
        let (l, c) = span.line_col(src);                // 行 1 起,列 0 起按字节
        println!("高亮 {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
```

变体:`MissingSeparatorSpace`、`InvalidTypedScalar`、`DuplicateKey`、
`KeyPathConflict`、`EmptyKey`、`InvalidKey`、`UnclosedCompound`、
`UnbalancedBracket`、`InlineNonEmptyCompound`、`MissingSeparator`、
`LossyScalar`（仅严格模式，见下文）、`Other`。该枚举标注
`#[non_exhaustive]` —— 必须包含 `_ =>` 分支。变体不重要时可使用便捷
访问器 `Error::line()` / `Error::span()`。`Display` 输出的字符串与
遗留 `Error::Syntax(_)` 完全一致,因此原有基于字符串的调用方无需修改
即可继续工作。

完整可运行示例遍历所有变体:
[`examples/errors.rs`](examples/errors.rs) —— `cargo run --example errors`。

### 任何结构化错误共用一个 JSON 信封

上面的访问器只存在于 Rust。`ErrorEnvelope` 是给其余各方的传输契约:
一个 JSON 对象,十个字段,永远是全部十个,顺序固定 —— `error`、
`reason`、`line`、`line_text`、`span`、`path`、`body`、
`canonical`、`spec_section`、`message`。

```rust
use ktav::{parse, ErrorEnvelope};

let src = "a: 1.10\n";
if let Err(e) = ktav::parse_strict(src) {
    println!("{}", ErrorEnvelope::from_error(&e, src).to_json());
}
```

```json
{"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
 "span":{"start":0,"end":7},"path":null,"body":"1.10",
 "canonical":"1.1","spec_section":"§3.6/§5.2",
 "message":"Syntax error: Line 1: LossyScalar: '1.10' would be …"}
```

`message` 是最后一个字段,也是唯一永远不为 `null` 的字段:它逐字
携带该错误的 `Display` 渲染结果。绑定层直接把它展示给用户,而不是自己
从结构化字段拼装文字,因此同一份文档在任何语言下都会给出相同的错误文本。
原有的九个字段保持各自的位置 —— `message` 是追加的,而非插入的。

缺失的信息是显式的 `null`,而不是省略键,因此使用方无需事先协商模式
即可按位置读取每个字段。

`span` 是 `{"start":N,"end":M}`,单位为 **UTF-8 源文本中的字节
偏移**,而不是 UTF-16 code unit —— 与 [`Span`](https://docs.rs/ktav)
本身一致。LSP 使用方要么自行转换,要么协商
`positionEncoding: "utf-8"`。

`path` 是**精确解码后的键段数组,绝不是拼接字符串**。字面名为
`a.b` 的键是一个段,不会与两段路径混淆 —— 传输契约里根本没有可供
产生歧义的分隔符。

写入器的拒绝使用同一个信封:`reason` 携带 § 5.9.0 的原因码
(`NonFiniteFloat`、`EmptyKeyName` 等),两种拒绝分别命名 ——
写入器能指出出错节点时为 `UnrepresentableAt`(此时也会填充
`path`),不能指出时为 `Unrepresentable`。

渲染用 `to_json()`(或 `push_json(&mut String)` 追加进你自己的
缓冲区)。对任何载荷它都是合法 JSON —— 每个字符串都按 RFC 8259
转义 —— 且不涉及 serde 依赖。

### 严格模式 —— 捕获被静默规范化的数字

类型由标量的词法形式推断，且推断出的数字会被规范化：`version: 1.10`
解析为 `Float(1.1)`。带冗余前导零的十进制字面量是唯一的例外：`zip: 01234`
是 String `"01234"`（§ 5.2），因为丢掉那个零会摧毁一个标识符。默认的
`parse()` 会静默完成这一过程，因此把文档写回去就会改写它。

`parse_strict()` 则拒绝这类**有损标量**：

```rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "version: 1.10\n";

assert!(parse(src).is_ok());                 // Float(1.1) —— 尾部零丢失
assert!(parse("zip: 01234\n").is_ok());       // String("01234") —— 前导零被保留

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
    }
    other => panic!("期望 LossyScalar，实际为 {other:?}"),
}
```

修复方式有两种：追加 `::` 让该值保持字符串（`zip:: 01234`），或直接
写成规范数字。凡是 `parse_strict()` 接受的文档，其产生的 `Value` 树
与 `parse()` 完全相同 —— 严格模式是一道校验闸门，而非另一种方言。
serde 路径（`from_str`）目前尚无严格模式变体。

### 流式解析 —— 不构建中间树的事件流

`parse_events` 对每个解析事件调用回调,字符串直接从 input 缓冲区
借用 —— 不在每个事件上分配,也不构建中间 `Value` 树。在不需要完整
文档时很有用:统计键、流式转换到其他格式、构建自定义形状。

```rust
use ktav::{parse_events, ParseEvent};

let src = "port: 8080\nhost: example.com\n";
let mut keys = Vec::new();
parse_events(src, |ev| {
    if let ParseEvent::Key(k) = ev {
        keys.push(k.to_string());
    }
})?;
assert_eq!(keys, ["port", "host"]);
```

根据文档第一条内容行的形状,根节点是 `BeginObject`/`EndObject` 或
`BeginArray`/`EndArray`(此处因为 `port: 8080` 是一个 pair,所以是
Object);嵌套复合值以同样方式括起自身内容。`ParseEvent` 标注
`#[non_exhaustive]`。完整的、带深度跟踪
和漂亮打印的可运行示例:
[`examples/events.rs`](examples/events.rs) —— `cargo run --example events`。

### 数字

Rust 数值类型(`u8`..`u128`、`i8`..`i128`、`usize`、`isize`、`f32`、
`f64`)会以裸数字序列化到 Ktav:`port: 8080`、`ratio: 0.5`。回程时,
裸整数/小数 body 直接反序列化为目标数值类型;以字符串形式到达的值
(例如用 `::` 强制的)仍通过 `FromStr` 接受。`NaN` / `±Infinity` 会被
序列化器拒绝(Ktav 不表示这些值)。

## 错误模型迁移（0.9）

**0.9 迁移（破坏性变更）：** `Error::InvalidUtf8` 现在包含
`{ valid_up_to, line, span }`，`ErrorKind::UnclosedCompound` 包含
`{ kind, line, span }`。下游构造器需要填写新增字段；只关注部分字段时，
模式应使用 `..`。两种错误的 `line()` 都返回源行号。UTF-8 区间覆盖原始
错误字节；EOF 诊断覆盖最内层未闭合起始标记至 EOF，并报告起始标记所在行。
坐标保留 BOM 及 LF/CR/CRLF；`Span::line_col` 和 envelope 的 `line_text`
识别全部三种换行形式。I/O 及 writer 错误仍不携带源位置。下述十字段 JSON
格式不变。新增位置 oracle 位于规范草案的 `versions/0.9/tests`；
当前工作版本尚未发布。

## 示例:Ktav → JSON5

右侧使用 JSON5,因为它读起来像普通 JavaScript,允许注释,并且
能完整展示解析器产出的结果。

### 1. 标量

```text
name: Russia
port: 20082
```
```json5
{
  name: "Russia",
  port: 20082
}
```

标量在 `Value` 层就已根据其字面形式被分类
(`Integer`/`Float`/`Bool`/`Null`/`String`);若数字形状的内容需要保持
字符串,可用 `::` raw 标记强制(例如 `port:: 20082`)。

### 2. 点分键 = 嵌套对象

```text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
```
```json5
{
  server: { host: "127.0.0.1", port: 8080 },
  app: { debug: true }
}
```

深度任意。完整地址写在每一行上。

### 3. 作为值的嵌套对象

```text
server: {
    host: 127.0.0.1
    port: 8080
    endpoints.api: /v1
    endpoints.admin: /admin
}
```
```json5
{
  server: {
    host: "127.0.0.1",
    port: 8080,
    endpoints: { api: "/v1", admin: "/admin" }
  }
}
```

### 4. 标量数组

```text
banned_patterns: [
    .*\.onion:\d+
    .*:25
]
```
```json5
{
  banned_patterns: [".*\\.onion:\\d+", ".*:25"]
}
```

### 5. 对象数组

```text
upstreams: [
    {
        host: a.example
        port: 1080
    }
    {
        host: b.example
        port: 1080
    }
]
```
```json5
{
  upstreams: [
    { host: "a.example", port: 1080 },
    { host: "b.example", port: 1080 }
  ]
}
```

### 6. 任意层嵌套

每个复合值都跨越多行(带内容的单行 `{ ... }` / `[ ... ]` 不被接受
——只有空形式 `{}` / `[]` 允许写作内联)。想嵌多深就嵌多深:

```text
countries: [
    {
        name: Russia
        cities: [
            {
                name: Moscow
                buildings: [
                    {
                        name: Kremlin
                    }
                    {
                        name: Saint Basil's
                    }
                ]
            }
            {
                name: Saint Petersburg
            }
        ]
    }
    {
        name: France
    }
]
```

### 7. 字面量字符串:`::`

某些值如果不加标记,会被当作复合值解析(因为以 `{` 或 `[` 开头):
正则、IPv6 地址、模板占位符。双冒号 `::` 把它们标记为「原样字符串,
不要继续解析」。

```text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl

hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
```
```json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
```

对于键值对,该标记位于键和值之间;对于数组元素,它位于行首。
**当字符串值以 `{` 或 `[` 开头时,序列化会自动输出 `::`**,所以
正则与 IPv6 地址的 round-trip 自然工作。

### 8. 注释

```text
## top-level comment
port: 8080

items: [
    ## this comment does not break the array
    a
    b
]
```

注释为整行,以 `#` 开头。不支持行内注释——太容易与值混淆。

### 9. 多行字符串:`( ... )` 与 `(( ... ))`

跨越多行的值放在圆括号里。开起行与关闭行**不**属于值。

`(` ... `)` —— 剥除公共前导缩进,且从 0.7 起还会剥除每行末尾的空白,
所以你可以按照周围代码的缩进
书写,而不会污染内容:

```text
body: (
    {
      "qwe": 1
    }
)
```
```json5
{ body: "{\n  \"qwe\": 1\n}" }
```

`((` ... `))` —— 逐字节保留:开始与结束标记之间的每一个字符都
进入值,包括前导空白:

```text
sig: ((
  -----BEGIN-----
  QUJDRA==
  -----END-----
))
```
```json5
{ sig: "  -----BEGIN-----\n  QUJDRA==\n  -----END-----" }
```

在块内,`{` / `[` / `#` 只是内容——**不做复合解析,不跳过注释**。
唯一的特殊序列就是独占一行的终止符。

空的内联形式:`key: ()` 或 `key: (())`——都会产生空字符串(等同
于 `key:`)。

序列化:字符串只有在不含 `\n`、没有首尾空白、且不含除 `TAB`
以外的控制字节时才会以单行形式输出。其他情况——哪怕只是普通字符串
边缘多了一个空格——都会采用多行形式:当内容含有会被去缩进破坏的
边缘空白时采用逐字形式 `(( ... ))`,否则采用去缩进形式
`( ... )`;无论 writer 选择哪种形式,
round-trip 都是字节级无损的。

具体采用哪种块形式,取决于空白出现在**哪一侧**。从 0.7 起,去缩进
形式会剥除每行内容末尾的空白(§ 5.6),因此末尾空格必须使用逐字
形式,由它逐字节保留:

```json5
{ password: "hunter2 " }
```
```text
password: ((
hunter2 
))
```

前导空白——从 0.7 起还包括末尾空白——必须使用逐字形式:去缩进会
把前导缩进吃掉:

```json5
{ indent: "  padded" }
```
```text
indent: ((
  padded
))
```

两种情况读回来都能得到原始字节。

限制:若正文中有一行 trim 后恰好是 `))`,则不能使用逐字形式,会
改用去缩进形式——除非正文中还含有恰好是 `)` 的一行、只含空白的
一行、末尾带空白的一行,或所有行都有缩进(没有可用于将去缩进基准定为零的行),此时
没有任何形式能容纳该正文,序列化会返回错误,而不是输出一个无法
还原的文档。

### 10. 空复合值

```text
meta: {}
tags: []
```

允许内联空。带内容的值必须跨越多行,且闭合的 `}` / `]` 必须独占
一行。

### 11. 枚举

Ktav 使用 serde 默认的 *externally tagged* 枚举表示形式。

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode { Fast, Slow }

#[derive(Serialize, Deserialize)]
enum Action {
    Log(String),
    Count(u32),
}
```

```text
## unit variant — just the name
mode: fast

## newtype variant — single-entry object
action: {
    Log: hello
}
```

## Round-trip

```rust
let cfg: MyConfig = ktav::from_str(text)?;
let back = ktav::to_string(&cfg)?;
let again: MyConfig = ktav::from_str(&back)?;
assert_eq!(cfg, again);
```

序列化会保持:
- **字段顺序** —— `Value::Object` 底层是 `IndexMap`,顺序由 serde
  输出决定(对结构体而言:就是声明顺序)。
- **字面量字符串** —— 以 `{` 或 `[` 开头的值会带 `::` 标记输出。
- **`None` 字段** —— 输出时跳过;输入时通过 serde 的 `Option`
  处理重新出现为 `None`。

## 格式化 —— 规范写法,注释保留

`ktav::format_str` 把文档改写为 `emit_canonical` 所产出的结构写法,
但保留规范写入器会丢弃的附属内容。每条注释都逐字保留。

```rust
let tidied = ktav::format_str("## the server\nserver: {host: a, port: 80}\n")?;
assert_eq!(tidied, "## the server\nserver: {\n    host: a\n    port: 80\n}\n");
```

也就是说,行内复合值展开为规范的多行形式,而注释仍停留在原处:

```ktav
## the server
server: {
    host: a
    port: 80
}
```

空行作为分组提示保留,但连续两行及以上会折叠为恰好一行,紧贴括号内侧
的空行填充会被丢弃。正是这一点让该变换成为不动点:对已格式化的输出再
次格式化不会再有任何改变。

键序永不改变。规范形式没有排序规则(§ 5.9),而重排键只会让评审差异
更糟,而非更好 —— 这是写法规范化工具,不是重构工具。

对于既无注释**也无空行**的文档,`format_str` 等于其解析结果的
`emit_canonical`。这个更强的条件是刻意的:空行与注释一样都不属于
`Value` 模型,因此 `emit_canonical` 会丢弃它们,而 `format_str`
不会。

### `ktav-fmt` —— 可选的命令行格式化器

位于 `cli` feature 之后,默认关闭。在已有工具链的项目里,上面的库
调用加一个构建钩子通常更合适,编辑器则经由 `ktav-lsp` 格式化;二进制
是留给两者都不可用的场景。

```text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
```

`--check` 不写入任何内容,只打印每个尚未格式化的文件路径,因此可以
直接放进 CI,紧挨着 `cargo fmt --check`。

## 源文本编辑（尚未发布）

此 API 正在开发中，**不包含在已发布的 0.8.1 crate 中**。
与 `format_str` 不同，`ktav::edit` 不会重新格式化整个文档，而是在原始文本中替换已有值。实际编辑范围之外的字节保持不变，包括注释、空行、键的写法、顺序、缩进、BOM 和原有换行符。通常只修改目标；若可读性要求展开行内祖先，编辑范围也包含这些外层容器。仅在新值需要时调整 raw 标记。输入以借用方式传入；成功时返回新的 `String`，出错时原文不变。

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;

let source = "## top\n\na: 1\n\n## end\n";
let replacement = Value::Integer("2".into());
let expected = "## top\n\na: 2\n\n## end\n";
assert_eq!(edit_value(source, "a", &replacement)?, expected);
assert_eq!(edit_value_segments(source, &["a"], &replacement)?, expected);
let path = vec![String::from("a")];
assert_eq!(edit_value_segments(source, &path, &replacement)?, expected);
```

文本路径 `key1.key2.0.key4` 使用 `.` 分隔各段。键名中的点写作 `\.`，反斜杠写作 `\\`；其他转义、未完成的转义和空文本段均为错误。对于字面键名 `a.b`，Rust 路径参数写作 `r"a\.b"` 或 `"a\\.b"`。切片/向量 API 接受已解码的名称（`&["a.b"]`），无需路径转义。在对象中，`0` 表示键 `"0"`；在数组中，它表示索引零。数组索引只能包含 ASCII 数字，允许前导零，不允许正负号。

### 保留源文本形式

点分键仍为点分键，行内对象/数组仍为行内形式，块对象/数组仍为块形式，包括清空容器时。数组深层编辑也遵守此规则。在行内父容器之外用新容器替换标量时使用块；替换整个行内容器时，只要可读就保留行内形式。类型变化本身不触发祖先格式化。值不变时跳过所有可读性规则。

仅在替换值包含多行字符串、修改后行长增长并超过 `max(100, 原始行长)`，或替换字符串需要至少四个行内转义且编码字符数增加至少 25% 时展开。长度按 Unicode 标量值计算，而非 UTF-8 字节或终端列；转义开销相对于解码后字符串长度计算。原本已很长但没有增长的行不会展开。

展开所在的行内外层容器，并仅继续展开仍需展开的已修改嵌套容器；未修改的嵌套分支可以保留行内形式。保留键的写法和顺序。仅当独立行语法需要不同编码时，才改变移位标量的写法和 raw 标记。新块层级使用四个空格及原文的 EOL 风格，不添加末尾换行。若无法无损转换为独立行，例如字符串包含 CR 或一整行 `))`，则保留有效的行内表示。可读性不能成为删除注释或改变语义的理由。这些固定规则不增加配置模式。

短时配对基准位于 `benches/source_edit.rs`。先使用 `cargo bench --bench source_edit --no-run --message-format=json` 单独编译，再以 `baseline` 或 `candidate` 参数运行输出中列出的可执行文件。用外部工具将每个进程限制为 15 秒，例如 Windows 下的 `caps 15 <benchmark-executable> candidate`。每个场景预热 20 ms、采样 300 ms；准备工作和精确输出验证不计入测量区间。应在相同工具链下交替执行 A/B 与 B/A；短时运行的耗时不作为 CI 回归阈值。

替换之外还提供 `insert_value`、`remove_value` 和 `rename_key`（见下文）：新增末级 Object 键、删除叶子/子树/数组元素，以及原位重命名成员。若值未改变，则返回原始文本，包括原有的数字写法。新值以 `Value` 传入；看起来像数字的 `Value::String` 仍是字符串，必要时使用 raw 标记。

`EditError::InvalidPath` 报告文本路径错误及字节偏移；`NotFound` 表示键不存在、数组索引无效或越界，或试图穿过标量继续访问。`Unsupported` 拒绝无效源文档、无法表示的新值，或当前实现无法在保留周围文本的前提下完成的替换。例如，由分散的点分键或重新打开的定义构成的整个对象没有单一替换范围；替换容器也不能删除其内部注释，但其中的叶子值仍可编辑。行内值使用转义表示 CR 和 LF。在行内复合值之外，含 CR 的字符串及部分依赖上下文的多行/容器替换会被拒绝，而不会被静默改写。所有成功返回的文本都经过解析，并与预期值树比较；错误不会返回部分编辑的文档。

### 插入、删除与重命名（尚未发布）

`insert_value(source, path, value)` 在已存在的 Object 父级下新增一个末级键。新键值对作为完整一行追加：块级和隐式根容器追加到父级末尾，行内容器作为 closer 前的行内段插入；保留源文档的 EOL 及末尾换行的有无。已存在的键报 `EditError::AlreadyExists`——插入绝不覆盖；对 Array 的插入是显式 `Unsupported`；缺失中间层级和穿过标量为 `NotFound`。行内字符串使用行内转义（逗号、括号、反斜杠、换行、数字样内容）；当可读性规则要求时，行内父级展开为块；若无法无损表示块形式（CR、`))` 行），则保留已验证的行内形式。

`remove_value(source, path)` 删除已存在的叶子、整个子树或一个数组元素（尾部前移）。空祖先不会被清理：删除唯一的点分叶子后，被清空的点分链会在原位置物化为 `k: {}` 行；真实块保留花括号，行内容器保留行内形式。被删容器内部的注释以 `Unsupported` 拒绝；相邻注释保留。BOM 作为源元数据保留，绝不进入编辑范围。

`rename_key(source, path, new_key)` 在原位重命名成员，仅替换最后寻址的键段：源键的点分前缀、引号和转义逐字节保留，新键采用标准 bare/quoted 编码。相同键为逐字节 no-op；同级已存在目标为 `AlreadyExists`；数组索引或不可表示的键为 `Unsupported`。顺序、值、分隔符字节与注释不变。

三个操作均提供接受字面已解码段的 `*_segments` 变体，使用与 `edit_value` 相同的文本路径语法，经过同样的 parse–splice–reparse–verify 流程；无效源以 `Unsupported` 失败闭合。

### 源文本编辑示例

以下示例适用于尚未发布的 0.9 API，而非已发布的 0.8.1 包。只能替换已有值：不会重命名键、创建缺失路径或删除条目。各代码片段相互独立；Rust 的 `?` 假定外层函数返回 `Result`。Go 示例仅针对固定且有效的输入使用 panic；应用程序应处理返回的错误。

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;
```

#### 1. 通过点分键修改部署配置

只修改地址；注释、点分形式及相邻的用户字段逐字节保持不变。

```rust
let source = "## deployment\ntargets.storagehost.address: old\ntargets.storagehost.user: mcp-agent\n";
let updated = edit_value(source, "targets.storagehost.address", &Value::String("192.168.0.88:22".into()))?;
```

结果:

```ktav
## deployment
targets.storagehost.address: 192.168.0.88:22
targets.storagehost.user: mcp-agent
```

#### 2. 字面点号：文本路径或路径段

源码中带引号的键名就是 `a.b`。文本路径需要转义点号；路径段 API 直接接收解码后的键名。两个调用返回相同结果。

```rust
let source = "\"a.b\": {port: 80}\n";
let updated = edit_value(source, "a\\.b.port", &Value::Integer("90".into()))?;
let same = edit_value_segments(source, &["a.b", "port"], &Value::Integer("90".into()))?;
```

结果:

```ktav
"a.b": {port: 90}
```

#### 3. 修改混合形式数组中的叶子值

此处 `0` 是数组索引；在对象中，同一个路径段表示名称为 `0` 的键。未修改的容器保持行内形式。

```rust
let source = "servers: [{host: old, keep: yes}, standby]\n";
let updated = edit_value_segments(source, &["servers", "0", "host"], &Value::String("new".into()))?;
```

结果:

```ktav
servers: [{host: new, keep: yes}, standby]
```

#### 4. 看似数字的字符串仍然是字符串

替换值是 String，而不是 Integer。raw 标记保留这种区别；需要数值时请传入类型化 Integer。

```rust
let source = "port: 8080\n";
let updated = edit_value(source, "port", &Value::String("9090".into()))?;
```

结果:

```ktav
port:: 9090
```

#### 5. 多行替换：仅展开必要的容器

修改的数组以四空格缩进展开。未修改的对象保持行内形式，verbatim 字符串内容不会重新缩进。

```rust
let source = "a: [old, {keep: yes}]\n";
let updated = edit_value(source, "a.0", &Value::String("x\ny".into()))?;
```

结果:

```ktav
a: [
    ((
x
y
    ))
    {keep: yes}
]
```

#### 6. 无变化时保留非规范拼写

Integer 的值已经是 1。相等判断直接返回原始字节，包括 `+1` 及分隔符周围的空白，不应用可读性阈值。

```rust
let source = "n  : +1\n";
let updated = edit_value(source, "n", &Value::Integer("1".into()))?;
assert_eq!(updated, source);
```

结果:

```ktav
n  : +1
```

#### 7. BOM、CRLF 及无换行符的末行

返回值保留 BOM 和 CRLF，且不会增加末尾换行。以下转义表示使这些字节清晰可见。

```rust
let source = "\u{FEFF}port: 8080\r\nhost: db";
let updated = edit_value(source, "port", &Value::Integer("9090".into()))?;
```

结果:

```text
"\uFEFFport: 9090\r\nhost: db"
```

展开条件固定：替换中包含多行字符串、修改后行长增长并超过 `max(100, 原始行长)`，或单个替换字符串需要至少四个转义且编码开销达到 25%。宽度按 Unicode 标量值计算，而非 UTF-8 字节或终端列。原本已很长但没有增长的行保持不变。新层级使用四空格及源文本的 EOL。若 CR 或数据行 `))` 无法无损转换为独立行，则保留有效的行内候选；这些字符不会使所有行内编辑自动失败。

#### 8. 精确替换 Integer

源文本的 Integer 域是有符号 64 位。此值超过 JavaScript Number 的安全整数范围，但仍在 i64 内；应使用精确的宿主表示，而非已舍入的浮点值。超出 i64 的类型化 Integer 无法在源码中保持 Integer 身份，因此会被拒绝而不是截断。大型文本标识符应使用 String。Float 区分负零和正零；Object 的 no-op 相等忽略键顺序，而 Array 顺序仍然重要。

```rust
let updated = edit_value("n: 1\n", "n", &Value::Integer("9007199254740993".into()))?;
```

```ktav
n: 9007199254740993
```

### 0.9 的诊断与安全失败

源码坐标为包含 BOM 的 UTF-8 字节偏移，span 使用半开区间 `[start, end)`，行号从 1 开始。支持 LF、独立 CR 和 CRLF，且 CRLF 计为一个换行。一个 Unicode 字符可能占用多个字节。

未完成的 `a: [\n` 报告 `UnclosedCompound`、第 1 行和 span `3..5`：区间从最内层未闭合的起始括号开始，而非键或行的开头。原始字节 `61 3A 20 FF`（`a: ` 后接无效 UTF-8）报告 `InvalidUtf8`、第 1 行及覆盖区间 `3..4`；`line_text` 为 null，不会通过有损解码伪造文本。UTF-8 校验优先于语法处理。

非法文本路径（包括空路径或空文本段）报告 `EditInvalidPath`，偏移按路径本身的 UTF-8 字节计算。空路径段列表、缺失键、索引越界或穿过标量下钻报告 `EditNotFound`。对象中的数字段仍是键；数组索引仅接受 ASCII 数字，允许前导零，不允许正负号。

由 dotted/reopened 定义拼成的整个中间对象可能占据多个物理范围，因此报告 `EditUnsupported`；已有叶子值仍可修改。容器结构替换不能删除内部注释。失败调用不会返回部分补丁，也不会修改传入的源码。返回文本会重新解析并进行语义检查：目标必须等于替换值，其余值树必须与原文相等。

在绑定中，无效源码会先于路径验证报告普通解析错误信封。非法 wire 载荷是无虚构源码坐标的 `Message` 错误。wire 信封包含 `error`、`reason`、`line`、`line_text`、`span`、`path`、`body`、`canonical`、`spec_section`、`message`；宿主语言的访问器名称见错误章节。`body` 分别承载 `EditInvalidPath` 的十进制路径字节偏移、`EditNotFound` 的 null，以及 `EditUnsupported` 的拒绝原因。

坐标示例描述普通解析错误（`Error` / `ErrorEnvelope`）。Rust 编辑 API 本身返回 `EditError::{InvalidPath, NotFound, Unsupported}`：`InvalidPath { offset }` 指向路径字节，无效源码包装为 `Unsupported`。C ABI 会先验证源码，并向绑定公开普通源码错误信封。这些 Rust 变体不是 wire 信封的字段布局。

## 面向其他语言的 C ABI

Go、Java、PHP、C#、JS 和 Python 绑定共用 Rust 核心。默认关闭的 `cabi` feature 提供 wire 解码、文档操作和错误编码；绑定的 `cdylib` 中一次宏调用即可导出 C 符号。PyO3 和 N-API 等直接 Rust 桥接可以调用相同的安全操作，无需增加 FFI 层。

```text
// 绑定的 crates/cabi/src/lib.rs：
ktav::declare_cabi!();
```

**尚未发布的 0.9 准备版本使用 ABI 2**，包含十八个符号：十五个文档函数（`ktav_loads`、`ktav_loads_strict`、`ktav_dumps`、`ktav_dumps_force_strings`、`ktav_emit_canonical`、`ktav_format`、`ktav_canonical_from_source`、`ktav_edit_value`、`ktav_edit_value_segments`、`ktav_insert_value`、`ktav_insert_value_segments`、`ktav_remove_value`、`ktav_remove_value_segments`、`ktav_rename_key`、`ktav_rename_key_segments`）、`ktav_free`、`ktav_version` 和 `ktav_abi_version`。六个对象变更函数是 ABI 2 的增量导出：现有二进制文件无需改动即可继续工作，需要 insert/remove/rename 的绑定必须要求新符号——没有隐藏的纯实现回退。宿主必须拒绝 ABI 1 二进制文件，不得静默切换后端。这不会发布或更改绑定包版本。

两个编辑函数均接受三个带长度的字节输入：Ktav 源文档、路径及 JSON wire 替换值。第一个路径为 UTF-8 点分文本；第二个为字面解码字符串的 JSON 数组。带标记的数字（`$i` / `$f`）保留标量类型和精度；替换值本身可以是标量。成功返回 UTF-8 Ktav 源文本，而非 JSON 字符串，遵守与 `ktav::edit` 相同的无损编辑和可读性规则。变更系列新增 `ktav_insert_value(_segments)` 用于追加新的 Object 键、`ktav_remove_value(_segments)`（恰为两个输入缓冲区：source 与 path，无替换参数）以及 `ktav_rename_key(_segments)`，其 `new_key` 输入是 UTF-8 字面已解码键——不是 JSON，也不是路径语法。

所有错误仍使用十字段 JSON envelope。源诊断保留 0.9 原始字节坐标。编辑错误为 `EditInvalidPath`、`EditNotFound`、`EditAlreadyExists`（body 为 null；插入绝不覆盖）或 `EditUnsupported`，不编造源位置：`body` 对应十进制路径字节偏移、拒绝原因，或 `EditNotFound` 与 `EditAlreadyExists` 时的 null。源文档之外的 JSON/UTF-8 传输错误使用 `Message`；宿主以 `message` 作为错误文本。

完整签名、所有权及产物命名约定（`ktav_cabi-windows-amd64.dll`、`libktav_cabi-darwin-arm64.dylib`、`libktav_cabi-linux-amd64.so`、`$KTAV_LIB_PATH`）见 [docs/CABI.md](docs/CABI.md)。

## 架构

```
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── thin/             — arena-backed borrowed parse (parse_events)
├── render/           — pretty-printer, canonical writer, formatter
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error, ErrorKind, ErrorEnvelope, serde::Error
├── bin/ktav-fmt.rs   — the ktav-fmt command-line formatter
└── lib.rs            — glue: from_str / to_string / format_str / …
```

每个文件持有一个导出项;实现细节相对其父模块私有。

## Ktav **不**做、也永远不会做的事

- **内联的非空复合值**,比如 `x: { a: 1, b: 2 }`。它们会带来逗号,
  逗号又会带来转义。复合值保持多行。
- **锚点 / 别名 / 合并键**(`&anchor`、`*ref`、`<<:`)。任何一行
  若其含义依赖远处的声明,就不再自洽。若需要 DRY,请在代码里
  组合默认值。
- **文件包含**(`@include`、`!import`)。大型配置请在代码里包一层
  封装。
- **顶层数组。** 文档始终是对象。

## 安装

下面的依赖为已发布的 0.8.1，不包含新增路径编辑 API 或 0.9 错误模型。
准备中的 0.9.0 目前需要使用本源代码 checkout。

```toml
[dependencies]
ktav = "0.8.1"
serde = { version = "1", features = ["derive"] }
```

格式化器也可作为二进制取用,位于一个默认关闭的 feature 之后:

```sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
```

## 支持本项目

作者有许多构想,可能对全球 IT 广泛有益——不局限于 Ktav。实现这些
构想需要资金支持。如果您愿意提供帮助,请联系
**phpcraftdream@gmail.com**。

## 许可证

双许可：**MIT OR Apache-2.0**，由你选择其一。详见
[LICENSE-MIT](LICENSE-MIT) 和 [LICENSE-APACHE](LICENSE-APACHE)。

## 其他 Ktav 实现

- [`spec`](https://github.com/ktav-lang/spec) —— 规范 + 一致性测试套件
- [`csharp`](https://github.com/ktav-lang/csharp) —— C# / .NET(`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) —— Go(`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) —— Java / JVM(`io.github.ktav-lang:ktav`,Maven Central)
- [`js`](https://github.com/ktav-lang/js) —— JS / TS(`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) —— PHP(`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) —— Python(`pip install ktav`)
