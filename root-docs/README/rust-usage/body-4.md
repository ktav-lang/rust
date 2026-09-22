>>>>> lang=en
Four public entry points: [`from_str`](https://docs.rs/ktav) /
[`from_file`](https://docs.rs/ktav) for reading, [`to_string`](https://docs.rs/ktav) /
[`to_file`](https://docs.rs/ktav) for writing. A complete runnable
example lives in [`examples/basic.rs`](examples/basic.rs).

### Inspect errors — structured variants for tooling

`Error::Structured(ErrorKind)` carries a typed category plus a
byte-offset `Span` for every parse failure, so editors and linters
can highlight the exact offending range instead of the whole line.

```rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\nport: 443\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("line {line}: duplicate key {key:?}");
        println!("offending bytes: {:?}", span.slice(src));   // -> Some("port")
        let (l, c) = span.line_col(src);                      // 1-based / 0-based byte col
        println!("highlight at {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
```

>>>>> lang=ru
Четыре публичных entry point-а: [`from_str`](https://docs.rs/ktav) /
[`from_file`](https://docs.rs/ktav) — для чтения,
[`to_string`](https://docs.rs/ktav) / [`to_file`](https://docs.rs/ktav) —
для записи. Полный запускаемый пример — в
[`examples/basic.rs`](examples/basic.rs).

### Анализ ошибок — структурированные варианты для тулинга

`Error::Structured(ErrorKind)` несёт типизированную категорию плюс
byte-offset `Span` для каждого parse-фейла, так что редакторы и
линтеры могут подсветить именно offending диапазон, а не всю строку.

```rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\nport: 443\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("строка {line}: дубликат ключа {key:?}");
        println!("offending байты: {:?}", span.slice(src));    // -> Some("port")
        let (l, c) = span.line_col(src);                       // line 1-based, col 0-based по байтам
        println!("подсветить в {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
```

>>>>> lang=zh
四个公共入口:读取用 [`from_str`](https://docs.rs/ktav) /
[`from_file`](https://docs.rs/ktav),写入用
[`to_string`](https://docs.rs/ktav) / [`to_file`](https://docs.rs/ktav)。
完整可运行示例:[`examples/basic.rs`](examples/basic.rs)。

### 检视错误 —— 给工具链使用的结构化变体

`Error::Structured(ErrorKind)` 为每次解析失败携带类型化的类别加字节
偏移 `Span`,因此编辑器和 linter 可以高亮恰好出错的范围,而不是
整行。

```rust
use ktav::{parse, Error, ErrorKind};

let src = "port: 80\nport: 443\n";
match parse(src) {
    Ok(_) => unreachable!(),
    Err(Error::Structured(ErrorKind::DuplicateKey { line, key, span, .. })) => {
        println!("第 {line} 行:重复键 {key:?}");
        println!("出错字节: {:?}", span.slice(src));   // -> Some("port")
        let (l, c) = span.line_col(src);                // 行 1 起,列 0 起按字节
        println!("高亮 {l}:{c}");
    }
    Err(other) => panic!("{other}"),
}
```

