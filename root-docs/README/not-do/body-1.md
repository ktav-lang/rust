>>>>> lang=en
## What Ktav does NOT do — and never will

- **Inline non-empty compounds** like `x: { a: 1, b: 2 }`. They'd bring
  commas, and commas would bring escaping. Compound values are
  multiline.
- **Anchors / aliases / merge keys** (`&anchor`, `*ref`, `<<:`). Any
  line whose meaning depends on a declaration far away stops being
  self-sufficient. If you want DRY, compose defaults in code.
- **File includes** (`@include`, `!import`). Write a wrapper in code
  for large configs.
- **Top-level arrays.** The document is always an object.

>>>>> lang=ru
## Чего Ktav НЕ делает — и никогда не будет

- **Inline непустые compound-ы** вроде `x: { a: 1, b: 2 }`. Они
  притащили бы запятые, а запятые притащили бы escape. Compound-
  значения многострочны.
- **Якоря / алиасы / merge-ключи** (`&anchor`, `*ref`, `<<:`). Любая
  строка, смысл которой зависит от декларации в отдалённом месте,
  перестаёт быть самодостаточной. Если нужен DRY — композируйте
  defaults в коде.
- **Инклюды файлов** (`@include`, `!import`). Для больших конфигов
  напишите обёртку в коде.
- **Массивы на верхнем уровне.** Документ всегда — объект.

>>>>> lang=zh
## Ktav **不**做、也永远不会做的事

- **内联的非空复合值**,比如 `x: { a: 1, b: 2 }`。它们会带来逗号,
  逗号又会带来转义。复合值保持多行。
- **锚点 / 别名 / 合并键**(`&anchor`、`*ref`、`<<:`)。任何一行
  若其含义依赖远处的声明,就不再自洽。若需要 DRY,请在代码里
  组合默认值。
- **文件包含**(`@include`、`!import`)。大型配置请在代码里包一层
  封装。
- **顶层数组。** 文档始终是对象。

