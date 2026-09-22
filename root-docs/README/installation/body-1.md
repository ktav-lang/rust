>>>>> lang=en
## Installation

```toml
[dependencies]
ktav = "0.8.0"
serde = { version = "1", features = ["derive"] }
```

The formatter is also available as a binary, behind an off-by-default
feature:

```sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
```

>>>>> lang=ru
## Установка

```toml
[dependencies]
ktav = "0.8.0"
serde = { version = "1", features = ["derive"] }
```

Форматтер доступен и как исполняемый файл — за выключенным по
умолчанию feature-флагом:

```sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
```

>>>>> lang=zh
## 安装

```toml
[dependencies]
ktav = "0.8.0"
serde = { version = "1", features = ["derive"] }
```

格式化器也可作为二进制取用,位于一个默认关闭的 feature 之后:

```sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
```

