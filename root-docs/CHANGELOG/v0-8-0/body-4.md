>>>>> lang=en
- **A load test for the whole surface** — a fixture cdylib whose
  body is one `declare_cabi!()` invocation, built and dlopened by the
  suite; all ten exported symbols resolve through `libloading`, and
  `ktav_abi_version`, `ktav_loads`, `ktav_format` and
  `ktav_canonical_from_source` are called for real.
  The wire round-trip runs over the conformance corpus: 223
  `valid/` fixtures keep their `Value` across `loads` ->
  `dumps`, and 223 canonical byte oracles match `emit_canonical`
  through the wire.

- **`ErrorEnvelope.message` — the error's `Display` rendering,
  verbatim.** The envelope now has ten fields; `message` is appended
  last, so the nine that shipped in 0.7.1 keep their positions, and it
  is the only field that is never `null`. It exists because the other
  nine are structured data and none of them is prose: a binding that
  wanted an exception message had to assemble one itself, and five of
  them did — Go, Java, PHP, C# and JavaScript each invented a different
  layout, and none matched what the crate and the PyO3 binding already
  printed for the same input. Hosts surface this field as-is.

### Fixed

- **`ErrorEnvelope.body` now carries the payload of
  `Error::Message` and `Error::Syntax`.** It was always `null`
  for these two classes. That broke under the C ABI's uniformity rule:
  non-`ktav` failures travel as `Message`, and a caller would have
  received `{"error":"Message"}` plus eight nulls instead of the
  diagnostic. No shipped consumer can regress — the envelope has never
  shipped in any binding.

>>>>> lang=ru
- **Загрузочный тест всей поверхности** — cdylib-фикстура, чьё тело —
  один вызов `declare_cabi!()`; тест собирает её, открывает через
  dlopen, разрешает все десять экспортированных символов через
  `libloading` и по-настоящему вызывает `ktav_abi_version`,
  `ktav_loads`, `ktav_format` и `ktav_canonical_from_source`. Wire-проверка идёт по
  конформанс-корпусу: 223 фикстуры из `valid/` сохраняют `Value`
  через `loads` -> `dumps`, и 223 канонических байтовых оракула
  совпадают с `emit_canonical` через wire.

- **`ErrorEnvelope.message` — дословное `Display`-представление
  ошибки.** В конверте теперь десять полей; `message` дописано в
  конец, поэтому девять полей из 0.7.1 сохранили свои позиции, и это
  единственное поле, которое никогда не `null`. Оно понадобилось
  потому, что остальные девять — структурированные данные, и ни одно из
  них не является текстом: биндингу, которому нужен текст исключения,
  приходилось собирать его самому — и пятеро так и сделали. Go, Java,
  PHP, C# и JavaScript изобрели пять разных раскладок, и ни одна не
  совпадала с тем, что для того же ввода уже печатали сам crate и
  биндинг на PyO3. Host показывает это поле как есть.

### Fixed

- **`ErrorEnvelope.body` теперь несёт полезную нагрузку
  `Error::Message` и `Error::Syntax`.** Раньше для этих двух
  классов там всегда был `null`. Это ломалось на правиле
  единообразия C ABI: сбои вне `ktav` путешествуют как `Message`, и
  вместо диагностики вызывающий получил бы `{"error":"Message"}` и
  восемь null. Регресса у выпущенных потребителей нет — конверт ещё не
  выходил ни в одном биндинге.

>>>>> lang=zh
- **整个表面的加载测试** —— 一份 body 仅有一行 `declare_cabi!()`
  调用的 fixture cdylib;测试套件构建它、以 dlopen 打开、经
  `libloading` 解析出全部十个导出符号,并真实调用
  `ktav_abi_version`、`ktav_loads`、`ktav_format` 与
  `ktav_canonical_from_source`。wire 往返
  覆盖一致性语料库:223 个 `valid/` fixture 在 `loads` ->
  `dumps` 之间保持 `Value` 不变,223 个规范字节预言机与经 wire 的
  `emit_canonical` 逐字节一致。

- **`ErrorEnvelope.message` —— 错误的 `Display` 渲染结果,逐字
  携带。** 信封现在有十个字段;`message` 追加在最后,因此 0.7.1 中
  发布的九个字段位置不变,而它是唯一永不为 `null` 的字段。之所以需要
  它,是因为其余九个都是结构化数据,没有一个是文字:想要异常消息的
  绑定只能自己拼装 —— 而且有五个确实这么做了。Go、Java、PHP、C# 与
  JavaScript 各自发明了一种不同的排版,没有一种与 crate 本身和 PyO3
  绑定对同一输入已经打印的内容一致。host 直接原样呈现该字段。

### 修复

- **`ErrorEnvelope.body` 现在携带 `Error::Message` 与
  `Error::Syntax` 的载荷。** 这两个类此前恒为 `null`。在 C ABI 的
  统一性规则下这会坏掉:非 ktav 的失败以 `Message` 形态传递,调用方将
  只会看到 `{"error":"Message"}` 和八个 null,而不是诊断信息。已发布
  的消费方不会回归——错误信封从未随任何绑定发布过。

