>>>>> lang=en
Writer rejections use the same envelope: `reason` carries the § 5.9.0
reason code (`NonFiniteFloat`, `EmptyKeyName`, …) and the two
rejections are named apart — `UnrepresentableAt` when the writer can
say where the offending node is (it fills `path` too),
`Unrepresentable` when it cannot.

Rendering is `to_json()` (or `push_json(&mut String)` to append into
a buffer you own). It is valid JSON for any payload — every string is
escaped per RFC 8259 — and no serde dependency is involved.

### Strict mode — catch silently canonicalised numbers

Types are inferred from a scalar's lexical form, and inferred numbers
are canonicalised: `version: 1.10` parses as `Float(1.1)`. A decimal
with a redundant leading zero is the one exception — `zip: 01234` is the
String `"01234"` (§ 5.2), because dropping that zero would destroy an
identifier. The default `parse()` does this
silently, so writing the document back out rewrites it.

`parse_strict()` rejects such **lossy scalars** instead:

```rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "version: 1.10\n";

>>>>> lang=ru
Отказы писателя используют тот же конверт: `reason` несёт код причины
из § 5.9.0 (`NonFiniteFloat`, `EmptyKeyName`, …), а сами отказы
названы раздельно — `UnrepresentableAt`, когда писатель может указать
узел (тогда он заполняет и `path`), и `Unrepresentable`, когда не
может.

Печать — `to_json()` (или `push_json(&mut String)`, чтобы дописать в
собственный буфер). Результат — валидный JSON для любого содержимого:
каждая строка экранируется по RFC 8259, зависимость от serde не
задействована.

### Строгий режим — ловим молча канонизированные числа

Типы выводятся по лексической форме скаляра, а выведенные числа
канонизируются: `version: 1.10` разбирается как `Float(1.1)`. Десятичный
литерал с избыточным ведущим нулём — единственное исключение: `zip: 01234`
это String `"01234"` (§ 5.2), потому что отбрасывание этого нуля
уничтожило бы идентификатор. Обычный `parse()` делает это молча,
поэтому обратная запись документа его переписывает.

`parse_strict()` вместо этого отвергает такие **скаляры с потерей**:

```rust
use ktav::{parse, parse_strict, Error, ErrorKind};

let src = "version: 1.10\n";

>>>>> lang=zh
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

