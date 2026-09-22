>>>>> lang=en
## Using it from Rust

Ktav is serde-native. Any type implementing `Serialize` / `Deserialize`
(including `#[derive]`-generated ones) round-trips through Ktav out of
the box.

### Parse — decode straight into a typed struct

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Db { host: String, timeout: u32 }

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port:    u16,
    ratio:   f64,
    tls:     bool,
    tags:    Vec<String>,
    db:      Db,
}

>>>>> lang=ru
## Использование из Rust

Ktav — serde-нативный. Любой тип, реализующий `Serialize` /
`Deserialize` (включая сгенерированные через `#[derive]`),
round-trip-ится через Ktav из коробки.

### Парсинг — декод сразу в типизированную структуру

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Db { host: String, timeout: u32 }

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port:    u16,
    ratio:   f64,
    tls:     bool,
    tags:    Vec<String>,
    db:      Db,
}

>>>>> lang=zh
## 在 Rust 中使用

Ktav 原生支持 serde。任何实现了 `Serialize` / `Deserialize` 的类型
(包括 `#[derive]` 生成的)都可以开箱即用地通过 Ktav 完成
round-trip。

### 解析 —— 直接解码到类型化结构体

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Db { host: String, timeout: u32 }

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    service: String,
    port:    u16,
    ratio:   f64,
    tls:     bool,
    tags:    Vec<String>,
    db:      Db,
}

