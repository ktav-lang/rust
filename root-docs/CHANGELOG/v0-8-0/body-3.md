>>>>> lang=en
### Added

- **`ktav::cabi` and the `declare_cabi!` macro** — the shareable
  half of the C ABI shim: the tagged `WireValue` decode (ordered
  maps, lossless integers of any size), the seven document operations
  `loads`, `loads_strict`, `dumps`, `dumps_force_strings`,
  `emit_canonical` and the new `format` and
  `canonical_from_source`, and envelope encoding for every failure,
  including the non-`ktav` ones. The module contains no
  `extern "C"`: symbols defined in a dependency rlib are not
  guaranteed to survive into a downstream cdylib. The macro expands the
  ten `#[no_mangle]` symbols into the calling crate instead, where
  export is guaranteed by construction.

- **`ktav_abi_version()` and `ktav::cabi::ABI_VERSION`** — the
  version of the exported ABI *shape* (starts at 1), so a host that
  picks up a stale native library refuses to load instead of corrupting
  memory. The crate version changes every release; the ABI shape almost
  never.

- **`ktav::cabi::library_file_name` and `docs/CABI.md`** — the
  artifact naming convention that the `php`, `csharp` and
  `golang` loaders each derived independently
  (`ktav_cabi-windows-amd64.dll`,
  `libktav_cabi-darwin-arm64.dylib`, …, the `$KTAV_LIB_PATH`
  override), written down once, executable, and pinned by tests.

>>>>> lang=ru
### Added

- **`ktav::cabi` и макрос `declare_cabi!`** — разделяемая половина
  C ABI-шима: тегированное декодирование `WireValue` (упорядоченные
  карты, целые любой величины без потерь), семь операций над
  документами `loads`, `loads_strict`, `dumps`,
  `dumps_force_strings`, `emit_canonical` и новые `format` и
  `canonical_from_source`, и кодирование любого сбоя — включая
  не-ktav-овские — в конверт. В модуле нет ни одного `extern "C"`:
  символы, определённые в rlib-зависимости, не гарантированно
  выживают в cdylib потребителя. Макрос вместо этого разворачивает
  десять символов `#[no_mangle]` в вызывающий crate, где экспорт
  гарантирован по построению.

- **`ktav_abi_version()` и `ktav::cabi::ABI_VERSION`** — версия
  *формы* экспортируемого ABI (начинается с 1): хост, получивший
  устаревшую нативную библиотеку, откажется от загрузки вместо порчи
  памяти. Версия crate меняется в каждом выпуске, форма ABI — почти
  никогда.

- **`ktav::cabi::library_file_name` и `docs/CABI.md`** — конвенция
  имён артефактов, которую загрузчики `php`, `csharp` и `golang`
  выводили каждый по-своему (`ktav_cabi-windows-amd64.dll`,
  `libktav_cabi-darwin-arm64.dylib`, …, переопределение через
  `$KTAV_LIB_PATH`), записанная один раз, исполнимая и закреплённая
  тестами.

>>>>> lang=zh
### 新增

- **`ktav::cabi` 与 `declare_cabi!` 宏** —— C ABI 垫片中可共享的
  一半:带标签的 `WireValue` 解码(有序映射、任意大小整数无损),七项
  文档操作 `loads`、`loads_strict`、`dumps`、
  `dumps_force_strings`、`emit_canonical` 与新增的 `format` 和
  `canonical_from_source`,以及把每一个错误——包括非 ktav 的
  错误——编码进信封。模块本身不含任何 `extern "C"`:依赖 rlib
  中定义的符号不保证能存活到下游的 cdylib。宏改为把十个
  `#[no_mangle]` 符号展开进调用方的 crate,在那里导出由构造方式
  保证。

- **`ktav_abi_version()` 与 `ktav::cabi::ABI_VERSION`** —— 导出
  ABI *形态* 的版本(从 1 开始):拿到过期原生库的宿主会拒绝加载而不是
  破坏内存。crate 版本每次发布都变,ABI 形态几乎从不变。

- **`ktav::cabi::library_file_name` 与 `docs/CABI.md`** ——
  `php`、`csharp` 与 `golang` 的加载器各自独立推导的产物命名约定
  (`ktav_cabi-windows-amd64.dll`、
  `libktav_cabi-darwin-arm64.dylib`、……、`$KTAV_LIB_PATH` 覆盖),
  如今一次性写下来、可执行、并由测试钉死。

