>>>>> lang=en
  Three flags, parsed by hand: a CLI argument crate is the one place a
  new runtime dependency might have been defensible, and with three
  flags it is not.

  It is off by default deliberately. This repository publishes to
  crates.io and attaches no prebuilt binaries, so an unconditional bin
  target would commit the default install surface to a tool only
  Rust-toolchain owners can reach — while every binding exposes the same
  formatter natively and editors go through `ktav-lsp`.

### Fixed

- **Root-kind detection used a narrower whitespace set than § 3.3.** A
  first-line pair whose `:` separator was followed by a § 3.3
  whitespace code point other than space or tab — U+00A0, for
  instance — was not recognised as opening an Object root. § 3.3 freezes
  twenty-five code points and states outright that there is no separate,
  narrower "structural" whitespace concept; the predicate used two. One
  predicate feeds all three parsers, so the fix reaches every one of
  them.

- **Raw key validation measured escape sequences as two bytes.**
  `\uXXXX` (§ 3.7.1) is six bytes and a surrogate pair twelve;
  `is_valid_key` and `check_quoted_key` now advance by the real
  escape length instead of a blind `i += 2`. Observable behavior is
  unchanged — `u` and the hex digits are never structural bytes — but
  the correctness no longer rests on that coincidence.

>>>>> lang=ru
  Три флага, разобранные вручную: парсер аргументов — то единственное
  место, где новая рантайм-зависимость могла бы быть оправдана, и при
  трёх флагах она не оправдана.

  Выключен по умолчанию намеренно. Репозиторий публикуется в crates.io и
  не прикладывает готовых бинарей, так что безусловная bin-цель связала
  бы поверхность установки по умолчанию инструментом, доступным только
  владельцам Rust-тулчейна, — тогда как тот же форматтер есть нативно в
  каждом биндинге, а редакторы ходят через `ktav-lsp`.

### Fixed

- **Определение вида корня использовало более узкий набор пробелов,
  чем § 3.3.** Пара в первой строке, у которой после разделителя
  `:` шёл пробельный код-пойнт из § 3.3, отличный от space и tab —
  например U+00A0, — не распознавалась как открывающая объектный
  корень. § 3.3 фиксирует двадцать пять код-пойнтов и прямо
  заявляет, что отдельного, более узкого «структурного» понятия
  пробела не существует; предикат использовал два. Предикат один на все
  три парсера, поэтому исправление доходит до каждого.

- **Валидация сырых ключей считала escape-последовательность
  двухбайтовой.** `\uXXXX` (§ 3.7.1) занимает шесть байт, а
  суррогатная пара — двенадцать; `is_valid_key` и
  `check_quoted_key` теперь сдвигаются на настоящую длину escape, а
  не вслепую на `i += 2`. Наблюдаемое поведение не изменилось — `u` и
  шестнадцатеричные цифры никогда не бывают структурными байтами, — но
  корректность больше не держится на этом совпадении.

>>>>> lang=zh
  三个选项,手工解析:命令行参数库是唯一一处新增运行时依赖尚可辩护的
  地方,而只有三个选项时并不成立。

  默认关闭是刻意的。本仓库只发布到 crates.io,不附带预编译二进制,
  因此无条件的 bin 目标会把默认安装面绑定到一个只有 Rust 工具链持有者
  才能取用的工具 —— 而每个绑定都原生提供同一个格式化器,编辑器则经由
  `ktav-lsp`。

### 修复

- **根类型判定使用了比 § 3.3 更窄的空白集合。** 首行键值对的 `:`
  分隔符之后若跟随 space 与 tab 以外的 § 3.3 空白码位 —— 例如
  U+00A0 —— 不会被识别为开启对象根。§ 3.3 冻结了二十五个码位,并明确指出不存在另一套更窄的
  「结构性」空白概念;而该谓词只用了两个。这一谓词为全部三个解析器
  所共用,因此修复对每一个都生效。

- **原始键校验把转义序列当作两个字节。** `\uXXXX`(§ 3.7.1)是
  六个字节,代理对则是十二个;`is_valid_key` 与 `check_quoted_key`
  现在按转义的真实长度前进,而不是盲目地 `i += 2`。可观察行为未变 —— `u` 与十六进制数字从不是结构性
  字节 —— 但正确性不再依赖于这一巧合。

