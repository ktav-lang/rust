>>>>> lang=en
```text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl

hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
```
```json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
```

For pairs the marker sits between key and value; for array items it
stands at the start of the line. **Serialization emits `::`
automatically** when a string value begins with `{` or `[`, so
round-tripping regexes and IPv6 addresses just works.

### 8. Comments

>>>>> lang=ru
```text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl

hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
```
```json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
```

Для пар маркер стоит между ключом и значением; для элементов массива —
в начале строки. **Сериализация эмитит `::` автоматически**, когда
строковое значение начинается с `{` или `[`, так что round-trip
regex-ов и IPv6-адресов просто работает.

### 8. Комментарии

>>>>> lang=zh
```text
pattern:: [a-z]+
ipv6:: [::1]:8080
template:: {issue.id}.tpl

hosts: [
    ok.example
    :: [::1]
    :: [2001:db8::1]:53
]
```
```json5
{
  pattern: "[a-z]+",
  ipv6: "[::1]:8080",
  template: "{issue.id}.tpl",
  hosts: ["ok.example", "[::1]", "[2001:db8::1]:53"]
}
```

对于键值对,该标记位于键和值之间;对于数组元素,它位于行首。
**当字符串值以 `{` 或 `[` 开头时,序列化会自动输出 `::`**,所以
正则与 IPv6 地址的 round-trip 自然工作。

### 8. 注释

