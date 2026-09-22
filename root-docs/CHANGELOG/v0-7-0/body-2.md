>>>>> lang=en
### Added

- **Leading BOM handling** (§ 3.1): exactly one leading U+FEFF is
  skipped if it is the document's first code point; the canonical writer
  never emits one. A U+FEFF anywhere else is ordinary content.
- **Quoted keys** (§ 5.3.3): quote-aware key and compound scanners, the
  0.7 escape table with quoted-segment validation and decoding, and the
  § 5.9.10 canonical key-form selection and re-escape recipes in every
  writer. The thin event path strips quoted-key delimiters.
- **`\uXXXX` escapes** decoded in values and in keys (§ 3.7.1), with
  `BadEscapeSequence` diagnostics for lone and mismatched surrogates.
- **Root Array documents** across the serde root serializer and both
  parsers, with the § 5.9.6 / § 5.9.12 Array-root first-item safeguards
  in every writer.
- **Writer-side representable-value rejection** (§ 5.9.0) with reason
  codes on all three writer surfaces.
- **`InvalidUtf8`** as a distinct error category carrying a byte offset,
  on the byte-boundary surface (§ 6.15); **`UnterminatedQuotedKey`**
  (§ 6.16) for an unclosed quoted key.
- **`i128` / `u128` support in both deserializers.** They serialized
  correctly before but could not be read back: serde's default methods
  reject those types outright unless overridden. Both the owned and the
  thin path now parse the digits exactly, with no 64-bit or `f64` hop.
- Conformance runner extended to the 0.7 corpus, including its
  `unrepresentable/` and `parseable-unrepresentable/` categories.

>>>>> lang=ru
### Добавлено

- **Обработка ведущего BOM** (§ 3.1): ровно один ведущий U+FEFF
  пропускается, если это первая кодовая точка документа; канонический
  writer его никогда не выводит. U+FEFF в любом другом месте — обычное
  содержимое.
- **Ключи в кавычках** (§ 5.3.3): quote-aware сканеры ключей и
  компаундов, таблица escape 0.7 с валидацией и декодированием
  quoted-сегментов, а также выбор канонической формы ключа и рецепты
  переэкранирования из § 5.9.10 во всех writer'ах. Thin event-путь
  снимает разделители quoted-ключа.
- **Escape-последовательности `\uXXXX`**, декодируемые в значениях и в
  ключах (§ 3.7.1), с диагностикой `BadEscapeSequence` для одиночных и
  несогласованных суррогатов.
- **Документы с корневым Array** во всех трёх точках: serde
  root-сериализатор и оба парсера, вместе с предохранителями первого
  элемента Array-корня из § 5.9.6 / § 5.9.12 в каждом writer'е.
- **Отклонение непредставимых значений на стороне writer** (§ 5.9.0) с
  reason-кодами на всех трёх writer-поверхностях.
- **`InvalidUtf8`** как отдельная категория ошибки с байтовым смещением
  на байтовой поверхности (§ 6.15); **`UnterminatedQuotedKey`** (§ 6.16)
  для незакрытого ключа в кавычках.
- **Поддержка `i128` / `u128` в обоих десериализаторах.** Раньше они
  корректно сериализовались, но не читались обратно: дефолтные методы
  serde отвергают эти типы, пока их не переопределить. Теперь и owned-,
  и thin-путь разбирают цифры точно, без промежуточного 64-битного или
  `f64` прыжка.
- Conformance-runner расширен на корпус 0.7, включая его категории
  `unrepresentable/` и `parseable-unrepresentable/`.

>>>>> lang=zh
### 新增

- **前导 BOM 处理**(§ 3.1):若 U+FEFF 是文档的首个码点,则恰好跳过一个;
  canonical writer 永不输出 BOM。位于其他位置的 U+FEFF 是普通内容。
- **带引号的键**(§ 5.3.3):quote-aware 的键与复合值扫描器、带 quoted
  段验证与解码的 0.7 escape 表,以及所有 writer 中 § 5.9.10 的
  canonical 键形式选择与重新转义规则。thin event 路径会剥除 quoted 键的
  定界符。
- **`\uXXXX` escape** 在值与键中均可解码(§ 3.7.1),对孤立代理与不匹配
  代理给出 `BadEscapeSequence` 诊断。
- **Array 作为根的文档**:serde root 序列化器与两个解析器均支持,并在每个
  writer 中带有 § 5.9.6 / § 5.9.12 的 Array 根首元素保护。
- **writer 侧对不可表示值的拒绝**(§ 5.9.0),三个 writer 面上均带
  reason code。
- **`InvalidUtf8`** 成为携带字节偏移的独立错误类别,位于字节边界面
  (§ 6.15);**`UnterminatedQuotedKey`**(§ 6.16)用于未闭合的带引号键。
- **两个反序列化器都支持 `i128` / `u128`。** 它们此前能够正确序列化却读不
  回来:除非覆盖,serde 的默认方法会直接拒绝这两个类型。现在 owned 与
  thin 两条路径都精确解析数字,不经过 64 位或 `f64` 中转。
- conformance runner 扩展到 0.7 语料库,包含其 `unrepresentable/` 与
  `parseable-unrepresentable/` 类别。

