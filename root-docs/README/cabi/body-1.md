>>>>> lang=en
## The C ABI for other languages

Six language bindings — Go, Java, PHP, C#, JS and Python — load a
small native library built on this crate. The shareable half of that
shim lives behind the off-by-default `cabi` feature: the
`ktav::cabi` module carries the wire decoding, the seven document
operations and the error encoding, and one macro call in a binding's
`cdylib` expands the exported symbols.

```text
// crates/cabi/src/lib.rs of a binding — the whole body:
ktav::declare_cabi!();
```

The expansion exports ten symbols: the seven document functions
(`ktav_loads`, `ktav_loads_strict`, `ktav_dumps`,
`ktav_dumps_force_strings`, `ktav_emit_canonical`,
`ktav_format`, `ktav_canonical_from_source`), `ktav_free`,
`ktav_version` and `ktav_abi_version`. Every error leaves as the
ten-field JSON envelope, so a host never has to sniff plain text
against JSON, and
`ktav_abi_version()` lets a host refuse a stale native library
instead of corrupting memory. The full contract — signatures,
ownership, the error encoding, the artifact naming convention
(`ktav_cabi-windows-amd64.dll`, `libktav_cabi-darwin-arm64.dylib`,
`libktav_cabi-linux-amd64.so`, the `$KTAV_LIB_PATH` override) — is
specified in [docs/CABI.md](docs/CABI.md).

>>>>> lang=ru
## C ABI для других языков

Шесть языковых биндингов — Go, Java, PHP, C#, JS и Python — загружают
небольшую нативную библиотеку, собранную на этом crate. Разделяемая
половина шима живёт за выключенной по умолчанию фичей `cabi`: модуль
`ktav::cabi` несёт декодирование wire-формата, семь операций над
документами и кодирование ошибок, а один макровызов в `cdylib`
биндинга разворачивает экспортируемые символы.

```text
// crates/cabi/src/lib.rs of a binding — the whole body:
ktav::declare_cabi!();
```

Развёртка экспортирует десять символов: семь функций-операций
(`ktav_loads`, `ktav_loads_strict`, `ktav_dumps`,
`ktav_dumps_force_strings`, `ktav_emit_canonical`,
`ktav_format`, `ktav_canonical_from_source`), `ktav_free`,
`ktav_version` и `ktav_abi_version`. Каждая ошибка уходит
десятиполевым JSON-конвертом, поэтому хосту не приходится гадать,
текст перед ним или JSON, а `ktav_abi_version()` позволяет хосту
отказаться от
устаревшей нативной библиотеки вместо порчи памяти. Полный контракт —
сигнатуры, владение, кодирование ошибок, конвенция имён артефактов
(`ktav_cabi-windows-amd64.dll`, `libktav_cabi-darwin-arm64.dylib`,
`libktav_cabi-linux-amd64.so`, переопределение `$KTAV_LIB_PATH`) —
описан в [docs/CABI.md](docs/CABI.md).

>>>>> lang=zh
## 面向其他语言的 C ABI

六个语言绑定——Go、Java、PHP、C#、JS 与 Python——加载一个构建在
本 crate 之上的小型原生库。垫片中可共享的一半位于默认关闭的
`cabi` feature 之后:`ktav::cabi` 模块承载 wire 解码、七项文档
操作与错误编码,而绑定 cdylib 中的一次宏调用即可展开全部导出符号。

```text
// crates/cabi/src/lib.rs of a binding — the whole body:
ktav::declare_cabi!();
```

展开导出十个符号:七个文档操作函数(`ktav_loads`、
`ktav_loads_strict`、`ktav_dumps`、`ktav_dumps_force_strings`、
`ktav_emit_canonical`、`ktav_format`、
`ktav_canonical_from_source`)、`ktav_free`、`ktav_version`
与 `ktav_abi_version`。每个错误都以十字段 JSON
信封传递,宿主无需嗅探它是纯文本还是 JSON;`ktav_abi_version()`
让宿主拒绝过期的原生库而不是破坏内存。完整契约——签名、所有权、
错误编码、产物命名约定(`ktav_cabi-windows-amd64.dll`、
`libktav_cabi-darwin-arm64.dylib`、`libktav_cabi-linux-amd64.so`、
`$KTAV_LIB_PATH` 覆盖)——见
[docs/CABI.md](docs/CABI.md)。

