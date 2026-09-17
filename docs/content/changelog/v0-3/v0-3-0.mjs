// Changelog units for 0.3.0.
export const v0_3_0 = [
  {
    id: 'v0-3-0-001',
    join: 'block',
    en: `## [0.3.0] — 2026-05-08`,
    ru: `## [0.3.0] — 2026-05-08`,
    zh: `## [0.3.0] —— 2026-05-08`,
  },
  {
    id: 'v0-3-0-002',
    join: 'block',
    en: `Minor release with one breaking parser strictness change, a
diagnostic-range fix, and hot-path micro-optimisations on the
typed-deserialize path.`,
    ru: `Минорный релиз с одним ломающим ужесточением парсера, исправлением
диапазона диагностики и микрооптимизациями горячего пути
типизированной десериализации.`,
    zh: `次要发布，含一项破坏性的解析器严格化、一处诊断范围修复，以及类型化
反序列化热路径上的微优化。`,
  },
  {
    id: 'v0-3-0-003',
    join: 'block',
    en: `### Fixed`,
    ru: `### Исправлено`,
    zh: `### 修复`,
  },
  {
    id: 'v0-3-0-004',
    join: 'block',
    en: `- \`ErrorKind::DuplicateKey\` and \`ErrorKind::KeyPathConflict\` now carry
  the span of the **offending key**, not the closing \`}\` / \`]\` of the
  compound that would have been assigned to it.`,
    ru: `- \`ErrorKind::DuplicateKey\` и \`ErrorKind::KeyPathConflict\` теперь несут
  span **виновного ключа**, а не закрывающей \`}\` / \`]\` того
  составного, которое ему присваивалось.`,
    zh: `- \`ErrorKind::DuplicateKey\` 与 \`ErrorKind::KeyPathConflict\` 现在携带
  **出错键**自身的 span，而不是本应赋给它的那个复合结构的收尾
  \`}\` / \`]\`。`,
  },
  {
    id: 'v0-3-0-005',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Previously, when the
  conflict was detected on \`attach_child_value\` (e.g. \`value: { ... }\`
  duplicating an earlier \`value: ...\`), the saved span pointed at the
  closing brace because that's the position the parser had at hand at
  the moment of detection.`,
    ru: `Раньше, когда конфликт
  обнаруживался на \`attach_child_value\` (например, \`value: { ... }\`,
  дублирующее более раннее \`value: ...\`), сохранённый span указывал на
  закрывающую скобку — это была позиция, которая оказывалась у парсера
  под рукой в момент обнаружения.`,
    zh: `此前，当冲突在 \`attach_child_value\` 处被发现时
  （例如 \`value: { ... }\` 与更早的 \`value: ...\` 重复），保存的 span
  指向收尾括号——那是解析器在发现冲突的那一刻手头持有的位置。`,
  },
  {
    id: 'v0-3-0-006',
    join: 'block',
    en: `  The parser now stores the key's own span (\`pending_key_span\`) on the
  parent frame when a compound is opened, and reuses it when the
  compound closes and the value is attached.`,
    ru: `  Теперь парсер сохраняет собственный span ключа (\`pending_key_span\`)
  в родительском фрейме при открытии составного и переиспользует его,
  когда составное закрывается и значение присваивается.`,
    zh: `  现在解析器在打开复合结构时把键自身的 span（\`pending_key_span\`）
  存入父帧，并在复合结构收尾、值被挂接时复用它。`,
  },
  {
    id: 'v0-3-0-007',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Editors / IDEs that draw
  diagnostic underlines from \`Span\` now point at the key.`,
    ru: `Редакторы и
  IDE, рисующие диагностические подчёркивания по \`Span\`, теперь
  указывают на ключ.`,
    zh: `依据 \`Span\` 绘制
  诊断下划线的编辑器 / IDE 现在会指向键。`,
  },
  {
    id: 'v0-3-0-008',
    join: 'block',
    en: `  This is a fix for a span value, not an API change — \`ErrorKind\`
  shape is unchanged.`,
    ru: `  Это исправление значения span, а не изменение API — форма
  \`ErrorKind\` не менялась.`,
    zh: `  这是对 span 取值的修复，不是 API 变更 —— \`ErrorKind\` 的形态未变。`,
  },
  {
    id: 'v0-3-0-009',
    join: 'block',
    en: `### Changed (breaking — parser strictness)`,
    ru: `### Изменено (breaking — строгость парсера)`,
    zh: `### 变更（breaking —— 解析器严格化）`,
  },
  {
    id: 'v0-3-0-010',
    join: 'block',
    en: `- \`key: (value)\` and \`key: ((value))\` now error with
  \`ErrorKind::InlineNonEmptyCompound { body: "paren-string" }\`.`,
    ru: `- \`key: (value)\` и \`key: ((value))\` теперь дают ошибку
  \`ErrorKind::InlineNonEmptyCompound { body: "paren-string" }\`.`,
    zh: `- \`key: (value)\` 与 \`key: ((value))\` 现在以
  \`ErrorKind::InlineNonEmptyCompound { body: "paren-string" }\` 报错。`,
  },
  {
    id: 'v0-3-0-011',
    join: 'tight',
    en: `  These shapes used to be accepted as plain string scalars \`(value)\`,
  but they are visually indistinguishable from multi-line openers and
  would confuse readers.`,
    ru: `  Раньше эти формы принимались как обычные строковые скаляры
  \`(value)\`, но визуально они неотличимы от многострочных опенеров и
  сбивали бы читателя с толку.`,
    zh: `  这些形态过去被当作普通字符串标量 \`(value)\` 接受，但它们与多行开启符
  在视觉上无法区分，会让读者困惑。`,
  },
  {
    id: 'v0-3-0-012',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The raw-marker form \`key:: (value)\` remains
  valid and is the canonical way to encode such literals.`,
    ru: `Форма с raw-маркером \`key:: (value)\`
  остаётся валидной и является каноническим способом записать такой
  литерал.`,
    zh: `带 raw 标记的形式 \`key:: (value)\`
  仍然有效，并且是编码此类字面量的规范写法。`,
  },
  {
    id: 'v0-3-0-013',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The
  ktav-lsp formatter auto-rewrites the legacy form on save.`,
    ru: `Форматтер ktav-lsp автоматически переписывает legacy-форму
  при сохранении.`,
    zh: `ktav-lsp 的格式化器会在
  保存时自动改写旧形式。`,
  },
  {
    id: 'v0-3-0-014',
    join: 'block',
    en: `### Optimised (no API change)`,
    ru: `### Оптимизировано (без изменения API)`,
    zh: `### 优化（无 API 变更）`,
  },
  {
    id: 'v0-3-0-015',
    join: 'block',
    en: `- \`render::render\` pre-sizes the output \`String\` with a recursive
  \`estimate_size(value)\` to skip the doubling reallocations that
  \`push_str\` chains would otherwise trigger on multi-KiB outputs.`,
    ru: `- \`render::render\` преднастраивает размер выходного \`String\` через
  рекурсивный \`estimate_size(value)\`, чтобы обойти удваивающие
  реаллокации, которые цепочки \`push_str\` иначе вызывали бы на выводе
  в несколько KiB.`,
    zh: `- \`render::render\` 通过递归的 \`estimate_size(value)\` 预设输出 \`String\`
  的容量，从而跳过 \`push_str\` 链在数 KiB 输出上本会触发的倍增式
  重分配。`,
  },
  {
    id: 'v0-3-0-016',
    join: 'tight',
    en: `- \`EventCursor::peek\` / \`next\` use \`unsafe get_unchecked\` for
  bounds-elision on the hot path; the parser's well-formed-stream
  invariant guarantees \`pos < len()\` on every call.`,
    ru: `- \`EventCursor::peek\` / \`next\` используют \`unsafe get_unchecked\` для
  устранения проверки границ на горячем пути; инвариант
  well-formed-потока парсера гарантирует \`pos < len()\` на каждом
  вызове.`,
    zh: `- \`EventCursor::peek\` / \`next\` 使用 \`unsafe get_unchecked\` 以消除热
  路径上的边界检查；解析器的良构流不变式保证每次调用时
  \`pos < len()\`。`,
  },
  {
    id: 'v0-3-0-017',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Falls back to
  \`None\` when the invariant is violated, so malformed inputs remain
  safe — the unsafe path is a pure branch elision win.`,
    ru: `При нарушении инварианта возвращается \`None\`, поэтому
  некорректные входы остаются безопасными — unsafe-путь здесь чистый
  выигрыш на устранении ветвления.`,
    zh: `不变式被破坏时回退为 \`None\`，因此畸形输入依然安全
  —— 这里的 unsafe 路径纯粹是省去一次分支判断的收益。`,
  },
  {
    id: 'v0-3-0-018',
    join: 'tight',
    en: `- \`MapAccess::next_key_seed\` folds redundant \`peek + next\` into a
  single \`next\` (both branches consume the cursor anyway).`,
    ru: `- \`MapAccess::next_key_seed\` сворачивает избыточные \`peek + next\` в
  один \`next\` (обе ветки всё равно потребляют курсор).`,
    zh: `- \`MapAccess::next_key_seed\` 将冗余的 \`peek + next\` 合并为单次 \`next\`
  （两个分支反正都会消费游标）。`,
  },
  {
    id: 'v0-3-0-019',
    join: 'tight',
    en: `- Event \`BumpVec\` capacity hint raised from \`text.len() / 8 + 16\`
  to \`text.len() / 4 + 64\`.`,
    ru: `- Подсказка ёмкости event-\`BumpVec\` поднята с \`text.len() / 8 + 16\`
  до \`text.len() / 4 + 64\`.`,
    zh: `- 事件 \`BumpVec\` 的容量提示从 \`text.len() / 8 + 16\` 提高到
  \`text.len() / 4 + 64\`。`,
  },
  {
    id: 'v0-3-0-020',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The previous hint underestimated the
  ~1-event-per-5-bytes density on synth fixtures and triggered
  8–10 realloc-copy steps inside the bump arena on a 500 KiB doc.`,
    ru: `Прежняя подсказка недооценивала плотность
  ~1 событие на 5 байт на synth-фикстурах и вызывала 8–10 шагов
  realloc-copy внутри bump-арены на документе в 500 KiB.`,
    zh: `旧提示低估了 synth fixture 上约每 5 字节
  1 个事件的密度，在 500 KiB 文档上会在 bump arena 内触发 8–10 次
  realloc-copy。`,
  },
  {
    id: 'v0-3-0-021',
    join: 'block',
    en: `### Experiment (reverted)`,
    ru: `### Эксперимент (откачен)`,
    zh: `### 实验（已回退）`,
  },
  {
    id: 'v0-3-0-022',
    join: 'block',
    en: `- A streaming-deserializer refactor (parse on demand, no whole-doc
  \`Vec<Event>\`) was implemented and tested — all 404 tests passed,
  but parse_to_struct regressed 15–60 % vs. the existing cursor on
  this hardware.`,
    ru: `- Рефакторинг стримингового десериализатора (разбор по требованию, без
  \`Vec<Event>\` на весь документ) был реализован и протестирован — все
  404 теста прошли, но parse_to_struct регрессировал на 15–60 % против
  существующего курсора на этом железе.`,
    zh: `- 一次流式反序列化器重构（按需解析，不保留整份文档的 \`Vec<Event>\`）
  已实现并测试 —— 全部 404 个测试通过，但在本机硬件上
  parse_to_struct 相对既有游标回退了 15–60 %。`,
  },
  {
    id: 'v0-3-0-023',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `Cause: the cursor walks a contiguous slice with
  one monotonic branch the predictor nails 100 %, while streaming
  interleaves parser state-machine work with deserializer work and
  blows the predictor.`,
    ru: `Причина: курсор идёт по
  непрерывному срезу с одной монотонной ветвью, которую предсказатель
  угадывает на 100 %, тогда как стриминг перемежает работу
  конечного автомата парсера с работой десериализатора и сбивает
  предсказатель.`,
    zh: `原因：游标沿连续切片
  行进，只有一条单调分支，预测器命中率 100 %；而流式方案把解析器状态
  机的工作与反序列化器的工作交错，打乱了预测器。`,
  },
  {
    id: 'v0-3-0-024',
    join: { en: 'flow', ru: 'flow', zh: 'none' },
    en: `The streaming code was removed; the
  \`EventSink<'a>\` trait introduced for the experiment survives in
  \`event.rs\` as harmless generic infrastructure (zero cost when
  used only with \`BumpVec\`).`,
    ru: `Стриминговый код удалён; трейт \`EventSink<'a>\`,
  введённый для эксперимента, остался в \`event.rs\` как безвредная
  обобщённая инфраструктура (нулевая стоимость при использовании
  только с \`BumpVec\`).`,
    zh: `流式代码已删除；为该
  实验引入的 \`EventSink<'a>\` trait 作为无害的泛型基础设施保留在
  \`event.rs\` 中（仅与 \`BumpVec\` 搭配使用时零开销）。`,
  },
];
