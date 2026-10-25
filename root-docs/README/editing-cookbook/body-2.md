>>>>> lang=en
The quoted source key is literally `a.b`. The textual path escapes its dot; the segments call passes the decoded key directly. Both calls return the same result.

```rust
let source = "\"a.b\": {port: 80}\n";
let updated = edit_value(source, "a\\.b.port", &Value::Integer("90".into()))?;
let same = edit_value_segments(source, &["a.b", "port"], &Value::Integer("90".into()))?;
```

Result:

```ktav
"a.b": {port: 90}
```

#### 3. A leaf inside a mixed-form array

`0` is an array index here. Inside an Object, the same segment would address the key named `0`; untouched containers remain inline.

```rust
let source = "servers: [{host: old, keep: yes}, standby]\n";
let updated = edit_value_segments(source, &["servers", "0", "host"], &Value::String("new".into()))?;
```

Result:

```ktav
servers: [{host: new, keep: yes}, standby]
```

>>>>> lang=ru
Ключ исходника в кавычках буквально называется `a.b`. В текстовом пути точка экранируется; API сегментов получает декодированное имя напрямую. Оба вызова возвращают одинаковый результат.

```rust
let source = "\"a.b\": {port: 80}\n";
let updated = edit_value(source, "a\\.b.port", &Value::Integer("90".into()))?;
let same = edit_value_segments(source, &["a.b", "port"], &Value::Integer("90".into()))?;
```

Результат:

```ktav
"a.b": {port: 90}
```

#### 3. Лист внутри массива со смешанными формами

Здесь `0` — индекс массива. В объекте такой же сегмент обозначал бы ключ с именем `0`; нетронутые контейнеры остаются inline.

```rust
let source = "servers: [{host: old, keep: yes}, standby]\n";
let updated = edit_value_segments(source, &["servers", "0", "host"], &Value::String("new".into()))?;
```

Результат:

```ktav
servers: [{host: new, keep: yes}, standby]
```

>>>>> lang=zh
源码中带引号的键名就是 `a.b`。文本路径需要转义点号；路径段 API 直接接收解码后的键名。两个调用返回相同结果。

```rust
let source = "\"a.b\": {port: 80}\n";
let updated = edit_value(source, "a\\.b.port", &Value::Integer("90".into()))?;
let same = edit_value_segments(source, &["a.b", "port"], &Value::Integer("90".into()))?;
```

结果:

```ktav
"a.b": {port: 90}
```

#### 3. 修改混合形式数组中的叶子值

此处 `0` 是数组索引；在对象中，同一个路径段表示名称为 `0` 的键。未修改的容器保持行内形式。

```rust
let source = "servers: [{host: old, keep: yes}, standby]\n";
let updated = edit_value_segments(source, &["servers", "0", "host"], &Value::String("new".into()))?;
```

结果:

```ktav
servers: [{host: new, keep: yes}, standby]
```

