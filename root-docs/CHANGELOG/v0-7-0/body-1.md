>>>>> lang=en
## [0.7.0] — 2026-09-10

Implements [Ktav 0.7.0](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.md),
released the same day. `spec-version` metadata moves to `0.7.0` and the
pinned `spec` submodule moves to the 0.7.0 release commit, so the
conformance corpus this crate is tested against is the released one.

### Breaking

- **`( … )` stripped multi-line strings now strip trailing whitespace
  from every content line** (§ 5.6), matching what the form already did
  to leading whitespace. Previously trailing whitespace survived
  byte-for-byte, which meant an editor's "trim on save" could silently
  change string content. `(( … ))` is unaffected and stays verbatim on
  both edges — use it when trailing whitespace is significant.
- **A recognised escape sequence forces String classification**
  (§ 3.7 / § 5.2). A body written as `\u0031` decodes to `1` but stays a
  String — the escape is evidence of intent, so the decoded text is no
  longer re-classified as a number or keyword.
- **Whitespace is the frozen 25-code-point § 3.3 set everywhere**, with
  no delegation to a host-language Unicode predicate. Key-segment
  trimming widens from ASCII-only to that same set (§ 4), so two keys
  differing only by a non-ASCII whitespace character at a trimmed edge
  now collide as one key.
- **Integers outside the i64 domain are String, including through
  `ser::to_value`.** The parser's Integer domain has always been i64
  (§ 5, § 5.2 rule 13); the serde bridge used to produce a wider
  `Value::Integer` that changed variant and canonical bytes on the very
  next round-trip. `to_value` now yields `Value::String` for a `u64`,
  `i128` or `u128` outside i64 — the same `Value` parsing that literal
  would give. In-range values are unchanged.

>>>>> lang=ru
## [0.7.0] — 2026-09-10

Реализует [Ktav 0.7.0](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.ru.md),
выпущенный в тот же день. Метаданные `spec-version` переходят на `0.7.0`,
а закреплённый submodule `spec` — на релизный коммит 0.7.0, так что
conformance-корпус, против которого проверяется crate, — именно выпущенный.

### Breaking

- **Многострочные строки формы `( … )` теперь срезают завершающие
  пробелы в каждой содержательной строке** (§ 5.6) — так же, как форма
  уже поступала с ведущими. Раньше завершающие пробелы сохранялись
  байт в байт, из-за чего «обрезка пробелов при сохранении» в редакторе
  могла молча изменить содержимое строки. Форма `(( … ))` не затронута и
  остаётся дословной с обеих сторон — используйте её, когда завершающие
  пробелы значимы.
- **Распознанная escape-последовательность принудительно даёт String**
  (§ 3.7 / § 5.2). Тело, записанное как `\u0031`, декодируется в `1`, но
  остаётся String: escape — свидетельство намерения, поэтому
  декодированный текст больше не переклассифицируется в число или
  ключевое слово.
- **Пробельные символы — это замороженный набор из 25 кодовых точек
  § 3.3 везде**, без делегирования Unicode-предикату языка-хозяина.
  Обрезка сегментов ключа расширяется с ASCII-only до того же набора
  (§ 4), поэтому два ключа, различающиеся только не-ASCII пробельным
  символом на обрезаемом краю, теперь совпадают как один ключ.
- **Целые вне домена i64 — это String, в том числе через
  `ser::to_value`.** Integer-домен парсера всегда был i64 (§ 5,
  § 5.2 правило 13); serde-мост же создавал более широкий
  `Value::Integer`, который менял и вариант, и canonical-байты на
  ближайшем же roundtrip. Теперь `to_value` для `u64`, `i128` или
  `u128` вне i64 отдаёт `Value::String` — ровно тот `Value`, который
  дал бы разбор этого литерала. Значения внутри диапазона не изменились.

>>>>> lang=zh
## [0.7.0] —— 2026-09-10

实现同日发布的
[Ktav 0.7.0](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.zh.md)。
`spec-version` 元数据升至 `0.7.0`,固定的 `spec` submodule 也移至 0.7.0
发布提交,因此本 crate 所对照的 conformance 语料库正是已发布的那一份。

### 破坏性变更

- **`( … )` 去缩进多行字符串现在会剥除每个内容行的行尾空白**(§ 5.6),
  与它早已对行首空白所做的一致。此前行尾空白逐字节保留,这意味着编辑器
  的「保存时去除行尾空白」可能悄然改变字符串内容。`(( … ))` 不受影响,
  两端仍然完全逐字——当行尾空白有意义时请使用该形式。
- **被识别的 escape 序列强制归类为 String**(§ 3.7 / § 5.2)。写作
  `\u0031` 的 body 解码为 `1`,但仍是 String:escape 是意图的证据,因此解码
  后的文本不再被重新归类为数字或关键字。
- **空白是各处统一的、冻结的 § 3.3 25 码点集合**,不再委派给宿主语言的
  Unicode 判定。键段修剪从仅 ASCII 扩展到同一集合(§ 4),因此仅在被修剪
  边缘相差一个非 ASCII 空白字符的两个键,现在会合并为同一个键。
- **i64 域之外的整数是 String,通过 `ser::to_value` 也是如此。** 解析器
  的 Integer 域一直是 i64(§ 5、§ 5.2 规则 13);而 serde 桥接层此前会
  产生更宽的 `Value::Integer`,它在下一次 roundtrip 就会改变变体与
  canonical 字节。现在 `to_value` 对超出 i64 的 `u64`、`i128` 或 `u128`
  返回 `Value::String`——与解析同一字面量所得的 `Value` 完全一致。域内
  数值不变。

