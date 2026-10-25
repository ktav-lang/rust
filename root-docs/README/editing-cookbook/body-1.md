>>>>> lang=en
### Source-edit cookbook

These examples describe the unreleased 0.9 API, not the published 0.8.1 package. They replace existing values only: keys are not renamed, missing paths are not created, and entries are not removed. Each snippet is independent; Rust `?` assumes an enclosing `Result`-returning function. Go examples panic only for these fixed, known-valid inputs; applications should handle the returned error.

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;
```

#### 1. Dotted deployment settings

Only the address changes; the comment, dotted form and neighbouring user entry remain byte-for-byte unchanged.

```rust
let source = "## deployment\ntargets.storagehost.address: old\ntargets.storagehost.user: mcp-agent\n";
let updated = edit_value(source, "targets.storagehost.address", &Value::String("192.168.0.88:22".into()))?;
```

Result:

```ktav
## deployment
targets.storagehost.address: 192.168.0.88:22
targets.storagehost.user: mcp-agent
```

#### 2. Literal dots: textual path or segments

>>>>> lang=ru
### Практические примеры правки исходника

Примеры относятся к ещё не выпущенному API 0.9, а не к опубликованному пакету 0.8.1. Они заменяют только существующие значения: не переименовывают ключи, не создают отсутствующие пути и не удаляют записи. Фрагменты независимы; Rust `?` предполагает внешнюю функцию с результатом `Result`. В Go panic используется только для фиксированных корректных данных примера; приложение должно обработать возвращённую ошибку.

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;
```

#### 1. Настройка развёртывания через dotted-ключ

Меняется только адрес; комментарий, dotted-форма и соседнее поле пользователя сохраняются байт-в-байт.

```rust
let source = "## deployment\ntargets.storagehost.address: old\ntargets.storagehost.user: mcp-agent\n";
let updated = edit_value(source, "targets.storagehost.address", &Value::String("192.168.0.88:22".into()))?;
```

Результат:

```ktav
## deployment
targets.storagehost.address: 192.168.0.88:22
targets.storagehost.user: mcp-agent
```

#### 2. Буквальные точки: текстовый путь или сегменты

>>>>> lang=zh
### 源文本编辑示例

以下示例适用于尚未发布的 0.9 API，而非已发布的 0.8.1 包。只能替换已有值：不会重命名键、创建缺失路径或删除条目。各代码片段相互独立；Rust 的 `?` 假定外层函数返回 `Result`。Go 示例仅针对固定且有效的输入使用 panic；应用程序应处理返回的错误。

```rust
use ktav::edit::{edit_value, edit_value_segments};
use ktav::Value;
```

#### 1. 通过点分键修改部署配置

只修改地址；注释、点分形式及相邻的用户字段逐字节保持不变。

```rust
let source = "## deployment\ntargets.storagehost.address: old\ntargets.storagehost.user: mcp-agent\n";
let updated = edit_value(source, "targets.storagehost.address", &Value::String("192.168.0.88:22".into()))?;
```

结果:

```ktav
## deployment
targets.storagehost.address: 192.168.0.88:22
targets.storagehost.user: mcp-agent
```

#### 2. 字面点号：文本路径或路径段

