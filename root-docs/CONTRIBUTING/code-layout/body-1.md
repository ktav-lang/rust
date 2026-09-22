>>>>> lang=en
## Code layout guide

Decomposition rule: **one exported item per file**. Private helpers
live with the type that uses them. Folders group closely-related
items (all of `src/thin/` is the zero-copy deserialization path).

```
src/
├── lib.rs                         public entry points
├── value/                         owned Value enum (public)
├── parser/                        Value-building parser
├── render/                        Value → text
├── thin/                          zero-copy de path (ThinValue → T)
├── ser/                           T → Value (public) + T → text (direct)
├── de/                            Value → T (via ValueDeserializer)
└── error/                         Error + serde impls
```

>>>>> lang=ru
## Руководство по раскладке кода

Правило декомпозиции: **один экспортируемый элемент на файл**.
Приватные хелперы живут рядом с типом, который ими пользуется.
Папки группируют тесно связанные элементы (весь `src/thin/` — это
zero-copy путь десериализации).

```
src/
├── lib.rs                         public entry points
├── value/                         owned Value enum (public)
├── parser/                        Value-building parser
├── render/                        Value → text
├── thin/                          zero-copy de path (ThinValue → T)
├── ser/                           T → Value (public) + T → text (direct)
├── de/                            Value → T (via ValueDeserializer)
└── error/                         Error + serde impls
```

>>>>> lang=zh
## 代码布局指南

分解规则:**每个文件一个导出项**。私有辅助函数与使用它的类型放
一起。目录把紧密相关的项归拢(`src/thin/` 全部属于零拷贝反序列化
路径)。

```
src/
├── lib.rs                         public entry points
├── value/                         owned Value enum (public)
├── parser/                        Value-building parser
├── render/                        Value → text
├── thin/                          zero-copy de path (ThinValue → T)
├── ser/                           T → Value (public) + T → text (direct)
├── de/                            Value → T (via ValueDeserializer)
└── error/                         Error + serde impls
```

