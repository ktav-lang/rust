>>>>> lang=en
## Error model migration (0.9)

**0.9 migration (breaking):** `Error::InvalidUtf8` now contains
`{ valid_up_to, line, span }`; `ErrorKind::UnclosedCompound` contains
`{ kind, line, span }`. Update downstream constructors with the new fields;
use `..` in patterns when only selected fields matter. `line()` now returns
the source line for both errors. UTF-8 spans cover original malformed bytes;
EOF diagnostics cover the innermost opener through EOF and report its line.
Coordinates preserve BOM and LF/CR/CRLF; `Span::line_col` and envelope
`line_text` recognize all three terminators. I/O and writer failures retain
absent source positions. The ten-field JSON schema below is unchanged.
The new location oracles live in the draft `versions/0.9/tests` spec corpus;
this working version has not been published.

>>>>> lang=ru
## Миграция модели ошибок (0.9)

**Миграция 0.9 (breaking):** `Error::InvalidUtf8` теперь содержит
`{ valid_up_to, line, span }`, `ErrorKind::UnclosedCompound` —
`{ kind, line, span }`. В downstream-конструкторах нужно заполнить новые
поля; в patterns используйте `..`, если важна только часть полей.
`line()` теперь возвращает исходную строку обеих ошибок. UTF-8 span
покрывает исходные ошибочные байты; EOF-диагностика охватывает самый
внутренний незакрытый opener до EOF и указывает его строку. Координаты
сохраняют BOM и LF/CR/CRLF; `Span::line_col` и envelope `line_text`
учитывают все три терминатора. Ошибки I/O и writer по-прежнему не имеют
позиции исходника. Схема JSON из десяти полей ниже не меняется. Новые
oracles координат находятся в draft corpus спеки `versions/0.9/tests`;
эта рабочая версия ещё не опубликована.

>>>>> lang=zh
## 错误模型迁移（0.9）

**0.9 迁移（破坏性变更）：** `Error::InvalidUtf8` 现在包含
`{ valid_up_to, line, span }`，`ErrorKind::UnclosedCompound` 包含
`{ kind, line, span }`。下游构造器需要填写新增字段；只关注部分字段时，
模式应使用 `..`。两种错误的 `line()` 都返回源行号。UTF-8 区间覆盖原始
错误字节；EOF 诊断覆盖最内层未闭合起始标记至 EOF，并报告起始标记所在行。
坐标保留 BOM 及 LF/CR/CRLF；`Span::line_col` 和 envelope 的 `line_text`
识别全部三种换行形式。I/O 及 writer 错误仍不携带源位置。下述十字段 JSON
格式不变。新增位置 oracle 位于规范草案的 `versions/0.9/tests`；
当前工作版本尚未发布。

