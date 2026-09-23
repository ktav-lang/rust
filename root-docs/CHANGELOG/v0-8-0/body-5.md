>>>>> lang=en
### Performance

Measured with callgrind instruction counts, not wall-clock benchmarks —
criterion's confidence intervals were noise-dominated on this host even
under CPU-affinity restriction, while callgrind counts are deterministic
per input regardless of what else the machine is doing
(`examples/callgrind_writers.rs`, run under valgrind in WSL). Per call,
before → after: `emit_canonical` −40%, `render` −36%,
`to_string_force_strings` −57%, `format_str` −25%. Each figure holds
flat across three orders of magnitude of document size — the signature
of removed per-node work, not a shifted constant — and `parse` itself
is untouched (agrees to 0.002%).

- Every writer opened by walking the whole `Value` through
  `check_representable` before emitting a byte, even though each writer
  already discards its buffer on rejection by construction — that
  guarantee comes from assembling into a local buffer, not from the
  pre-pass. The check now happens at the point of emission, and the
  cold path re-walks only to name the offending key once a rejection
  has already happened.
- The formatter paid the most: `emit_formatted` materialised an entire
  throw-away `Value` tree — a fresh map per object, every key and
  scalar cloned — purely to hand `check_representable` an argument of
  the right type. That tree is now built only after a failure.
- `to_string_force_strings` no longer deep-clones the document; a
  `force_strings` flag reaches the leaf emitters directly.
- `benches/emit.rs` adds the criterion coverage `emit_canonical`,
  `render` and `to_string_force_strings` never had.

>>>>> lang=ru
### Производительность

Измерено через счётчики инструкций callgrind, а не через замеры
времени на стенке — доверительные интервалы criterion на этой машине
тонули в шуме даже при ограничении по affinity, тогда как счётчики
callgrind детерминированы для данного входа независимо от того, чем
ещё занята машина (`examples/callgrind_writers.rs`, прогон под valgrind
в WSL). На вызов, до → после: `emit_canonical` −40%, `render` −36%,
`to_string_force_strings` −57%, `format_str` −25%. Каждая цифра
держится ровно на трёх порядках величины размера документа — это
признак устранённой работы на узел, а не сдвинутой константы — а сам
`parse` не тронут (совпадение до 0.002%).

- Каждый writer открывался обходом всего `Value` через
  `check_representable` ещё до вывода первого байта, хотя каждый writer
  и так отбрасывает свой буфер при отказе по построению — эта гарантия
  идёт от сборки в локальный буфер, а не от пре-паса. Теперь условие
  проверяется в момент вывода, а холодный путь повторно обходит дерево
  только чтобы назвать проблемный ключ, когда отказ уже произошёл.
- Больше всего платил форматтер: `emit_formatted` материализовал целое
  одноразовое дерево `Value` — свежую карту на каждый объект, копию
  каждого ключа и скаляра — только чтобы дать `check_representable`
  аргумент нужного типа. Теперь это дерево строится только после отказа.
- `to_string_force_strings` больше не клонирует документ целиком; флаг
  `force_strings` доходит до листовых эмиттеров напрямую.
- `benches/emit.rs` добавляет criterion-покрытие, которого у
  `emit_canonical`, `render` и `to_string_force_strings` не было вовсе.

>>>>> lang=zh
### 性能

通过 callgrind 指令计数测量,而非墙钟时间基准测试 —— 在这台机器上,
即便限制了 CPU 亲和性,criterion 的置信区间依然被噪声淹没,而
callgrind 的计数对给定输入是确定性的,无论机器上还有什么其他负载
(`examples/callgrind_writers.rs`,在 WSL 下用 valgrind 运行)。每次
调用,变更前 → 后:`emit_canonical` −40%,`render` −36%,
`to_string_force_strings` −57%,`format_str` −25%。每个数字在三个
数量级的文档大小上保持不变 —— 这是去除了按节点开销的特征,而非
偏移了某个常数 —— `parse` 本身未受影响(误差在 0.002% 以内)。

- 此前每个 writer 在写出第一个字节之前都会用 `check_representable`
  遍历整棵 `Value`,尽管每个 writer 本就通过先写入本地缓冲区、失败时
  丢弃的方式保证了这一点,而非依赖这次预扫描。现在该条件在写出的那一刻
  才检测,冷路径只在已经发生拒绝之后才重新遍历以指出问题键。
- 代价最大的是格式化器:`emit_formatted` 会整体物化一棵一次性的
  `Value` 树 —— 每个对象一份新映射,每个键和标量都被克隆 —— 仅仅是为
  了给 `check_representable` 一个类型匹配的参数。现在这棵树只在失败
  之后才会构建。
- `to_string_force_strings` 不再深拷贝整个文档;`force_strings`
  标志直接传递到叶子发射器。
- `benches/emit.rs` 为 `emit_canonical`、`render` 与
  `to_string_force_strings` 补上了此前完全没有的 criterion 覆盖。

