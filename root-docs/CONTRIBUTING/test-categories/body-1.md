>>>>> lang=en
Test categories:

- `src/**/tests.rs` — private unit tests per module.
- `tests/de/*` — deserialization by feature.
- `tests/ser/*` — serialization by feature.
- `tests/roundtrip/*` — round-trip (`T → text → T`).
- `tests/edge_cases/*` — combinatorial edge cases (paren literals,
  keywords in maps, deep nesting, special strings…).
- `tests/fixtures.rs` — end-to-end against real `.conf` files.
- `tests/spec_conformance.rs` — language-agnostic suite from
  `ktav-lang/spec` (valid fixtures match JSON oracle; invalid fixtures
  are rejected; valid fixtures survive lossless round-trip).

>>>>> lang=ru
Категории тестов:

- `src/**/tests.rs` — приватные unit-тесты на модуль.
- `tests/de/*` — десериализация по фичам.
- `tests/ser/*` — сериализация по фичам.
- `tests/roundtrip/*` — round-trip (`T → text → T`).
- `tests/edge_cases/*` — комбинаторные edge case-ы (литералы со
  скобками, ключевые слова в map-ах, глубокая вложенность, особые
  строки…).
- `tests/fixtures.rs` — end-to-end по реальным `.conf` файлам.
- `tests/spec_conformance.rs` — языково-нейтральный набор из
  `ktav-lang/spec` (valid-фикстуры совпадают с JSON-оракулом;
  invalid-фикстуры отклоняются; valid-фикстуры переживают round-trip).

>>>>> lang=zh
测试分类:

- `src/**/tests.rs` —— 模块内部的私有单元测试。
- `tests/de/*` —— 按特性组织的反序列化测试。
- `tests/ser/*` —— 按特性组织的序列化测试。
- `tests/roundtrip/*` —— round-trip(`T → text → T`)。
- `tests/edge_cases/*` —— 组合型边界用例(括号字面量、map 中的
  关键字、深度嵌套、特殊字符串……)。
- `tests/fixtures.rs` —— 针对真实 `.conf` 文件的端到端测试。
- `tests/spec_conformance.rs` —— 来自 `ktav-lang/spec` 的语言无关
  套件（valid 固件匹配 JSON 预期值；invalid 固件被拒绝；valid 固件
  经历无损 round-trip）。

