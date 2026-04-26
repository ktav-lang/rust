# Ktav (כְּתָב)

> 一种朴素的配置格式。形态上接近 JSON5,但不带引号、不用逗号,
> 以点分键表达嵌套。原生 `serde` 集成。

**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**

**规范:** 本 crate 实现 **Ktav 0.1**。格式与 crate 彼此独立地
版本化与维护——规范正文见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec)。

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
# comment              — any line starting with '#'
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
# `::` 强制将值解释为字面量字符串
pattern:: [a-z]+
```

### 数字

数字不加引号。在 `Value` 层,它们仍是字符串;serde 会在反序列化
时通过 `FromStr` 将其解析为目标 Rust 类型(`u16`、`i64`、`f64`……),
并在序列化时通过 `Display` 进行格式化。

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
# Value::Bool(true)
on: true
# Value::Bool(false)
off: false
# Value::String("True")
capitalized: True
# Value::String("FALSE")
yelling:    FALSE
# Value::String("true")
literal:: true
```

### Null:`null`

严格小写。对应 Rust 侧的 `Option::None`,以及 unit 类型 `()`。

```text
# Value::Null
label: null
# Value::String("Null")
capitalized: Null
# Value::String("null")
literal:: null
```

序列化时,`Option::None` 会输出为 `null`。若你希望该字段干脆缺省,
可以加上 `#[serde(skip_serializing_if = "Option::is_none")]`。

### 空对象 / 空数组

**唯一**允许的内联复合值——没什么要分隔,不需要逗号。

```text
# 空对象
meta: {}
# 空数组
tags: []
```

### 与关键字同形的字符串需要 `::`

若字符串内容恰好等于关键字(`true`、`false`、`null`),或以 `{`、
`[` 开头,**序列化器会自动输出 `::`**,以保证 round-trip 无损。
写入端请用同样的方式:

```text
# 字符串 "true",而非 Bool
flag:: true
# 字符串 "null",而非 Null
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
# rejected — inline non-empty compound
server: { host: 127.0.0.1, port: 8080 }
tags: [primary, eu, prod]

# accepted — multi-line form
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
port:i 8080
ratio:f 0.75
tls: true
tags: [
    prod
    eu-west-1
]
db.host: primary.internal
db.timeout:i 30
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

### 类型化标记

Rust 数值类型(`u8`..`u128`、`i8`..`i128`、`usize`、`isize`、`f32`、
`f64`)会以显式的类型化标记序列化到 Ktav:`port:i 8080`、
`ratio:f 0.5`。反序列化接受*两种*形式 —— 带标记的以及纯字符串形式;
未带标记的旧文档仍能正常工作,与以前一致。`NaN` / `±Infinity` 会被
序列化器拒绝(Ktav 0.1.0 不表示这些值)。

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
  port: "20082"
}
```

在 `Value` 层,所有标量都是字符串;当你反序列化到 `u16` / `bool` /
`f64` / …… 时,由 serde 负责把它们解析为数值或布尔类型。

### 2. 点分键 = 嵌套对象

```text
server.host: 127.0.0.1
server.port: 8080
app.debug: true
```
```json5
{
  server: { host: "127.0.0.1", port: "8080" },
  app: { debug: "true" }
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
    port: "8080",
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
    { host: "a.example", port: "1080" },
    { host: "b.example", port: "1080" }
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
# top-level comment
port: 8080

items: [
    # this comment does not break the array
    a
    b
]
```

注释为整行,以 `#` 开头。不支持行内注释——太容易与值混淆。

### 9. 多行字符串:`( ... )` 与 `(( ... ))`

跨越多行的值放在圆括号里。开起行与关闭行**不**属于值。

`(` ... `)` —— 剥除公共前导缩进,所以你可以按照周围代码的缩进
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

序列化:任何含 `\n` 的字符串都以 `(( ... ))` 输出,从而保证
round-trip 字节级无损。不含换行的字符串使用常规的单行形式。

限制:一行若其 trim 后的内容恰好等于 `)` / `))`,总会关闭块,因此
这种字面量无法作为内容出现——除非借助外部文件。

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
# unit variant — just the name
mode: fast

# newtype variant — single-entry object
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

## 架构

```
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── render/           — pretty-printer (Value → text)
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error + serde::Error impls
└── lib.rs            — glue: from_str / from_file / to_string / to_file
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

发布后:

```toml
[dependencies]
ktav = "0.1"
serde = { version = "1", features = ["derive"] }
```

## 支持本项目

作者有许多构想,可能对全球 IT 广泛有益——不局限于 Ktav。实现这些
构想需要资金支持。如果您愿意提供帮助,请联系
**phpcraftdream@gmail.com**。

## 许可证

MIT。见 [LICENSE](LICENSE)。

## 其他 Ktav 实现

- [`spec`](https://github.com/ktav-lang/spec) —— 规范 + 一致性测试套件
- [`csharp`](https://github.com/ktav-lang/csharp) —— C# / .NET(`dotnet add package Ktav`)
- [`golang`](https://github.com/ktav-lang/golang) —— Go(`go get github.com/ktav-lang/golang`)
- [`java`](https://github.com/ktav-lang/java) —— Java / JVM(`io.github.ktav-lang:ktav`,Maven Central)
- [`js`](https://github.com/ktav-lang/js) —— JS / TS(`npm install @ktav-lang/ktav`)
- [`php`](https://github.com/ktav-lang/php) —— PHP(`composer require ktav-lang/ktav`)
- [`python`](https://github.com/ktav-lang/python) —— Python(`pip install ktav`)
