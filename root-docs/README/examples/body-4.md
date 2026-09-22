>>>>> lang=en
```text
upstreams: [
    {
        host: a.example
        port: 1080
    }
    {
        host: b.example
        port: 1080
    }
]
```
```json5
{
  upstreams: [
    { host: "a.example", port: 1080 },
    { host: "b.example", port: 1080 }
  ]
}
```

### 6. Arbitrary nesting

Every compound value spans multiple lines (single-line `{ ... }` / `[ ... ]`
with contents is not accepted — only the empty forms `{}` / `[]` are
inline). Nest as deep as needed:

>>>>> lang=ru
```text
upstreams: [
    {
        host: a.example
        port: 1080
    }
    {
        host: b.example
        port: 1080
    }
]
```
```json5
{
  upstreams: [
    { host: "a.example", port: 1080 },
    { host: "b.example", port: 1080 }
  ]
}
```

### 6. Произвольная вложенность

Каждое составное значение занимает несколько строк (однострочные
`{ ... }` / `[ ... ]` с содержимым не принимаются — инлайн разрешены
только пустые формы `{}` / `[]`). Вкладывайте сколько угодно:

>>>>> lang=zh
```text
upstreams: [
    {
        host: a.example
        port: 1080
    }
    {
        host: b.example
        port: 1080
    }
]
```
```json5
{
  upstreams: [
    { host: "a.example", port: 1080 },
    { host: "b.example", port: 1080 }
  ]
}
```

### 6. 任意层嵌套

每个复合值都跨越多行(带内容的单行 `{ ... }` / `[ ... ]` 不被接受
——只有空形式 `{}` / `[]` 允许写作内联)。想嵌多深就嵌多深:

