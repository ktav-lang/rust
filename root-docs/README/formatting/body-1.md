>>>>> lang=en
## Formatting — canonical spelling, comments kept

`ktav::format_str` rewrites a document into the structural spelling
`emit_canonical` produces, but keeps the trivia the canonical writer
drops. Every comment survives verbatim.

```rust
let tidied = ktav::format_str("## the server\nserver: {host: a, port: 80}\n")?;
assert_eq!(tidied, "## the server\nserver: {\n    host: a\n    port: 80\n}\n");
```

That is, the inline compound expands to the canonical multi-line
form and the comment stays exactly where it was:

```ktav
## the server
server: {
    host: a
    port: 80
}
```

Blank lines survive as a grouping hint, but a run of two or more
collapses to exactly one, and blank padding just inside a bracket is
dropped. That is what makes the transform a fixed point: formatting
already-formatted output never changes it again.

>>>>> lang=ru
## Форматирование — каноническое написание, комментарии на месте

`ktav::format_str` переписывает документ в то структурное написание,
которое выдаёт `emit_canonical`, но сохраняет оформление, выбрасываемое
каноническим писателем. Каждый комментарий сохраняется дословно.

```rust
let tidied = ktav::format_str("## the server\nserver: {host: a, port: 80}\n")?;
assert_eq!(tidied, "## the server\nserver: {\n    host: a\n    port: 80\n}\n");
```

То есть inline-компаунд разворачивается в каноническую многострочную
форму, а комментарий остаётся ровно там, где был:

```ktav
## the server
server: {
    host: a
    port: 80
}
```

Пустые строки сохраняются как подсказка группировки, но серия из двух
и более схлопывается ровно в одну, а пустой отступ сразу внутри скобки
выбрасывается. Именно это делает преобразование неподвижной точкой:
форматирование уже отформатированного вывода больше ничего не меняет.

>>>>> lang=zh
## 格式化 —— 规范写法,注释保留

`ktav::format_str` 把文档改写为 `emit_canonical` 所产出的结构写法,
但保留规范写入器会丢弃的附属内容。每条注释都逐字保留。

```rust
let tidied = ktav::format_str("## the server\nserver: {host: a, port: 80}\n")?;
assert_eq!(tidied, "## the server\nserver: {\n    host: a\n    port: 80\n}\n");
```

也就是说,行内复合值展开为规范的多行形式,而注释仍停留在原处:

```ktav
## the server
server: {
    host: a
    port: 80
}
```

空行作为分组提示保留,但连续两行及以上会折叠为恰好一行,紧贴括号内侧
的空行填充会被丢弃。正是这一点让该变换成为不动点:对已格式化的输出再
次格式化不会再有任何改变。

