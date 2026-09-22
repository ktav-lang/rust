>>>>> lang=en
```text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
```

A value like `port: abc` parses fine *at the Ktav level* (string
`"abc"`), but `serde::deserialize` into `u16` will return a clear
`ParseError`.

### Booleans: `true` / `false`

Strict lowercase. Anything else is a string.

>>>>> lang=ru
```text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
```

Значение вида `port: abc` парсится нормально *на уровне Ktav* (строка
`"abc"`), но `serde::deserialize` в `u16` вернёт понятный
`ParseError`.

### Булевы: `true` / `false`

Строго нижний регистр. Всё остальное — строка.

>>>>> lang=zh
```text
port: 8080
ratio: 3.14159
offset: -42
huge: 1234567890123
```

像 `port: abc` 这样的值在 *Ktav 层*可以正常解析(即字符串 `"abc"`),
但 `serde::deserialize` 到 `u16` 时会返回一个清晰的 `ParseError`。

### 布尔:`true` / `false`

严格小写。其它写法都是字符串。

