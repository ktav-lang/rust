>>>>> lang=en
### Performance (criterion, 22 KB typed config, Windows release)

- `parse → struct`: **275 µs** (~80 MB/s)
- `render struct → text`: **46 µs** (~475 MB/s)
- `round-trip`: **377 µs**

### Dependencies

- `serde` with `derive`
- `indexmap` with the `serde` feature
- `rustc-hash` (FxHash — fast and deterministic; not
  collision-resistant, which a config parser does not need)

### MSRV

`rustc 1.70` or newer.
>>>>> lang=ru
### Performance (criterion, typed-конфиг 22 KB, Windows release)

- `parse → struct`: **275 µs** (~80 MB/s)
- `render struct → text`: **46 µs** (~475 MB/s)
- `round-trip`: **377 µs**

### Dependencies

- `serde` с `derive`
- `indexmap` с фичей `serde`
- `rustc-hash` (FxHash — быстрый и детерминированный; не
  устойчив к коллизиям, а парсеру конфигов это и не нужно)

### MSRV

`rustc 1.70` или новее.
>>>>> lang=zh
### Performance(criterion,22 KB 的 typed 配置,Windows release)

- `parse → struct`: **275 µs**(~80 MB/s)
- `render struct → text`: **46 µs**(~475 MB/s)
- `round-trip`: **377 µs**

### Dependencies

- `serde`(含 `derive`)
- `indexmap`(启用 `serde` 特性)
- `rustc-hash`(FxHash —— 快且确定性;不抗碰撞,而配置解析器
  并不需要抗碰撞)

### MSRV

`rustc 1.70` 或更新版本。
