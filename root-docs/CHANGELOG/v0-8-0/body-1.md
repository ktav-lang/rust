>>>>> lang=en
## [0.8.0] — 2026-09-23

Two things ship together. A decimal with a redundant leading zero no
longer infers a number — `zip: 01234` parses to the String
`"01234"`, not `Integer(1234)` — which changes what a document
means and is why this is a minor bump rather than a patch. Alongside it,
the `cabi` feature turns the six language bindings' private copies of
the C ABI shim into one macro invocation against this crate; that half
is purely additive and off by default, so a default build still pulls no
`serde_json`.

### Changed

- **A redundant leading zero is no longer a number
  ([spec 0.8 § 5.2](https://github.com/ktav-lang/spec/blob/main/versions/0.8/spec.md)).**
  A base-10 digit run whose first digit is `0` with at least one
  further digit — `01234`, `-045`, `00`, `0_7`, and a float's
  integer part in `01.5` and `05e3` — is now a String carrying the
  digits as written. `zip: 01234` was `Integer(1234)`, which
  destroyed a postcode, a phone number or a zero-padded id without
  saying so; it is now `"01234"`. The rule holds at every entry point —
  `parse`, `parse_strict`, `from_str`, the thin event parser and
  the C ABI — and it is confined to base 10: `0`, `0.5`, `0x1A`,
  `0o755`, `0b1010`, `1_000_000` and `+7` still infer numbers
  exactly as before. Canonical output does not move either, because the
  `::` marker keys off § 3.6's grammar, which 0.8 left untouched: the
  String `"01234"` still renders as `zip:: 01234`. Two consequences
  for callers: code that relied on the old coercion must parse the
  String itself, and `parse_strict` no longer reports
  `LossyScalar` for these forms, since nothing is lost any more.

>>>>> lang=ru
## [0.8.0] — 2026-09-23

Выходят две вещи вместе. Десятичное число с избыточным ведущим нулём
больше не выводится как число — `zip: 01234` разбирается в String
`"01234"`, а не в `Integer(1234)`, — это меняет смысл документа и
именно поэтому выпуск минорный, а не патч. Вместе с этим фича `cabi`
превращает шесть приватных копий C ABI-шима из шести языковых биндингов
в один макровызов к этому crate; эта половина чисто аддитивна и по
умолчанию выключена, так что обычная сборка по-прежнему не тянет
`serde_json`.

### Changed

- **Избыточный ведущий ноль больше не число
  ([spec 0.8 § 5.2](https://github.com/ktav-lang/spec/blob/main/versions/0.8/spec.ru.md)).**
  Ряд цифр по основанию 10, чья первая цифра `0`, при этом следует
  хотя бы ещё одна цифра — `01234`, `-045`, `00`, `0_7`, а также
  целая часть float в `01.5` и `05e3`, — теперь String, несущая
  цифры как написано. `zip: 01234` был `Integer(1234)`, что молча
  уничтожало почтовый индекс, номер телефона или дополненный нулями
  идентификатор; теперь это `"01234"`. Правило действует в каждой точке входа —
  `parse`, `parse_strict`, `from_str`, тонкий event-парсер и
  C ABI — и ограничено основанием 10: `0`, `0.5`, `0x1A`,
  `0o755`, `0b1010`, `1_000_000` и `+7` по-прежнему выводятся в
  числа в точности как раньше. Каноническая запись тоже не меняется,
  потому что маркер `::` привязан к грамматике § 3.6, которую 0.8 не
  тронула: String `"01234"` всё так же выводится как
  `zip:: 01234`. Два следствия для вызывающих: код, полагавшийся на
  старое приведение, должен сам разбирать String, а `parse_strict`
  больше не сообщает `LossyScalar` для этих форм, поскольку терять
  уже нечего.

>>>>> lang=zh
## [0.8.0] —— 2026-09-23

本次发布同时带来两件事。带冗余前导零的十进制数不再被推断为数字 ——
`zip: 01234` 解析为 String `"01234"`,而不是 `Integer(1234)` ——
这改变了文档的含义,也正是本次发布为 minor 而非 patch 的原因。与之同行,
`cabi` feature 把六个语言绑定各自私有的 C ABI 垫片副本收敛为对本 crate
的一次宏调用;这一半是纯增量的、默认关闭,因此默认构建依然不会拉取
`serde_json`。

### 变更

- **冗余前导零不再是数字
  ([spec 0.8 § 5.2](https://github.com/ktav-lang/spec/blob/main/versions/0.8/spec.zh.md))。**
  以 `0` 开头且后面至少还有一位数字的十进制数字串 —— `01234`、
  `-045`、`00`、`0_7`,以及 `01.5` 与 `05e3` 中 float 的整数
  部分 —— 现在是按书写原样携带这些数字的 String。`zip: 01234` 曾是
  `Integer(1234)`,这会不声不响地毁掉邮政编码、电话号码或补零的
  标识符;现在它是 `"01234"`。该规则在每一个入口都成立 ——
  `parse`、`parse_strict`、`from_str`、thin event parser 与
  C ABI —— 并且仅限于十进制:`0`、`0.5`、`0x1A`、`0o755`、
  `0b1010`、`1_000_000` 与 `+7` 仍与此前完全一致地推断为数字。
  规范化输出同样不变,因为 `::` 标记绑定到 § 3.6 的语法,而 0.8 未
  触动它:String `"01234"` 依旧写作 `zip:: 01234`。对调用方有两点
  影响:依赖旧的强制转换的代码须自行解析该 String;而
  `parse_strict` 不再为这些形态报告 `LossyScalar`,因为已经没有
  任何信息会丢失。

