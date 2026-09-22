>>>>> lang=en
- `Error::Syntax(String)` is preserved for backward compatibility —
  the public API stays deny-no-old-callers. Removal is deferred to
  ktav 1.0.
- Test count: 332 (0.1.4) → 391 (+59) plus 1 new doctest.
- The cabi/binding migration to consume `ErrorKind` over the FFI
  boundary is tracked separately in
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md) and ships as a
  coordinated ecosystem 0.2.0.

### SemVer note

Adding `#[non_exhaustive]` to a previously-unmarked enum (`Error`,
`ConflictKind`, `CompoundKind`) is, per the
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
a breaking change that would normally require a major bump (0.2.0).
This release ships as **0.1.5** intentionally:

1. Pre-1.0 Cargo convention permits breaking changes on any bump,
   including patches.
2. All known downstream consumers of `ktav::Error` (the six language
   bindings under `ktav-lang/`) call `Err(e) => e.to_string()` only.
   No exhaustive `match err { Error::Io(_) => …, Error::Syntax(_)
   => …, Error::Message(_) => … }` patterns exist in the ecosystem
   that this change would silently break.
3. The seven canonical-category Display strings remain byte-identical
   to 0.1.4, so any hypothetical out-of-tree consumer doing string
   matching keeps working unmodified.

If your code does keep an exhaustive match over `ktav::Error` and
this release breaks it, add an `_ => …` arm. That arm is now
required forever and will not need to change again as future
variants are added.

>>>>> lang=ru
- `Error::Syntax(String)` сохранён для обратной совместимости —
  публичный API остаётся deny-no-old-callers. Удаление отложено до
  ktav 1.0.
- Тестов: 332 (0.1.4) → 391 (+59) плюс 1 новый doctest.
- Миграция cabi/биндингов на потребление `ErrorKind` через FFI
  трекается отдельно в
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md) и идёт как
  coordinated ecosystem 0.2.0.

### SemVer-замечание

Добавление `#[non_exhaustive]` к ранее непомеченным enum-ам (`Error`,
`ConflictKind`, `CompoundKind`) — согласно
[Cargo SemVer reference](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive)
— breaking change, требующий major-bump-а (0.2.0). Этот релиз
выпускается как **0.1.5** намеренно:

1. Pre-1.0 Cargo-конвенция допускает breaking-изменения на любом
   bump-е, включая патчи.
2. Все известные downstream-потребители `ktav::Error` (шесть
   языковых биндингов под `ktav-lang/`) делают только
   `Err(e) => e.to_string()`. Exhaustive `match err { Error::Io(_)
   => …, Error::Syntax(_) => …, Error::Message(_) => … }` в
   экосистеме нет — ломать тихо нечего.
3. Display-строки семи канонических категорий byte-identical к
   0.1.4, поэтому любой гипотетический out-of-tree-потребитель,
   делающий string matching, продолжает работать без изменений.

Если ваш код всё-таки держит exhaustive `match` по `ktav::Error` и
этот релиз его ломает — добавьте arm `_ => …`. Этот arm теперь
обязателен навсегда и больше не потребует изменений при добавлении
будущих вариантов.

>>>>> lang=zh
- 为了向后兼容,`Error::Syntax(String)` 保留 —— 公共 API 仍不拒绝
  老调用方。移除推迟到 ktav 1.0。
- 测试数:332 (0.1.4) → 391 (+59),外加 1 个新 doctest。
- cabi/绑定迁移以通过 FFI 边界消费 `ErrorKind` 的工作单独记录在
  [`STRUCTURED_ERRORS.md`](../STRUCTURED_ERRORS.md),并作为协调发布
  的生态系统 0.2.0 一并交付。

### SemVer 说明

按
[Cargo SemVer 参考](https://doc.rust-lang.org/cargo/reference/semver.html#enum-non-exhaustive),
为先前未标注的枚举(`Error`、`ConflictKind`、`CompoundKind`)添加
`#[non_exhaustive]` 是破坏性变更,通常需要主版本号 bump(0.2.0)。
本次发布有意作为 **0.1.5** 推出:

1. Pre-1.0 的 Cargo 惯例允许在任何 bump(包括 patch)上做破坏性
   变更。
2. `ktav::Error` 所有已知的下游消费者(`ktav-lang/` 下的六个语言
   绑定)均仅调用 `Err(e) => e.to_string()`。生态系统中不存在会被
   该变更悄悄破坏的穷尽 `match err { Error::Io(_) => …,
   Error::Syntax(_) => …, Error::Message(_) => … }` 模式。
3. 七个标准类别的 Display 字符串与 0.1.4 字节相同,因此任何在
   tree 之外做字符串匹配的假想消费者也无需修改即可继续工作。

如果你的代码确实保留了对 `ktav::Error` 的穷尽 `match` 而本次发布
将其破坏 —— 请添加 `_ => …` 分支。该分支从此永久必须存在,且不会
随未来变体的添加而再次需要变更。

