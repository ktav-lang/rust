>>>>> lang=en
## Scope

This is the reference Rust crate that every other binding wraps. A
real issue here usually affects `ktav-lang/python`, `ktav-lang/js`,
`ktav-lang/golang` at once — treat it accordingly.

Issues that count as security problems for this crate:

- Panics on crafted input reaching `parse` / `from_str` / `to_string`
  / `render`. The crate targets `panic = "abort"` builds in downstream
  bindings, so a panic terminates the consumer process.
- Runaway memory or CPU (quadratic behaviour, unbounded allocation,
  infinite loops) on crafted input.
- `unsafe` correctness: any soundness hole in `unsafe` blocks —
  out-of-bounds access, UB, aliasing violations — even if no obvious
  exploit path exists.
- Any behaviour that allows crafted Ktav input to produce a `Value`
  outside the documented grammar (lossy round-trips that drop or
  forge data).

Issues that are **not** security problems here — please use regular
issues for these:

- Performance regressions without a DoS-shape characteristic.
- Parser error messages being unclear or imprecise.
- Problems in the Ktav format itself — those belong in
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec).
>>>>> lang=ru
## Область

Это референсный Rust-крейт, который оборачивают все остальные биндинги.
Реальная проблема здесь обычно затрагивает `ktav-lang/python`,
`ktav-lang/js`, `ktav-lang/golang` одновременно — относитесь соответственно.

Что считается проблемой безопасности для этого крейта:

- Паники на сформированном входе в `parse` / `from_str` / `to_string`
  / `render`. Крейт используется в downstream-биндингах со сборкой
  `panic = "abort"`, так что паника валит consumer-процесс.
- Неконтролируемое потребление памяти или CPU (квадратичное поведение,
  неограниченные аллокации, бесконечные циклы) на сформированном входе.
- Корректность `unsafe`: любая дыра в safety `unsafe`-блоков — выход
  за границы, UB, нарушения aliasing — даже если очевидного пути
  эксплуатации не видно.
- Любое поведение, при котором сформированный Ktav-вход порождает
  `Value` вне документированной грамматики (lossy round-trip,
  теряющий или подделывающий данные).

Что **не** считается проблемой безопасности здесь — пожалуйста,
используйте обычные issue:

- Регрессии производительности без DoS-характеристик.
- Непонятные или неточные сообщения парсера об ошибках.
- Проблемы в самом формате Ktav — им место в
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec).
>>>>> lang=zh
## 范围

这是所有其他绑定都封装的参考 Rust crate。这里的真实问题通常会同时
影响 `ktav-lang/python`、`ktav-lang/js`、`ktav-lang/golang` —— 请据此对待。

以下问题会按本 crate 的安全问题处理:

- `parse` / `from_str` / `to_string` / `render` 在构造输入下 panic。
  downstream 绑定采用 `panic = "abort"` 构建，panic 会终止 consumer 进程。
- 构造输入导致的失控内存或 CPU 消耗（二次复杂度、无界分配、死循环）。
- `unsafe` 正确性：`unsafe` 块中的任何 soundness 漏洞 —— 越界访问、
  UB、aliasing 违例 —— 即使没有明显的利用路径。
- 任何允许构造的 Ktav 输入产生文档化语法之外 `Value` 的行为（丢数据
  或伪造数据的 lossy round-trip）。

以下**不**算本 crate 的安全问题 —— 请走普通 issue:

- 无 DoS 特征的性能回归。
- 解析器错误信息不清晰或不精确。
- Ktav 格式本身的问题 —— 这类问题属于
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec)。
