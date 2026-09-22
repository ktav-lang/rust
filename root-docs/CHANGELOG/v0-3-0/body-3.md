>>>>> lang=en
- `render::render` pre-sizes the output `String` with a recursive
  `estimate_size(value)` to skip the doubling reallocations that
  `push_str` chains would otherwise trigger on multi-KiB outputs.
- `EventCursor::peek` / `next` use `unsafe get_unchecked` for
  bounds-elision on the hot path; the parser's well-formed-stream
  invariant guarantees `pos < len()` on every call. Falls back to
  `None` when the invariant is violated, so malformed inputs remain
  safe — the unsafe path is a pure branch elision win.
- `MapAccess::next_key_seed` folds redundant `peek + next` into a
  single `next` (both branches consume the cursor anyway).
- Event `BumpVec` capacity hint raised from `text.len() / 8 + 16`
  to `text.len() / 4 + 64`. The previous hint underestimated the
  ~1-event-per-5-bytes density on synth fixtures and triggered
  8–10 realloc-copy steps inside the bump arena on a 500 KiB doc.

### Experiment (reverted)

- A streaming-deserializer refactor (parse on demand, no whole-doc
  `Vec<Event>`) was implemented and tested — all 404 tests passed,
  but parse_to_struct regressed 15–60 % vs. the existing cursor on
  this hardware. Cause: the cursor walks a contiguous slice with
  one monotonic branch the predictor nails 100 %, while streaming
  interleaves parser state-machine work with deserializer work and
  blows the predictor. The streaming code was removed; the
  `EventSink<'a>` trait introduced for the experiment survives in
  `event.rs` as harmless generic infrastructure (zero cost when
  used only with `BumpVec`).

>>>>> lang=ru
- `render::render` преднастраивает размер выходного `String` через
  рекурсивный `estimate_size(value)`, чтобы обойти удваивающие
  реаллокации, которые цепочки `push_str` иначе вызывали бы на выводе
  в несколько KiB.
- `EventCursor::peek` / `next` используют `unsafe get_unchecked` для
  устранения проверки границ на горячем пути; инвариант
  well-formed-потока парсера гарантирует `pos < len()` на каждом
  вызове. При нарушении инварианта возвращается `None`, поэтому
  некорректные входы остаются безопасными — unsafe-путь здесь чистый
  выигрыш на устранении ветвления.
- `MapAccess::next_key_seed` сворачивает избыточные `peek + next` в
  один `next` (обе ветки всё равно потребляют курсор).
- Подсказка ёмкости event-`BumpVec` поднята с `text.len() / 8 + 16`
  до `text.len() / 4 + 64`. Прежняя подсказка недооценивала плотность
  ~1 событие на 5 байт на synth-фикстурах и вызывала 8–10 шагов
  realloc-copy внутри bump-арены на документе в 500 KiB.

### Эксперимент (откачен)

- Рефакторинг стримингового десериализатора (разбор по требованию, без
  `Vec<Event>` на весь документ) был реализован и протестирован — все
  404 теста прошли, но parse_to_struct регрессировал на 15–60 % против
  существующего курсора на этом железе. Причина: курсор идёт по
  непрерывному срезу с одной монотонной ветвью, которую предсказатель
  угадывает на 100 %, тогда как стриминг перемежает работу
  конечного автомата парсера с работой десериализатора и сбивает
  предсказатель. Стриминговый код удалён; трейт `EventSink<'a>`,
  введённый для эксперимента, остался в `event.rs` как безвредная
  обобщённая инфраструктура (нулевая стоимость при использовании
  только с `BumpVec`).

>>>>> lang=zh
- `render::render` 通过递归的 `estimate_size(value)` 预设输出 `String`
  的容量，从而跳过 `push_str` 链在数 KiB 输出上本会触发的倍增式
  重分配。
- `EventCursor::peek` / `next` 使用 `unsafe get_unchecked` 以消除热
  路径上的边界检查；解析器的良构流不变式保证每次调用时
  `pos < len()`。不变式被破坏时回退为 `None`，因此畸形输入依然安全
  —— 这里的 unsafe 路径纯粹是省去一次分支判断的收益。
- `MapAccess::next_key_seed` 将冗余的 `peek + next` 合并为单次 `next`
  （两个分支反正都会消费游标）。
- 事件 `BumpVec` 的容量提示从 `text.len() / 8 + 16` 提高到
  `text.len() / 4 + 64`。旧提示低估了 synth fixture 上约每 5 字节
  1 个事件的密度，在 500 KiB 文档上会在 bump arena 内触发 8–10 次
  realloc-copy。

### 实验（已回退）

- 一次流式反序列化器重构（按需解析，不保留整份文档的 `Vec<Event>`）
  已实现并测试 —— 全部 404 个测试通过，但在本机硬件上
  parse_to_struct 相对既有游标回退了 15–60 %。原因：游标沿连续切片
  行进，只有一条单调分支，预测器命中率 100 %；而流式方案把解析器状态
  机的工作与反序列化器的工作交错，打乱了预测器。流式代码已删除；为该
  实验引入的 `EventSink<'a>` trait 作为无害的泛型基础设施保留在
  `event.rs` 中（仅与 `BumpVec` 搭配使用时零开销）。

