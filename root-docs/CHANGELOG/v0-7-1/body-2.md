>>>>> lang=en
- **`format_str` — a comment-preserving formatter.** It normalises a
  document's structural spelling to what `emit_canonical` produces,
  interleaved with the trivia the canonical writer drops. Every comment
  survives verbatim. Blank lines survive as a grouping hint, but a run
  of two or more collapses to one and blank padding just inside a
  bracket is dropped — which is what makes the transform a fixed
  point. Key order is never changed: canonical form has no
  sorting rule (§ 5.9), and reordering keys would make review diffs
  worse, not better. For a document with no comments *and no
  blank lines*, `format_str` equals `emit_canonical` of its parse.

- **`ktav-fmt` — an optional command-line formatter**, behind the
  `cli` feature. The repository shipped no binary at all before this
  release.

  ```text
  cargo install ktav --features cli

  ktav-fmt <file>...           format each file in place
  ktav-fmt --stdout <file>     print the result, leave the file alone
  ktav-fmt --check <file>...   exit non-zero if a file is not formatted
  ktav-fmt -                   read one document from stdin
  ```

>>>>> lang=ru
- **`format_str` — форматтер, сохраняющий комментарии.** Приводит
  структурное написание документа к тому, что выдаёт
  `emit_canonical`, вперемежку с тем оформлением, которое канонический
  писатель выбрасывает. Каждый комментарий сохраняется дословно. Пустые
  строки сохраняются как подсказка группировки, но серия из двух и
  более схлопывается в одну, а пустой отступ сразу внутри скобки
  выбрасывается — именно это делает преобразование неподвижной
  точкой. Порядок ключей не меняется
  никогда: у канонической формы нет правила сортировки (§ 5.9), а
  перестановка ключей ухудшила бы диффы на ревью, а не
  улучшила. Для документа без комментариев *и без пустых
  строк* `format_str` совпадает с `emit_canonical` его разбора.

- **`ktav-fmt` — необязательный форматтер командной строки**, за
  feature-флагом `cli`. До этого выпуска репозиторий не поставлял ни
  одного исполняемого файла.

  ```text
  cargo install ktav --features cli

  ktav-fmt <file>...           format each file in place
  ktav-fmt --stdout <file>     print the result, leave the file alone
  ktav-fmt --check <file>...   exit non-zero if a file is not formatted
  ktav-fmt -                   read one document from stdin
  ```

>>>>> lang=zh
- **`format_str` —— 保留注释的格式化器。** 它把文档的结构写法规范到
  `emit_canonical` 的产物,并穿插保留规范写入器会丢弃的附属内容。每条注释都逐字保留。空行作为分组提示保留,但连续
  两行及以上会折叠为一行,紧贴括号内侧的空行填充会被丢弃 —— 正是这一点
  让该变换成为不动点。键序永不改变:规范形式没有排序规则(§ 5.9),
  而重排键只会让评审差异更糟,而非更好。对于既无注释*也无空行*的文档,`format_str`
  等于其解析结果的 `emit_canonical`。

- **`ktav-fmt` —— 可选的命令行格式化器**,位于 `cli` feature 之后。
  在此版本之前,本仓库不提供任何可执行文件。

  ```text
  cargo install ktav --features cli

  ktav-fmt <file>...           format each file in place
  ktav-fmt --stdout <file>     print the result, leave the file alone
  ktav-fmt --check <file>...   exit non-zero if a file is not formatted
  ktav-fmt -                   read one document from stdin
  ```

