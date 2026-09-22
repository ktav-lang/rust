>>>>> lang=en
## Corners worth knowing

Four rules the examples above never reach. Each one is settled by the
specification rather than by this implementation, so every conforming
parser behaves the same way.

### Quoted keys — when a key contains a dot or a space

A dot in a bare key means nesting: `db.host: primary` builds
`db` → `host`. Quoting the key turns off that reading, so the dot is
part of the name (spec § 5.3.3):

```ktav
db.host: primary
"db.host": literal
"a b": spaces are fine too
```

The first line nests. The second is a single key literally named
`db.host`. This is why the error envelope reports `path` as an array
of segments rather than a joined string — a joined string could not tell
those two apart.

### Escapes apply inside inline compounds and quoted keys

`\uXXXX` and the named escapes (spec § 3.7, § 3.7.1) are read where
a delimiter would otherwise be structural — inside `{ }` and `[ ]`,
and inside a quoted key. A bare block-level value has no delimiters to
escape, so a backslash there is ordinary text:

>>>>> lang=ru
## Углы, о которых стоит знать

Четыре правила, до которых примеры выше не доходят. Каждое решено
спецификацией, а не этой реализацией, поэтому любой соответствующий
парсер ведёт себя так же.

### Кавычные ключи — когда в ключе точка или пробел

Точка в голом ключе означает вложенность: `db.host: primary`
строит `db` → `host`. Кавычки отключают это прочтение, и точка
становится частью имени (§ 5.3.3):

```ktav
db.host: primary
"db.host": literal
"a b": spaces are fine too
```

Первая строка вкладывает. Вторая — один ключ, буквально названный
`db.host`. Именно поэтому конверт ошибок отдаёт `path` массивом
сегментов, а не склеенной строкой: склеенная строка не смогла бы
различить эти два случая.

### Экранирование действует внутри inline-компаундов и кавычных ключей

`\uXXXX` и именованные escape-последовательности (§ 3.7, § 3.7.1)
читаются там, где разделитель иначе был бы структурным — внутри
`{ }` и `[ ]`, а также внутри кавычного ключа. У голого блочного
значения экранировать нечего, поэтому обратный слэш там — обычный
текст:

>>>>> lang=zh
## 值得知道的边角

以上示例未触及的四条规则。每一条都由规范而非本实现决定,因此任何
符合规范的解析器行为都一致。

### 引号键 —— 当键里有点号或空格时

裸键中的点号表示嵌套:`db.host: primary` 会构建 `db` → `host`。
给键加引号会关闭这一解读,点号因此成为名字的一部分(§ 5.3.3):

```ktav
db.host: primary
"db.host": literal
"a b": spaces are fine too
```

第一行产生嵌套。第二行是一个字面名为 `db.host` 的键。这正是错误
信封把 `path` 表示为段数组而非拼接字符串的原因 —— 拼接字符串无法区分
这两者。

### 转义在行内复合值与引号键中生效

`\uXXXX` 与具名转义(§ 3.7、§ 3.7.1)在分隔符本应具有结构含义的
位置被解读 —— 即 `{ }` 与 `[ ]` 内部,以及引号键内部。裸的块级值没有
需要转义的分隔符,因此其中的反斜杠是普通文本:

