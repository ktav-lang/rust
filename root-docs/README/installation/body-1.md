>>>>> lang=en
## Installation

The published dependency below is 0.8.1; it does not include the new path
editing or 0.9 error model. Version 0.9.0 currently requires this source checkout.

```toml
[dependencies]
ktav = "0.8.1"
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

Ниже указана опубликованная 0.8.1: в ней нет новых методов редактирования
по пути и модели ошибок 0.9. Для готовящейся 0.9.0 пока нужен этот source checkout.

```toml
[dependencies]
ktav = "0.8.1"
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

下面的依赖为已发布的 0.8.1，不包含新增路径编辑 API 或 0.9 错误模型。
准备中的 0.9.0 目前需要使用本源代码 checkout。

```toml
[dependencies]
ktav = "0.8.1"
serde = { version = "1", features = ["derive"] }
```

格式化器也可作为二进制取用,位于一个默认关闭的 feature 之后:

```sh
cargo install ktav --locked --features cli
ktav-fmt --check config.ktav
```

