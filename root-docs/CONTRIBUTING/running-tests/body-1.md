>>>>> lang=en
## Running tests

```
cargo test                         # all tests (includes spec conformance)
cargo test --test spec_conformance # language-agnostic conformance suite only
cargo test --test edge_cases       # one category
cargo test multiline               # by name filter
cargo test --doc                   # doc-tests only
```

>>>>> lang=ru
## Запуск тестов

```
cargo test                         # все тесты (включая conformance)
cargo test --test spec_conformance # только языково-нейтральный набор
cargo test --test edge_cases       # одна категория
cargo test multiline               # по фильтру имени
cargo test --doc                   # doc-тесты
```

>>>>> lang=zh
## 运行测试

```
cargo test                         # 全部测试（含规范一致性）
cargo test --test spec_conformance # 仅语言无关的一致性套件
cargo test --test edge_cases       # 单个分类
cargo test multiline               # 按名称过滤
cargo test --doc                   # 文档测试
```

