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

**规范:** 本 crate 实现 **Ktav 0.7.0**。格式与 crate 彼此独立地
版本化与维护——规范正文见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec),0.7.0 的变更见
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
(无下划线、无前导零/`+`),`Float` 保存能还原精确 `f64` 位模式的最短
十进制形式——`+1_000` 变为 `Integer("1000")`,`1.0e+2` 变为
`Float("100.0")`。`Value` 层的 `Integer` 覆盖 i64 范围;比 i64 更宽的
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
一个 JSON 对象,九个字段,永远是全部九个,顺序固定 —— `error`、
`reason`、`line`、`line_text`、`span`、`path`、`body`、
`canonical`、`spec_section`。

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
 "canonical":"1.1","spec_section":"§3.6/§5.2"}
```

缺失的信息是显式的 `null`,而不是省略键,因此使用方无需事先协商模式
即可按位置读取每个字段。

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
解析为 `Float(1.1)`，`zip: 01234` 解析为 `Integer(1234)`。默认的
`parse()` 会静默完成这一过程，因此把文档写回去就会改写它。

`parse_strict()` 则拒绝这类**有损标量**：

```rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "zip: 01234\n";

assert!(parse(src).is_ok());                 // Integer(1234) —— 前导零丢失

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
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

## 面向其他语言的 C ABI

六个语言绑定——Go、Java、PHP、C#、JS 与 Python——加载一个构建在
本 crate 之上的小型原生库。垫片中可共享的一半位于默认关闭的
`cabi` feature 之后:`ktav::cabi` 模块承载 wire 解码、六项文档
操作与错误编码,而绑定 cdylib 中的一次宏调用即可展开全部导出符号。

```text
// crates/cabi/src/lib.rs of a binding — the whole body:
ktav::declare_cabi!();
```

展开导出九个符号:六个文档操作函数(`ktav_loads`、
`ktav_loads_strict`、`ktav_dumps`、`ktav_dumps_force_strings`、
`ktav_emit_canonical`、`ktav_format`)、`ktav_free`、
`ktav_version` 与 `ktav_abi_version`。每个错误都以九字段 JSON
信封传递,宿主无需嗅探它是纯文本还是 JSON;`ktav_abi_version()`
让宿主拒绝过期的原生库而不是破坏内存。完整契约——签名、所有权、
错误编码、产物命名约定(`ktav_cabi-windows-amd64.dll`、
`libktav_cabi-darwin-arm64.dylib`、`libktav_cabi-linux-amd64.so`、
`$KTAV_LIB_PATH` 覆盖)——见
[docs/CABI.md](docs/CABI.md)。

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

```toml
[dependencies]
ktav = "0.7.1"
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
