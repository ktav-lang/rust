>>>>> lang=en
- **`spec-version` metadata moves to `0.8.0`, and the pinned
  `spec` submodule with it.** The suite now runs the 0.8 conformance
  corpus: 223 `valid/`, 74 `invalid/`, 5 `unrepresentable/`, 4
  `parseable-unrepresentable/` and 13 `strict-lossy/` fixtures —
  the exact counts § 8.5's `manifest.json` declares, loaded and
  checked before any fixture is enumerated. The rule above is pinned by
  `valid/numbers/integer/leading_zero_is_string` and
  `valid/numbers/float/leading_zero_is_string`, which carry both sides
  of the boundary in one document each, so every implementation is held
  to it and not just this one.

- **`ktav_version()` now reports this crate's version, not the
  binding's.** Each binding's private shim used to return its own
  package version; the shared macro cannot know it. The number a host
  reads therefore changes meaning, and constants such as a binding's
  `LIB_VERSION` become stale without any error.

  This is the honest reading — the native library *is* `ktav` — but
  it removes the check those constants were performing. Use
  `ktav_abi_version()` for that instead: it answers "is this library
  the shape I was built against", which is the question a loader
  actually has, and it does not move on every release.

>>>>> lang=ru
- **Метаданные `spec-version` переходят на `0.8.0`, а вместе с ними
  и закреплённый submodule `spec`.** Набор тестов теперь идёт по
  conformance-корпусу 0.8: 223 фикстуры `valid/`, 74 `invalid/`, 5
  `unrepresentable/`, 4 `parseable-unrepresentable/` и 13
  `strict-lossy/` — в точности те количества, которые объявляет
  `manifest.json` из § 8.5, загружаемые и проверяемые до перечисления
  любой фикстуры. Правило выше закреплено фикстурами
  `valid/numbers/integer/leading_zero_is_string` и
  `valid/numbers/float/leading_zero_is_string`, каждая из которых
  несёт обе стороны границы в одном документе, так что правилу
  подчиняется каждая реализация, а не только эта.

- **`ktav_version()` теперь сообщает версию этого crate-а, а не
  биндинга.** Приватный шим каждого биндинга возвращал собственную
  версию пакета; общий макрос знать её не может. Смысл числа, которое
  читает host, тем самым меняется, а константы вроде `LIB_VERSION` в
  биндинге устаревают без единой ошибки.

  Это честное прочтение — нативная библиотека и **есть** `ktav`, — но
  оно убирает проверку, которую эти константы выполняли. Для неё
  используйте `ktav_abi_version()`: он отвечает на вопрос «та ли это
  библиотека по форме, под которую меня собрали», то есть на настоящий
  вопрос загрузчика, и не меняется каждый выпуск.

>>>>> lang=zh
- **`spec-version` 元数据升至 `0.8.0`,固定的 `spec` submodule
  也随之移动。** 测试套件现在跑的是 0.8 一致性语料库:223 个
  `valid/`、74 个 `invalid/`、5 个 `unrepresentable/`、4 个
  `parseable-unrepresentable/` 与 13 个 `strict-lossy/` fixture ——
  正是 § 8.5 的 `manifest.json` 所声明的精确数量,并在枚举任何
  fixture 之前加载与校验。上述规则由
  `valid/numbers/integer/leading_zero_is_string` 与
  `valid/numbers/float/leading_zero_is_string` 钉死,两者各自在一份
  文档里同时承载边界的两侧,因此受约束的是每一个实现,而不只是这一个。

- **`ktav_version()` 现在报告本 crate 的版本,而非绑定的版本。**
  各绑定的私有垫片过去返回自己的包版本;共享宏无从得知它。因此宿主
  读到的数字含义发生了变化,绑定中诸如 `LIB_VERSION` 的常量会在没有
  任何报错的情况下过时。

  这是更诚实的读法 —— 原生库**就是** `ktav` —— 但它移除了那些常量
  原本承担的检查。请改用 `ktav_abi_version()`:它回答「这个库是否
  与我构建时的形状一致」,这才是加载器真正的问题,而且它不会每次
  发布都变动。

