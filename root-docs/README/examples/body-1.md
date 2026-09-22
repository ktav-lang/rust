>>>>> lang=en
## Examples: Ktav → JSON5

JSON5 is on the right because it reads like ordinary JavaScript, allows
comments, and shows exactly what the parser produces.

### 1. Scalars

```text
name: Russia
port: 20082
```
```json5
{
  name: "Russia",
  port: 20082
}
```

Scalars are typed at the `Value` level from their lexical form
(`Integer`/`Float`/`Bool`/`Null`/`String`); force a literal string with
the `::` raw marker (e.g. `port:: 20082`) when a numeric-looking body
must stay a string.

### 2. Dotted keys = nested objects

>>>>> lang=ru
## Примеры: Ktav → JSON5

JSON5 справа потому, что читается как обычный JavaScript, допускает
комментарии и показывает ровно то, что производит парсер.

### 1. Скаляры

```text
name: Russia
port: 20082
```
```json5
{
  name: "Russia",
  port: 20082
}
```

Скаляры типизируются на уровне `Value` по своей лексической форме
(`Integer`/`Float`/`Bool`/`Null`/`String`); принудить числоподобное
тело остаться строкой можно raw-маркером `::` (например,
`port:: 20082`).

### 2. Точечные ключи = вложенные объекты

>>>>> lang=zh
## 示例:Ktav → JSON5

右侧使用 JSON5,因为它读起来像普通 JavaScript,允许注释,并且
能完整展示解析器产出的结果。

### 1. 标量

```text
name: Russia
port: 20082
```
```json5
{
  name: "Russia",
  port: 20082
}
```

标量在 `Value` 层就已根据其字面形式被分类
(`Integer`/`Float`/`Bool`/`Null`/`String`);若数字形状的内容需要保持
字符串,可用 `::` raw 标记强制(例如 `port:: 20082`)。

### 2. 点分键 = 嵌套对象

