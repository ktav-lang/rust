>>>>> lang=en
assert!(parse(src).is_ok());                 // Float(1.1) — trailing zero gone
assert!(parse("zip: 01234\n").is_ok());       // String("01234") — leading zero kept

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
    }
    other => panic!("expected LossyScalar, got {other:?}"),
}
```

Fix either by appending `::` to keep the value a String
(`zip:: 01234`) or by writing the canonical number. Any document
`parse_strict()` accepts yields exactly the same `Value` tree as
`parse()`, so strict mode is a validation gate, not a different
dialect. The serde path (`from_str`) has no strict variant yet.

### Stream parse — events without an intermediate tree

`parse_events` invokes a callback for each parse event, with strings
borrowed directly into the input buffer — no allocation per event, no
intermediate `Value` tree. Useful when you don't need the full document:
counting keys, streaming to another format, building a custom shape.

```rust
use ktav::{parse_events, ParseEvent};

>>>>> lang=ru
assert!(parse(src).is_ok());                 // Float(1.1) — замыкающий ноль потерян
assert!(parse("zip: 01234\n").is_ok());       // String("01234") — ведущий ноль сохранён

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
    }
    other => panic!("ожидался LossyScalar, получено {other:?}"),
}
```

Исправить можно либо дописав `::`, чтобы значение осталось строкой
(`zip:: 01234`), либо записав число в канонической форме. Любой
документ, принятый `parse_strict()`, даёт ровно то же дерево `Value`,
что и `parse()` — строгий режим это проверочный шлюз, а не другой
диалект. У serde-пути (`from_str`) строгого варианта пока нет.

### Stream-парсинг — события без промежуточного дерева

`parse_events` вызывает callback на каждое событие парсинга, со
строками заимствующимися напрямую из input-буфера — без аллокации на
событие, без промежуточного `Value`-дерева. Полезно когда полный
документ не нужен: подсчёт ключей, стриминг в другой формат, построение
custom-shape-а.

```rust
use ktav::{parse_events, ParseEvent};

>>>>> lang=zh
assert!(parse(src).is_ok());                 // Float(1.1) —— 尾部零丢失
assert!(parse("zip: 01234\n").is_ok());       // String("01234") —— 前导零被保留

match parse_strict(src) {
    Err(Error::Structured(ErrorKind::LossyScalar { body, canonical, .. })) => {
        assert_eq!((body.as_str(), canonical.as_str()), ("1.10", "1.1"));
    }
    other => panic!("期望 LossyScalar，实际为 {other:?}"),
}
```

修复方式有两种：追加 `::` 让该值保持字符串（`zip:: 01234`），或直接
写成规范数字。凡是 `parse_strict()` 接受的文档，其产生的 `Value` 树
与 `parse()` 完全相同 —— 严格模式是一道校验闸门，而非另一种方言。
serde 路径（`from_str`）目前尚无严格模式变体。

### 流式解析 —— 不构建中间树的事件流

`parse_events` 对每个解析事件调用回调,字符串直接从 input 缓冲区
借用 —— 不在每个事件上分配,也不构建中间 `Value` 树。在不需要完整
文档时很有用:统计键、流式转换到其他格式、构建自定义形状。

```rust
use ktav::{parse_events, ParseEvent};

