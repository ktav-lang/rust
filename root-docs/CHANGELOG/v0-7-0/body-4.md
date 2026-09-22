>>>>> lang=en
- **`ser::to_value` now stores the payload the parser would store.**
  Float payloads carried a text-literal `.0` mantissa (`1.0e100` where
  the parser stores `1e100`), and `f32` went through ryu's f32
  thresholds rather than the f64 ones the parser uses (`0.000001` vs
  `1e-6`). A document built with `to_value` therefore stopped comparing
  equal to itself after `render`/`emit_canonical` + `parse`. Numeric
  bits and canonical output were always correct; only the stored
  representation disagreed.
- **Typed map keys behave identically through both read APIs and both
  write APIs.** `from_str` and `de::from_value` disagreed about numeric,
  `bool`, unit-enum and newtype keys; `to_string` and `ser::to_value`
  produced different key *names* for the same value (`1` vs `1.0`) and
  accepted different key types. Both sides now share one policy. Keys
  wrapped in `Some(...)` were accepted by the writers but rejected by
  both readers — they round-trip now, and a key literally named `null`
  stays the string `"null"` rather than becoming `None`.
- **Inline-compound scanning**, across many edge cases surfaced by the
  0.7 conformance corpus and the review series: quote tracking keyed to
  the correct scope, quotes in a value no longer shielding a structural
  closer, mid-scalar openers treated as literal bytes, raw scalars
  terminating at any unescaped closer, per-scope raw closers and scope
  restore, comma key-context derived from the active scope, EOF after a
  whitespace skip, and a bracket in an inline-key position reported as
  `InvalidKey` rather than a phantom compound error.
- **§ 5.6 dedent measures the common prefix in § 3.3 code points, not
  bytes**, through a single shared prefix scan subtracted per non-blank
  line.
- **§ 5.3.2 dotted-key re-entry** in the thin event parser, plus
  compound child-path registration and merge frames for bare compound
  openers.
- **`i64::MIN` is represented exactly** from a negative prefixed integer
  literal.
- Empty root tuple-variant names are rejected by the serde text
  serializer.

### Performance

No timing figures are claimed for this release — the work below was
driven by source-level analysis and allocation counters, not benchmarks.

>>>>> lang=ru
- **`ser::to_value` теперь хранит тот payload, который сохранил бы
  парсер.** Float-payload нёс текстовую мантиссу с `.0` (`1.0e100` там,
  где парсер хранит `1e100`), а `f32` проходил через f32-пороги ryu
  вместо f64-порогов, которыми пользуется парсер (`0.000001` против
  `1e-6`). Из-за этого документ, построенный через `to_value`,
  переставал сравниваться равным самому себе после
  `render`/`emit_canonical` + `parse`. Числовые биты и canonical-вывод
  всегда были верны; расходилось только хранимое представление.
- **Типизированные ключи maps ведут себя одинаково через оба read API и
  оба write API.** `from_str` и `de::from_value` расходились на
  числовых, `bool`, unit-enum и newtype ключах; `to_string` и
  `ser::to_value` давали разные *имена* ключа для одного значения
  (`1` против `1.0`) и принимали разные типы ключей. Теперь обе стороны
  используют одну политику. Ключи, обёрнутые в `Some(...)`, писались
  обоими writer'ами, но отвергались обоими reader'ами — теперь они
  проходят roundtrip, а ключ с буквальным именем `null` остаётся
  строкой `"null"`, а не становится `None`.
- **Сканирование inline-компаундов** во множестве краевых случаев,
  вскрытых corpus 0.7 и серией ревью: отслеживание кавычек привязано к
  правильному scope, кавычки в значении больше не заслоняют структурный
  закрыватель, открыватели в середине скаляра трактуются как обычные
  байты, сырые скаляры завершаются на любом неэкранированном
  закрывателе, пер-scope сырые закрыватели и восстановление scope,
  key-контекст запятой выводится из активного scope, EOF после
  пропуска пробелов, а скобка в позиции inline-ключа даёт `InvalidKey`,
  а не фантомную ошибку компаунда.
- **Дедент § 5.6 измеряет общий префикс в кодовых точках § 3.3, а не в
  байтах**, через единственный общий скан префикса, вычитаемый из
  каждой непустой строки.
- **Повторный вход в точечный ключ (§ 5.3.2)** в thin event-парсере,
  плюс регистрация дочерних путей компаунда и merge-фреймы для голых
  открывателей компаундов.
- **`i64::MIN` представляется точно** из отрицательного целочисленного
  литерала с префиксом.
- Пустые имена корневых tuple-variant отвергаются текстовым
  serde-сериализатором.

### Производительность

Никаких замеров времени для этого релиза не заявляется — работа ниже
основана на анализе исходников и счётчиках аллокаций, не на benchmark'ах.

>>>>> lang=zh
- **`ser::to_value` 现在存放解析器会存放的 payload。** Float payload 曾带
  有文本字面量式的 `.0` 尾数(解析器存 `1e100` 处它存 `1.0e100`),而
  `f32` 走的是 ryu 的 f32 阈值而非解析器所用的 f64 阈值(`0.000001` 对
  `1e-6`)。因此用 `to_value` 构建的文档在 `render`/`emit_canonical` +
  `parse` 之后不再与自身相等。数值位与 canonical 输出一直是正确的,分歧
  仅在于存放的表示。
- **类型化的 map 键在两个读 API 与两个写 API 上行为一致。** `from_str`
  与 `de::from_value` 在数字、`bool`、unit enum 与 newtype 键上互相分歧;
  `to_string` 与 `ser::to_value` 对同一个值给出不同的键*名*(`1` 对
  `1.0`)且接受不同的键类型。现在两侧共用一套策略。被 `Some(...)` 包裹的
  键此前可被 writer 接受却被两个 reader 拒绝——现在可以完整 roundtrip,
  而字面名为 `null` 的键保持为字符串 `"null"`,不会变成 `None`。
- **inline 复合值扫描** 的大量边界情形,由 0.7 语料库与系列评审揭示:引号
  跟踪绑定到正确的 scope、值中的引号不再遮蔽结构性闭合符、标量中部的开启
  符视作普通字节、裸标量在任何未转义闭合符处终止、逐 scope 的裸闭合符与
  scope 恢复、逗号的键上下文取自当前 scope、空白跳过后的 EOF,以及 inline
  键位置上的方括号报告为 `InvalidKey` 而非幻影复合值错误。
- **§ 5.6 去缩进以 § 3.3 码点而非字节度量公共前缀**,通过一次共享的前缀
  扫描,并从每个非空行中扣除。
- thin event 解析器中的 **§ 5.3.2 点分键重入**,以及复合值子路径注册与裸
  复合开启符的 merge frame。
- **`i64::MIN` 由带前缀的负整数字面量精确表示**。
- serde 文本序列化器拒绝空的根 tuple-variant 名称。

### 性能

本次发布不声称任何计时数据——以下工作基于源码级分析与分配计数器,而非
benchmark。

