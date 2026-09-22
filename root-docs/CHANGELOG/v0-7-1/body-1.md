>>>>> lang=en
## [0.7.1] — 2026-09-16

Implements [Ktav 0.7.1](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.md),
released 2026-09-16. That specification change is
editorial — § 8.5 and a machine-readable corpus manifest — and asks
nothing new of a parser or a writer: every 0.7.0 document parses to the
same Value and every canonical rendering is unchanged byte for
byte. What this release adds is the tooling that verification story leans
on, plus one conformance fix.

`spec-version` metadata moves to `0.7.1` and the pinned `spec`
submodule moves to the 0.7.1 release commit, so the conformance corpus
this crate is tested against is the released one.

### Added

- **`ErrorEnvelope` — one JSON object for every structured error**,
  parse-time and writer-time alike. Nine fields, always all nine, in a
  fixed order: `error`, `reason`, `line`, `line_text`, `span`,
  `path`, `body`, `canonical`, `spec_section`. Absent information is
  an explicit `null`, never an omitted key, so a consumer in any
  language can read every field positionally without negotiating a
  schema first. `path` is an
  array of exact decoded key segments, never a joined string: a key
  literally named `a.b` is one segment and cannot be confused with a
  two-segment path. Build it with
  `ErrorEnvelope::from_error(&err, source)`, render it with
  `to_json()` or `push_json()`. No serde dependency is involved and
  every string is escaped per RFC 8259.

>>>>> lang=ru
## [0.7.1] — 2026-09-16

Реализует [Ktav 0.7.1](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.ru.md),
выпущенный 2026-09-16. Это изменение спецификации редакционное — § 8.5 и машиночитаемый
манифест корпуса — и от парсера или писателя не требует ничего нового:
любой документ 0.7.0 разбирается в тот же Value, и любая каноническая
отрисовка не меняется ни на байт. Этот выпуск добавляет инструментарий, на который
опирается такая проверка, и одно исправление соответствия.

Метаданные `spec-version` переходят на `0.7.1`, а закреплённый
submodule `spec` — на релизный коммит 0.7.1, так что conformance-корпус,
против которого проверяется crate, — именно выпущенный.

### Added

- **`ErrorEnvelope` — один JSON-объект для любой структурированной
  ошибки**, как на разборе, так и на записи. Девять полей, всегда все
  девять, в фиксированном порядке: `error`, `reason`, `line`,
  `line_text`, `span`, `path`, `body`, `canonical`,
  `spec_section`. Отсутствующие сведения — явный `null`, а не
  пропущенный ключ, поэтому потребитель на любом языке читает каждое
  поле позиционно, без предварительного согласования схемы. `path` — массив
  точных декодированных сегментов ключа, а не склеенная строка: ключ,
  буквально названный `a.b`, — это один сегмент, и его нельзя спутать
  с путём из двух. Строится через
  `ErrorEnvelope::from_error(&err, source)`, печатается через
  `to_json()` или `push_json()`. Зависимость от serde не
  задействована, каждая строка экранируется по RFC 8259.

>>>>> lang=zh
## [0.7.1] —— 2026-09-16

实现于 2026-09-16 发布的
[Ktav 0.7.1](https://github.com/ktav-lang/spec/blob/main/versions/0.7/spec.zh.md)。
该规范变更是编辑性的 —— § 8.5 与一份机器可读的语料库清单 —— 对解析器
或写入器并无新要求:每一份 0.7.0 文档解析所得的 Value 不变,每一次
规范化输出也逐字节不变。本次发布新增的是那套验证所依赖的
工具,以及一处一致性修复。

`spec-version` 元数据升至 `0.7.1`,固定的 `spec` submodule 也移至
0.7.1 发布提交,因此本 crate 所对照的 conformance 语料库正是已发布的
那一份。

### 新增

- **`ErrorEnvelope` —— 任何结构化错误都对应一个 JSON 对象**,解析期与
  写入期一视同仁。九个字段,永远是全部九个,顺序固定:`error`、
  `reason`、`line`、`line_text`、`span`、`path`、`body`、
  `canonical`、`spec_section`。缺失的信息是显式的 `null`,而不是省略
  键,因此任何语言的使用方都能按位置读取每个字段,无需事先协商模式。`path` 是精确解码后的键段数组,而非拼接
  字符串:字面名为 `a.b` 的键是一个段,不会与两段路径混淆。用
  `ErrorEnvelope::from_error(&err, source)` 构建,用 `to_json()` 或
  `push_json()` 渲染。不涉及 serde 依赖,每个字符串都按 RFC 8259
  转义。

