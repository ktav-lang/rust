// README units: rust-usage section.
export const rustUsage = [
  {
    id: 'rust-usage-heading',
    join: 'block',
    en: `## Using it from Rust`,
    ru: `## Использование из Rust`,
    zh: `## 在 Rust 中使用`,
  },
  {
    id: 'rust-usage-serde-intro-1',
    join: 'block',
    en: `Ktav is serde-native.`,
    ru: `Ktav — serde-нативный.`,
    zh: `Ktav 原生支持 serde。`,
  },
  {
    id: 'rust-usage-serde-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Any type implementing \`Serialize\` / \`Deserialize\`
(including \`#[derive]\`-generated ones) round-trips through Ktav out of
the box.`,
    ru: `Любой тип, реализующий \`Serialize\` /
\`Deserialize\` (включая сгенерированные через \`#[derive]\`),
round-trip-ится через Ktav из коробки.`,
    zh: `任何实现了 \`Serialize\` / \`Deserialize\` 的类型
(包括 \`#[derive]\` 生成的)都可以开箱即用地通过 Ktav 完成
round-trip。`,
  },
  {
    id: 'parse-heading',
    join: 'block',
    en: `### Parse — decode straight into a typed struct`,
    ru: `### Парсинг — декод сразу в типизированную структуру`,
    zh: `### 解析 —— 直接解码到类型化结构体`,
  },
  {
    id: 'preamble-060',
    join: 'block',
    common: `\`\`\`rust
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

const SRC: &str = "\\
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
\`\`\``,
  },
  {
    id: 'walk-heading',
    join: 'block',
    en: `### Walk — match on the dynamic \`Value\` enum`,
    ru: `### Обход — match по динамическому enum \`Value\``,
    zh: `### 遍历 —— 在动态 \`Value\` 枚举上 match`,
  },
  {
    id: 'preamble-062',
    join: 'block',
    en: `\`\`\`rust
use ktav::value::Value;

let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("top is always an object") };

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
\`\`\``,
    ru: `\`\`\`rust
use ktav::value::Value;

let v = ktav::parse(SRC)?;
let Value::Object(top) = &v else { unreachable!("top — всегда object") };

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
\`\`\``,
    zh: `\`\`\`rust
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
\`\`\``,
  },
  {
    id: 'build-render-heading',
    join: 'block',
    en: `### Build & render — construct a document in code`,
    ru: `### Билд + рендер — собираем документ в коде`,
    zh: `### 构建并渲染 —— 用代码搭建文档`,
  },
  {
    id: 'preamble-064',
    join: 'block',
    common: `\`\`\`rust
use ktav::value::{ObjectMap, Value};

let mut top = ObjectMap::default();
top.insert("name".into(),  Value::String("frontend".into()));
top.insert("port".into(),  Value::Integer("8443".into()));
top.insert("tls".into(),   Value::Bool(true));
top.insert("ratio".into(), Value::Float("0.95".into()));
top.insert("notes".into(), Value::Null);

let text = ktav::render::render(&Value::Object(top))?;
\`\`\``,
  },
  {
    id: 'build-render-prefer-serde',
    join: 'block',
    en: `For typical app use prefer the serde path — \`ktav::to_string(&cfg)\` —
and reach for \`Value\` only when the schema is dynamic.`,
    ru: `В обычных сценариях используйте serde-путь — \`ktav::to_string(&cfg)\`.
К \`Value\` обращайтесь только когда схема динамическая.`,
    zh: `通常请走 serde 路径(\`ktav::to_string(&cfg)\`);仅在 schema
是动态的时候才需要直接操作 \`Value\`。`,
  },
  {
    id: 'entry-points-1',
    join: 'block',
    en: `Four public entry points: [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav) for reading, [\`to_string\`](https://docs.rs/ktav) /
[\`to_file\`](https://docs.rs/ktav) for writing.`,
    ru: `Четыре публичных entry point-а: [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav) — для чтения,
[\`to_string\`](https://docs.rs/ktav) / [\`to_file\`](https://docs.rs/ktav) —
для записи.`,
    zh: `四个公共入口:读取用 [\`from_str\`](https://docs.rs/ktav) /
[\`from_file\`](https://docs.rs/ktav),写入用
[\`to_string\`](https://docs.rs/ktav) / [\`to_file\`](https://docs.rs/ktav)。`,
  },
  {
    id: 'entry-points-2',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `A complete runnable
example lives in [\`examples/basic.rs\`](examples/basic.rs).`,
    ru: `Полный запускаемый пример — в
[\`examples/basic.rs\`](examples/basic.rs).`,
    zh: `完整可运行示例:[\`examples/basic.rs\`](examples/basic.rs)。`,
  },
  {
    id: 'errors-heading',
    join: 'block',
    en: `### Inspect errors — structured variants for tooling`,
    ru: `### Анализ ошибок — структурированные варианты для тулинга`,
    zh: `### 检视错误 —— 给工具链使用的结构化变体`,
  },
  {
    id: 'errors-structured-intro',
    join: 'block',
    en: `\`Error::Structured(ErrorKind)\` carries a typed category plus a
byte-offset \`Span\` for every parse failure, so editors and linters
can highlight the exact offending range instead of the whole line.`,
    ru: `\`Error::Structured(ErrorKind)\` несёт типизированную категорию плюс
byte-offset \`Span\` для каждого parse-фейла, так что редакторы и
линтеры могут подсветить именно offending диапазон, а не всю строку.`,
    zh: `\`Error::Structured(ErrorKind)\` 为每次解析失败携带类型化的类别加字节
偏移 \`Span\`,因此编辑器和 linter 可以高亮恰好出错的范围,而不是
整行。`,
  },
  {
    id: 'preamble-070',
    join: 'block',
    en: `\`\`\`rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\\nport: 443\\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("line {line}: duplicate key {key:?}");
        println!("offending bytes: {:?}", span.slice(src));   // -> Some("port")
        let (l, c) = span.line_col(src);                      // 1-based / 0-based byte col
        println!("highlight at {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
\`\`\``,
    ru: `\`\`\`rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\\nport: 443\\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("строка {line}: дубликат ключа {key:?}");
        println!("offending байты: {:?}", span.slice(src));    // -> Some("port")
        let (l, c) = span.line_col(src);                       // line 1-based, col 0-based по байтам
        println!("подсветить в {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
\`\`\``,
    zh: `\`\`\`rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\\nport: 443\\n";
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
\`\`\``,
  },
  {
    id: 'errors-variants-list-1',
    join: 'block',
    en: `Variants: \`MissingSeparatorSpace\`, \`InvalidTypedScalar\`, \`DuplicateKey\`,
\`KeyPathConflict\`, \`EmptyKey\`, \`InvalidKey\`, \`UnclosedCompound\`,
\`UnbalancedBracket\`, \`InlineNonEmptyCompound\`, \`MissingSeparator\`,
\`LossyScalar\` (strict mode only, see below), \`Other\`.`,
    ru: `Варианты: \`MissingSeparatorSpace\`, \`InvalidTypedScalar\`, \`DuplicateKey\`,
\`KeyPathConflict\`, \`EmptyKey\`, \`InvalidKey\`, \`UnclosedCompound\`,
\`UnbalancedBracket\`, \`InlineNonEmptyCompound\`, \`MissingSeparator\`,
\`LossyScalar\` (только в строгом режиме, см. ниже), \`Other\`.`,
    zh: `变体:\`MissingSeparatorSpace\`、\`InvalidTypedScalar\`、\`DuplicateKey\`、
\`KeyPathConflict\`、\`EmptyKey\`、\`InvalidKey\`、\`UnclosedCompound\`、
\`UnbalancedBracket\`、\`InlineNonEmptyCompound\`、\`MissingSeparator\`、
\`LossyScalar\`（仅严格模式，见下文）、\`Other\`。`,
  },
  {
    id: 'errors-variants-list-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The enum is
\`#[non_exhaustive]\` — always include a \`_ =>\` arm.`,
    ru: `Enum помечен
\`#[non_exhaustive]\` — обязателен arm \`_ =>\`.`,
    zh: `该枚举标注
\`#[non_exhaustive]\` —— 必须包含 \`_ =>\` 分支。`,
  },
  {
    id: 'errors-variants-list-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`Error::line()\` /
\`Error::span()\` are convenience accessors when the variant doesn't
matter.`,
    ru: `\`Error::line()\` /
\`Error::span()\` — convenience accessors для случаев когда вариант
неважен.`,
    zh: `变体不重要时可使用便捷
访问器 \`Error::line()\` / \`Error::span()\`。`,
  },
  {
    id: 'errors-variants-list-4',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The \`Display\` impl produces the same human-readable string the
legacy \`Error::Syntax(_)\` did, so existing string-based callers keep
working.`,
    ru: `\`Display\` выдаёт те же читаемые строки, что и legacy
\`Error::Syntax(_)\`, поэтому существующие string-based вызывающие
работают без изменений.`,
    zh: `\`Display\` 输出的字符串与
遗留 \`Error::Syntax(_)\` 完全一致,因此原有基于字符串的调用方无需修改
即可继续工作。`,
  },
  {
    id: 'errors-example-pointer',
    join: 'block',
    en: `A complete runnable example walks all variants:
[\`examples/errors.rs\`](examples/errors.rs) — \`cargo run --example errors\`.`,
    ru: `Полный запускаемый пример проходит по всем вариантам:
[\`examples/errors.rs\`](examples/errors.rs) — \`cargo run --example errors\`.`,
    zh: `完整可运行示例遍历所有变体:
[\`examples/errors.rs\`](examples/errors.rs) —— \`cargo run --example errors\`。`,
  },
  {
    id: 'envelope-heading',
    join: 'block',
    en: `### One JSON envelope for every structured error`,
    ru: `### Один JSON-конверт для любой структурированной ошибки`,
    zh: `### 任何结构化错误共用一个 JSON 信封`,
  },
  {
    id: 'envelope-intro',
    join: 'block',
    en: `The accessors above are Rust-only. \`ErrorEnvelope\` is the wire
contract for everyone else: one JSON object, ten fields, always all
ten, in a fixed order — \`error\`, \`reason\`, \`line\`, \`line_text\`,
\`span\`, \`path\`, \`body\`, \`canonical\`, \`spec_section\`,
\`message\`.`,
    ru: `Аксессоры выше существуют только в Rust. \`ErrorEnvelope\` — контракт
для всех остальных: один JSON-объект, десять полей, всегда все
десять, в фиксированном порядке — \`error\`, \`reason\`, \`line\`,
\`line_text\`, \`span\`, \`path\`, \`body\`, \`canonical\`,
\`spec_section\`, \`message\`.`,
    zh: `上面的访问器只存在于 Rust。\`ErrorEnvelope\` 是给其余各方的传输契约:
一个 JSON 对象,十个字段,永远是全部十个,顺序固定 —— \`error\`、
\`reason\`、\`line\`、\`line_text\`、\`span\`、\`path\`、\`body\`、
\`canonical\`、\`spec_section\`、\`message\`。`,
  },
  {
    id: 'envelope-snippet',
    join: 'block',
    common: `\`\`\`rust
use ktav::{parse, ErrorEnvelope};

let src = "a: 1.10\\n";
if let Err(e) = ktav::parse_strict(src) {
    println!("{}", ErrorEnvelope::from_error(&e, src).to_json());
}
\`\`\``,
  },
  {
    id: 'envelope-snippet-output',
    join: 'block',
    common: `\`\`\`json
{"error":"LossyScalar","reason":null,"line":1,"line_text":"a: 1.10",
 "span":{"start":0,"end":7},"path":null,"body":"1.10",
 "canonical":"1.1","spec_section":"§3.6/§5.2",
 "message":"Syntax error: Line 1: LossyScalar: '1.10' would be …"}
\`\`\``,
  },
  {
    id: 'envelope-message',
    join: 'block',
    en: `\`message\` is the last field and the only one that is never
\`null\`: it carries this error's \`Display\` rendering verbatim. A
binding shows it to the user as-is rather than assembling prose from
the structured fields, so the same document produces the same error
text in every language. The nine older fields keep the positions they
shipped with — \`message\` was appended, not inserted.`,
    ru: `\`message\` — последнее поле и единственное, которое никогда не
\`null\`: в нём дословно лежит \`Display\`-представление этой ошибки.
Биндинг показывает его пользователю как есть, а не собирает текст сам
из структурированных полей, поэтому один и тот же документ даёт
одинаковый текст ошибки на любом языке. Девять прежних полей сохранили
свои позиции — \`message\` дописано в конец, а не вставлено в середину.`,
    zh: `\`message\` 是最后一个字段,也是唯一永远不为 \`null\` 的字段:它逐字
携带该错误的 \`Display\` 渲染结果。绑定层直接把它展示给用户,而不是自己
从结构化字段拼装文字,因此同一份文档在任何语言下都会给出相同的错误文本。
原有的九个字段保持各自的位置 —— \`message\` 是追加的,而非插入的。`,
  },
  {
    id: 'envelope-nulls',
    join: 'block',
    en: `Absent information is an explicit \`null\`, never an omitted key, so a
consumer can read every field positionally without negotiating a
schema first.`,
    ru: `Отсутствующие сведения — явный \`null\`, а не пропущенный ключ, поэтому
потребитель читает каждое поле позиционно, без предварительного
согласования схемы.`,
    zh: `缺失的信息是显式的 \`null\`,而不是省略键,因此使用方无需事先协商模式
即可按位置读取每个字段。`,
  },
  {
    id: 'envelope-span-unit',
    join: 'block',
    en: `\`span\` is \`{"start":N,"end":M}\` in **byte offsets into the UTF-8
source**, not UTF-16 code units — the same unit [\`Span\`](https://docs.rs/ktav)
itself uses. An LSP consumer either converts, or negotiates
\`positionEncoding: "utf-8"\`.`,
    ru: `\`span\` — это \`{"start":N,"end":M}\` в **байтовых смещениях по
UTF-8-исходнику**, а не в code units UTF-16: та же единица, что у самого
[\`Span\`](https://docs.rs/ktav). Потребителю LSP нужно либо
конвертировать, либо договариваться о \`positionEncoding: "utf-8"\`.`,
    zh: `\`span\` 是 \`{"start":N,"end":M}\`,单位为 **UTF-8 源文本中的字节
偏移**,而不是 UTF-16 code unit —— 与 [\`Span\`](https://docs.rs/ktav)
本身一致。LSP 使用方要么自行转换,要么协商
\`positionEncoding: "utf-8"\`。`,
  },
  {
    id: 'envelope-path-rule',
    join: 'block',
    en: `\`path\` is an **array of exact decoded key segments, never a joined
string**. A key literally named \`a.b\` is one segment and cannot be
confused with a two-segment path — there is no separator in the wire
contract to be ambiguous about.`,
    ru: `\`path\` — **массив точных декодированных сегментов ключа, а не
склеенная строка**. Ключ, буквально названный \`a.b\`, — это один
сегмент, и спутать его с путём из двух нельзя: в контракте просто нет
разделителя, вокруг которого возникла бы двусмысленность.`,
    zh: `\`path\` 是**精确解码后的键段数组,绝不是拼接字符串**。字面名为
\`a.b\` 的键是一个段,不会与两段路径混淆 —— 传输契约里根本没有可供
产生歧义的分隔符。`,
  },
  {
    id: 'envelope-writer-note',
    join: 'block',
    en: `Writer rejections use the same envelope: \`reason\` carries the § 5.9.0
reason code (\`NonFiniteFloat\`, \`EmptyKeyName\`, …) and the two
rejections are named apart — \`UnrepresentableAt\` when the writer can
say where the offending node is (it fills \`path\` too),
\`Unrepresentable\` when it cannot.`,
    ru: `Отказы писателя используют тот же конверт: \`reason\` несёт код причины
из § 5.9.0 (\`NonFiniteFloat\`, \`EmptyKeyName\`, …), а сами отказы
названы раздельно — \`UnrepresentableAt\`, когда писатель может указать
узел (тогда он заполняет и \`path\`), и \`Unrepresentable\`, когда не
может.`,
    zh: `写入器的拒绝使用同一个信封:\`reason\` 携带 § 5.9.0 的原因码
(\`NonFiniteFloat\`、\`EmptyKeyName\` 等),两种拒绝分别命名 ——
写入器能指出出错节点时为 \`UnrepresentableAt\`(此时也会填充
\`path\`),不能指出时为 \`Unrepresentable\`。`,
  },
  {
    id: 'envelope-rendering',
    join: 'block',
    en: `Rendering is \`to_json()\` (or \`push_json(&mut String)\` to append into
a buffer you own). It is valid JSON for any payload — every string is
escaped per RFC 8259 — and no serde dependency is involved.`,
    ru: `Печать — \`to_json()\` (или \`push_json(&mut String)\`, чтобы дописать в
собственный буфер). Результат — валидный JSON для любого содержимого:
каждая строка экранируется по RFC 8259, зависимость от serde не
задействована.`,
    zh: `渲染用 \`to_json()\`(或 \`push_json(&mut String)\` 追加进你自己的
缓冲区)。对任何载荷它都是合法 JSON —— 每个字符串都按 RFC 8259
转义 —— 且不涉及 serde 依赖。`,
  },
  {
    id: 'strict-mode-heading',
    join: 'block',
    en: `### Strict mode — catch silently canonicalised numbers`,
    ru: `### Строгий режим — ловим молча канонизированные числа`,
    zh: `### 严格模式 —— 捕获被静默规范化的数字`,
  },
  {
    id: 'strict-mode-intro-1',
    join: 'block',
    en: `Types are inferred from a scalar's lexical form, and inferred numbers
are canonicalised: \`version: 1.10\` parses as \`Float(1.1)\` and
\`zip: 01234\` as \`Integer(1234)\`.`,
    ru: `Типы выводятся по лексической форме скаляра, а выведенные числа
канонизируются: \`version: 1.10\` разбирается как \`Float(1.1)\`, а
\`zip: 01234\` — как \`Integer(1234)\`.`,
    zh: `类型由标量的词法形式推断，且推断出的数字会被规范化：\`version: 1.10\`
解析为 \`Float(1.1)\`，\`zip: 01234\` 解析为 \`Integer(1234)\`。`,
  },
  {
    id: 'strict-mode-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The default \`parse()\` does this
silently, so writing the document back out rewrites it.`,
    ru: `Обычный \`parse()\` делает это молча,
поэтому обратная запись документа его переписывает.`,
    zh: `默认的
\`parse()\` 会静默完成这一过程，因此把文档写回去就会改写它。`,
  },
  {
    id: 'strict-mode-rejects',
    join: 'block',
    en: `\`parse_strict()\` rejects such **lossy scalars** instead:`,
    ru: `\`parse_strict()\` вместо этого отвергает такие **скаляры с потерей**:`,
    zh: `\`parse_strict()\` 则拒绝这类**有损标量**：`,
  },
  {
    id: 'preamble-080',
    join: 'block',
    en: `\`\`\`rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "zip: 01234\\n";

assert!(parse(src).is_ok());                 // Integer(1234) — leading zero gone

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("expected LossyScalar, got {other:?}"),
}
\`\`\``,
    ru: `\`\`\`rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "zip: 01234\\n";

assert!(parse(src).is_ok());                 // Integer(1234) — ведущий ноль потерян

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("ожидался LossyScalar, получено {other:?}"),
}
\`\`\``,
    zh: `\`\`\`rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "zip: 01234\\n";

assert!(parse(src).is_ok());                 // Integer(1234) —— 前导零丢失

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("01234", "1234"));
    }
    other => panic!("期望 LossyScalar，实际为 {other:?}"),
}
\`\`\``,
  },
  {
    id: 'strict-mode-fix-options-1',
    join: 'block',
    en: `Fix either by appending \`::\` to keep the value a String
(\`zip:: 01234\`) or by writing the canonical number.`,
    ru: `Исправить можно либо дописав \`::\`, чтобы значение осталось строкой
(\`zip:: 01234\`), либо записав число в канонической форме.`,
    zh: `修复方式有两种：追加 \`::\` 让该值保持字符串（\`zip:: 01234\`），或直接
写成规范数字。`,
  },
  {
    id: 'strict-mode-fix-options-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Any document
\`parse_strict()\` accepts yields exactly the same \`Value\` tree as
\`parse()\`, so strict mode is a validation gate, not a different
dialect.`,
    ru: `Любой
документ, принятый \`parse_strict()\`, даёт ровно то же дерево \`Value\`,
что и \`parse()\` — строгий режим это проверочный шлюз, а не другой
диалект.`,
    zh: `凡是 \`parse_strict()\` 接受的文档，其产生的 \`Value\` 树
与 \`parse()\` 完全相同 —— 严格模式是一道校验闸门，而非另一种方言。`,
  },
  {
    id: 'strict-mode-fix-options-3',
    join: { en: 'flow', ru: 'flow', zh: 'tight' },
    en: `The serde path (\`from_str\`) has no strict variant yet.`,
    ru: `У serde-пути (\`from_str\`) строгого варианта пока нет.`,
    zh: `serde 路径（\`from_str\`）目前尚无严格模式变体。`,
  },
  {
    id: 'stream-parse-heading',
    join: 'block',
    en: `### Stream parse — events without an intermediate tree`,
    ru: `### Stream-парсинг — события без промежуточного дерева`,
    zh: `### 流式解析 —— 不构建中间树的事件流`,
  },
  {
    id: 'stream-parse-intro-1',
    join: 'block',
    en: `\`parse_events\` invokes a callback for each parse event, with strings
borrowed directly into the input buffer — no allocation per event, no
intermediate \`Value\` tree.`,
    ru: `\`parse_events\` вызывает callback на каждое событие парсинга, со
строками заимствующимися напрямую из input-буфера — без аллокации на
событие, без промежуточного \`Value\`-дерева.`,
    zh: `\`parse_events\` 对每个解析事件调用回调,字符串直接从 input 缓冲区
借用 —— 不在每个事件上分配,也不构建中间 \`Value\` 树。`,
  },
  {
    id: 'stream-parse-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Useful when you don't need the full document:
counting keys, streaming to another format, building a custom shape.`,
    ru: `Полезно когда полный
документ не нужен: подсчёт ключей, стриминг в другой формат, построение
custom-shape-а.`,
    zh: `在不需要完整
文档时很有用:统计键、流式转换到其他格式、构建自定义形状。`,
  },
  {
    id: 'preamble-087',
    join: 'block',
    common: `\`\`\`rust
use ktav::{parse_events, ParseEvent};

let src = "port: 8080\\nhost: example.com\\n";
let mut keys = Vec::new();
parse_events(src, |ev| {
    if let ParseEvent::Key(k) = ev {
        keys.push(k.to_string());
    }
})?;
assert_eq!(keys, ["port", "host"]);
\`\`\``,
  },
  {
    id: 'stream-parse-root-events-1',
    join: 'block',
    en: `The root is \`BeginObject\`/\`EndObject\` or \`BeginArray\`/\`EndArray\`
depending on the document's first content line (an Object here, since
\`port: 8080\` is a pair); nested compounds bracket their contents the
same way.`,
    ru: `Корень — \`BeginObject\`/\`EndObject\` или \`BeginArray\`/\`EndArray\`, в
зависимости от первой содержательной строки документа (здесь Object,
так как \`port: 8080\` — это пара); вложенные компаунды обрамляют своё
содержимое так же.`,
    zh: `根据文档第一条内容行的形状,根节点是 \`BeginObject\`/\`EndObject\` 或
\`BeginArray\`/\`EndArray\`(此处因为 \`port: 8080\` 是一个 pair,所以是
Object);嵌套复合值以同样方式括起自身内容。`,
  },
  {
    id: 'stream-parse-root-events-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`ParseEvent\` is \`#[non_exhaustive]\`.`,
    ru: `\`ParseEvent\` помечен
\`#[non_exhaustive]\`.`,
    zh: `\`ParseEvent\` 标注
\`#[non_exhaustive]\`。`,
  },
  {
    id: 'stream-parse-root-events-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `A complete runnable
example with depth tracking
and a pretty-printer:
[\`examples/events.rs\`](examples/events.rs) — \`cargo run --example events\`.`,
    ru: `Полный запускаемый пример с tracking-ом глубины
и pretty-print-ом:
[\`examples/events.rs\`](examples/events.rs) — \`cargo run --example events\`.`,
    zh: `完整的、带深度跟踪
和漂亮打印的可运行示例:
[\`examples/events.rs\`](examples/events.rs) —— \`cargo run --example events\`。`,
  },
  {
    id: 'preamble-091',
    join: 'block',
    en: `### Numbers`,
    ru: `### Числа`,
    zh: `### 数字`,
  },
  {
    id: 'stream-numbers-notes-1',
    join: 'block',
    en: `Rust numeric types (\`u8\`..\`u128\`, \`i8\`..\`i128\`, \`usize\`, \`isize\`, \`f32\`,
\`f64\`) serialize to Ktav as bare numbers: \`port: 8080\`, \`ratio: 0.5\`.`,
    ru: `Числовые Rust-типы (\`u8\`..\`u128\`, \`i8\`..\`i128\`, \`usize\`, \`isize\`, \`f32\`,
\`f64\`) сериализуются в Ktav как голые числа: \`port: 8080\`,
\`ratio: 0.5\`.`,
    zh: `Rust 数值类型(\`u8\`..\`u128\`、\`i8\`..\`i128\`、\`usize\`、\`isize\`、\`f32\`、
\`f64\`)会以裸数字序列化到 Ktav:\`port: 8080\`、\`ratio: 0.5\`。`,
  },
  {
    id: 'stream-numbers-notes-2',
    join: { en: 'tight', ru: 'flow', zh: 'none' },
    en: `Coming back, a bare integer/decimal body deserializes straight into
the target numeric type; a value that arrived as a string (e.g. forced
with \`::\`) is still accepted via \`FromStr\`.`,
    ru: `На обратном пути голое целое/десятичное тело
десериализуется прямо в нужный числовой тип; значение, пришедшее
строкой (например, форсированное через \`::\`), по-прежнему принимается
через \`FromStr\`.`,
    zh: `回程时,
裸整数/小数 body 直接反序列化为目标数值类型;以字符串形式到达的值
(例如用 \`::\` 强制的)仍通过 \`FromStr\` 接受。`,
  },
  {
    id: 'stream-numbers-notes-3',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `\`NaN\` / \`±Infinity\` are
rejected by the serializer (Ktav does not represent them).`,
    ru: `\`NaN\` / \`±Infinity\` отвергаются сериализатором
(Ktav их не представляет).`,
    zh: `\`NaN\` / \`±Infinity\` 会被
序列化器拒绝(Ktav 不表示这些值)。`,
  },
];
