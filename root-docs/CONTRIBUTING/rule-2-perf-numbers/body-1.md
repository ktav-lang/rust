>>>>> lang=en
### 2. Performance-sensitive changes include before/after numbers

If a PR touches any of:

- `src/parser/` / `src/thin/parser.rs` — parsing hot path
- `src/ser/text_serializer.rs` / `src/render/` — serialization hot path
- `src/thin/deserializer.rs` / `src/de/` — deserialization hot path
- `src/value/` — the dynamic value type

… include a block in the PR description with **before/after** criterion
numbers for the affected benches:

```
parse_to_struct/100_upstreams_typed
  before: 275 µs
  after:  198 µs
  change: -28%
```

The goal isn't to be fastest at any cost — it's to make changes
*accountable*. A 5 % regression is fine if it buys clarity or
correctness, but it should be visible and justified.

Use the convenience script:

```
./bench.sh                # quick run (warmup 1s, measurement 2s, 20 samples)
./bench.sh full           # criterion defaults — longer, more accurate
./bench.sh parse          # filter: only parse benches
./bench.sh render         # filter: only render benches
./bench.sh "parse|render" # any criterion regex
```

Criterion stores the last run's numbers in `target/criterion/` and
automatically diffs against them on the next run, so you'll see
`change: +/- X %` in the output.

>>>>> lang=ru
### 2. Чувствительные к производительности изменения сопровождаются числами before/after

Если PR трогает что-либо из:

- `src/parser/` / `src/thin/parser.rs` — горячий путь парсинга
- `src/ser/text_serializer.rs` / `src/render/` — горячий путь
  сериализации
- `src/thin/deserializer.rs` / `src/de/` — горячий путь
  десериализации
- `src/value/` — динамический тип значения

… в описании PR должен быть блок с **before/after** числами criterion
по затронутым бенчам:

```
parse_to_struct/100_upstreams_typed
  before: 275 µs
  after:  198 µs
  change: -28%
```

Цель — не быть самым быстрым любой ценой, а сделать изменения
*подотчётными*. Регрессия 5 % допустима, если она покупает ясность
или корректность, но должна быть видна и обоснована.

Используйте удобный скрипт:

```
./bench.sh                # quick run (warmup 1s, measurement 2s, 20 samples)
./bench.sh full           # criterion defaults — longer, more accurate
./bench.sh parse          # filter: only parse benches
./bench.sh render         # filter: only render benches
./bench.sh "parse|render" # any criterion regex
```

Criterion хранит числа прошлого прогона в `target/criterion/` и
автоматически сравнивает их при следующем прогоне, так что в выводе
будет `change: +/- X %`.

>>>>> lang=zh
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

