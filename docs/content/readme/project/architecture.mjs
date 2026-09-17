// README units: architecture section.
export const architecture = [
  {
    id: 'architecture-heading',
    join: 'block',
    en: `## Architecture`,
    ru: `## Архитектура`,
    zh: `## 架构`,
  },
  {
    id: 'architecture-tree',
    join: 'block',
    common: `\`\`\`
ktav/
├── value/            — the Value enum, ObjectMap
├── parser/           — line-by-line parser (text → Value)
├── thin/             — arena-backed borrowed parse (parse_events)
├── render/           — pretty-printer, canonical writer, formatter
├── ser/              — serde::Serializer (T: Serialize → Value)
├── de/               — serde::Deserializer (Value → T: Deserialize)
├── error/            — Error, ErrorKind, ErrorEnvelope, serde::Error
├── bin/ktav-fmt.rs   — the ktav-fmt command-line formatter
└── lib.rs            — glue: from_str / to_string / format_str / …
\`\`\``,
  },
  {
    id: 'architecture-file-conventions',
    join: 'block',
    en: `Each file holds one exported item; implementation details are private to
their parent module.`,
    ru: `В каждом файле — один экспортируемый элемент; детали реализации
приватны внутри родительского модуля.`,
    zh: `每个文件持有一个导出项;实现细节相对其父模块私有。`,
  },
];
