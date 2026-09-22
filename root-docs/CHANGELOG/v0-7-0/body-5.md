>>>>> lang=en
- Inline compound boundaries are computed once and reused through an
  `InlineBounds` memo; the three inline scanners are one shared state
  machine; `ScopeFrame` packs into a single byte.
- The thin path scans inline compounds straight into events instead of
  building an owned `Value` detour, stages inline arrays as flat event
  blocks, and registers child paths in one pass through a shared
  path-node index.
- Decoders borrow when there is nothing to unescape.
- The § 5.6 prefix scan is taken lazily over an iterator, and the
  canonical multi-line body is no longer split and rejoined.
- Map key names are written straight into the inline-capable `Scalar`,
  including through `collect_str` (`Display` and `fmt::Arguments` keys);
  a short key name no longer allocates a temporary `String`. On the read
  side an inline key is handed over as a slice, while an existing heap
  buffer is still moved into the target rather than copied.
- The i64-domain check for integers is a native range comparison instead
  of formatting and re-parsing the decimal text.

### Notes

- MSRV stays `1.71`. Dependencies are unchanged.
- Seventeen rounds of independent implementation review against spec
  0.7.0 are archived under `docs/reviews/`; every finding is either
  fixed here or explicitly recorded there as a profiling candidate.
- The published tarball no longer carries the vendored spec
  corpus, the internal review notes or the documentation generator:
  1,715 files (1.4 MiB compressed) down to 188 (403 KiB). Nothing that
  the library needs to build was removed — run the conformance suite
  from a git checkout, where nothing is excluded.

>>>>> lang=ru
- Границы inline-компаундов вычисляются один раз и переиспользуются
  через memo `InlineBounds`; три inline-сканера сведены в один общий
  автомат; `ScopeFrame` упакован в один байт.
- Thin-путь сканирует inline-компаунды сразу в события, без обходного
  построения owned `Value`, складывает inline-массивы плоскими блоками
  событий и регистрирует дочерние пути за один проход через общий
  индекс path-узлов.
- Декодеры заимствуют, когда снимать экранирование нечего.
- Скан префикса § 5.6 берётся лениво, поверх итератора, а канонический
  многострочный body больше не разбивается и не сшивается обратно.
- Имена ключей maps пишутся прямо в inline-способный `Scalar`, в том
  числе через `collect_str` (ключи `Display` и `fmt::Arguments`);
  короткое имя ключа больше не выделяет временный `String`. На стороне
  чтения inline-ключ отдаётся срезом, а уже существующий heap-буфер
  по-прежнему передаётся владением, а не копируется.
- Проверка i64-домена для целых — нативное сравнение диапазона вместо
  форматирования и повторного разбора десятичного текста.

### Заметки

- MSRV остаётся `1.71`. Зависимости не изменились.
- Семнадцать раундов независимого ревью реализации против спеки 0.7.0
  заархивированы в `docs/reviews/`; каждая находка либо исправлена
  здесь, либо явно зафиксирована там как кандидат для профилирования.
- Публикуемый tarball больше не тащит вендоренный корпус спеки,
  внутренние отчёты ревью и генератор документации: 1 715 файлов
  (1,4 MiB в сжатом виде) превратились в 188 (403 KiB). Ничего из
  того, что нужно для сборки библиотеки, не убрано — conformance-сюиту
  гоняйте из git-чекаута, где не исключено ничего.

>>>>> lang=zh
- inline 复合值边界只计算一次,并通过 `InlineBounds` memo 复用;三个
  inline 扫描器合为一个共享状态机;`ScopeFrame` 压缩进单个字节。
- thin 路径直接把 inline 复合值扫描为事件,不再绕行构建 owned `Value`,
  把 inline 数组以扁平事件块暂存,并通过共享的 path 节点索引一次性注册
  子路径。
- 无需去转义时解码器采用借用。
- § 5.6 前缀扫描改为在迭代器上惰性进行,canonical 多行 body 不再拆分后
  再拼接。
- map 键名直接写入可 inline 的 `Scalar`,包括经由 `collect_str`
  (`Display` 与 `fmt::Arguments` 键);短键名不再分配临时 `String`。读取
  侧 inline 键以切片交付,而已有的 heap 缓冲区仍以移动而非复制交给目标。
- 整数的 i64 域检查改为原生范围比较,不再格式化后重新解析十进制文本。

### 备注

- MSRV 仍为 `1.71`。依赖未变。
- 针对 spec 0.7.0 的十七轮独立实现评审归档于 `docs/reviews/`;每条发现要么
  已在此修复,要么在那里明确记录为待剖析的候选项。
- 发布的 tarball 不再携带随仓库附带的规范语料、内部评审记录与文档
  生成器:文件数从 1,715(压缩后 1.4 MiB)降至 188(403 KiB)。构建库
  所需的内容一个都没少——conformance 套件请在 git 检出中运行,那里不
  排除任何文件。

