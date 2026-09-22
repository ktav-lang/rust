>>>>> lang=en
Key order is never changed. Canonical form has no sorting rule
(§ 5.9), and reordering keys would make review diffs worse, not
better — this is a spelling normaliser, not a refactoring tool.

For a document with no comments **and no blank lines**,
`format_str` equals `emit_canonical` of its parse. The stronger
condition is deliberate: blank lines are no more part of the `Value`
model than comments are, so `emit_canonical` drops them and
`format_str` does not.

### `ktav-fmt` — the optional command-line formatter

Behind the `cli` feature, off by default. In a project that already
has a toolchain the library call above plus a build hook is usually the
better answer, and editors format through `ktav-lsp`; the binary is
for the case where neither is at hand.

```text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
```

`--check` writes nothing and prints the path of every file that is
not already formatted, so it drops straight into CI next to
`cargo fmt --check`.

>>>>> lang=ru
Порядок ключей не меняется никогда. У канонической формы нет правила
сортировки (§ 5.9), а перестановка ключей ухудшила бы диффы на ревью,
а не улучшила: это нормализатор написания, а не инструмент
рефакторинга.

Для документа без комментариев **и без пустых строк** `format_str`
совпадает с `emit_canonical` его разбора. Условие намеренно сильнее
очевидного: пустые строки входят в модель `Value` ровно настолько же,
насколько комментарии, — то есть никак, поэтому `emit_canonical` их
выбрасывает, а `format_str` нет.

### `ktav-fmt` — необязательный форматтер командной строки

Живёт за feature-флагом `cli` и по умолчанию выключен. В проекте, где
тулчейн уже есть, библиотечный вызов выше плюс хук сборки обычно
уместнее, а редакторы форматируют через `ktav-lsp`; бинарь — для
случая, когда ни того, ни другого под рукой нет.

```text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
```

`--check` ничего не пишет и печатает путь каждого файла, который ещё
не отформатирован, — так что он встаёт в CI прямо рядом с
`cargo fmt --check`.

>>>>> lang=zh
键序永不改变。规范形式没有排序规则(§ 5.9),而重排键只会让评审差异
更糟,而非更好 —— 这是写法规范化工具,不是重构工具。

对于既无注释**也无空行**的文档,`format_str` 等于其解析结果的
`emit_canonical`。这个更强的条件是刻意的:空行与注释一样都不属于
`Value` 模型,因此 `emit_canonical` 会丢弃它们,而 `format_str`
不会。

### `ktav-fmt` —— 可选的命令行格式化器

位于 `cli` feature 之后,默认关闭。在已有工具链的项目里,上面的库
调用加一个构建钩子通常更合适,编辑器则经由 `ktav-lsp` 格式化;二进制
是留给两者都不可用的场景。

```text
cargo install ktav --features cli

ktav-fmt <file>...           format each file in place
ktav-fmt --stdout <file>     print the result, leave the file alone
ktav-fmt --check <file>...   exit non-zero if a file is not formatted
ktav-fmt -                   read one document from stdin
```

`--check` 不写入任何内容,只打印每个尚未格式化的文件路径,因此可以
直接放进 CI,紧挨着 `cargo fmt --check`。

