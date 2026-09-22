>>>>> lang=en
## [0.2.0] — 2026-05-07

Minor release with two breaking output / validation changes:

### Changed (breaking)

- **Multi-line strings emit indented stripped form `( ... )` by default**,
  not verbatim `(( ... ))`. Verbatim is still produced as a fallback when
  the content has its own leading whitespace (which the parser-side
  dedent would clobber) or contains a sole-`)` line that would close the
  stripped form prematurely. Code that compares `to_string` /
  `render` output byte-for-byte to a baked-in `((...))` literal needs to
  be updated. Round-tripping (`parse(to_string(v)) == v`) is unchanged.

      // Before (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // After (0.2.0):
      // body: (
      //     line1
      //     line2
      // )

>>>>> lang=ru
## [0.2.0] — 2026-05-07

Минорный релиз с двумя breaking-изменениями вывода / валидации:

### Изменено (breaking)

- **Многострочные строки по умолчанию выводятся в форме stripped
  `( ... )`** с отступом, а не verbatim `(( ... ))`. Verbatim остаётся
  как fallback когда содержимое имеет собственный leading-whitespace
  (parser-side dedent его съест) или строку, тримящуюся в `)` (закрыла
  бы stripped преждевременно). Код, сравнивающий вывод `to_string` /
  `render` побайтово с фиксированным `((...))`, нужно обновить.
  Round-trip (`parse(to_string(v)) == v`) не изменился.

      // Было (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // Стало (0.2.0):
      // body: (
      //     line1
      //     line2
      // )

>>>>> lang=zh
## [0.2.0] —— 2026-05-07

次要发布,带两项 breaking 输出 / 校验改动:

### 变更(breaking)

- **多行字符串默认输出为缩进 stripped 形式 `( ... )`**,而非 verbatim
  `(( ... ))`。当内容自带前导空白(会被解析侧的 dedent 吞掉)或包含
  仅为 `)` 的行(会提前关闭 stripped)时,fallback 到 verbatim。逐字节
  比较 `to_string` / `render` 输出与硬编码 `((...))` 的代码需要更新。
  Round-trip(`parse(to_string(v)) == v`)未受影响。

      // 之前 (0.1.5):
      // body: ((
      // line1
      // line2
      // ))
      //
      // 之后 (0.2.0):
      // body: (
      //     line1
      //     line2
      // )

