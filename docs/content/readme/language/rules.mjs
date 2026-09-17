// README units: rules section.
export const rules = [
  {
    id: 'rules-heading',
    join: 'block',
    en: `## The rules`,
    ru: `## Правила`,
    zh: `## 规则`,
  },
  {
    id: 'rules-intro-1',
    join: 'block',
    en: `A Ktav document is an implicit top-level object.`,
    ru: `Документ Ktav — это неявный объект верхнего уровня.`,
    zh: `Ktav 文档是一个隐式的顶层对象。`,
  },
  {
    id: 'rules-intro-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Inside any object you
have pairs; inside any array you have items.`,
    ru: `Внутри любого
объекта — пары; внутри любого массива — элементы.`,
    zh: `任何对象里是键值对,任何数组里是
元素。`,
  },
  {
    id: 'rules-reference-card',
    join: 'block',
    common: `\`\`\`text
## comment              — any line starting with '##'
key: value             — scalar pair; key may be a dotted path (a.b.c)
key:: value            — scalar pair; value is ALWAYS a literal string
key: { ... }           — multi-line object; \`}\` closes on its own line
key: [ ... ]           — multi-line array; \`]\` closes on its own line
key: {}   /   key: []  — empty compound, inline
key: ( ... )           — multi-line string; common indent stripped
key: (( ... ))         — multi-line string; verbatim (no stripping)
:: value               — inside an array: literal-string item
\`\`\``,
  },
  {
    id: 'rules-summary-1',
    join: 'block',
    en: `That's the whole language.`,
    ru: `Это весь язык.`,
    zh: `整个语言就这些。`,
  },
  {
    id: 'rules-summary-2',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `No commas, no quotes, no escape inside the
value itself — the only "escape" is the \`::\` marker, and it lives in the
separator (for pairs) or as a line prefix (for array items).`,
    ru: `Никаких запятых, никаких кавычек, никаких escape
внутри самого значения — единственный «escape» это маркер \`::\`, и он
живёт в разделителе (для пар) или в префиксе строки (для элементов
массива).`,
    zh: `没有逗号、没有引号、没有值内部的转义——唯一的
「转义」是 \`::\` 标记,它出现在分隔符里(用于键值对)或行首前缀里
(用于数组元素)。`,
  },
];
