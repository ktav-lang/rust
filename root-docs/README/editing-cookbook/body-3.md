>>>>> lang=en
#### 4. A numeric-looking String stays a String

The replacement is a String, not an Integer. The raw marker preserves that distinction. Supply a typed Integer when a numeric value is intended.

```rust
let source = "port: 8080\n";
let updated = edit_value(source, "port", &Value::String("9090".into()))?;
```

Result:

```ktav
port:: 9090
```

#### 5. Multiline replacement: expand only what is necessary

The changed array expands with four-space indentation. The untouched object stays inline, and verbatim string contents are not reindented.

```rust
let source = "a: [old, {keep: yes}]\n";
let updated = edit_value(source, "a.0", &Value::String("x\ny".into()))?;
```

Result:

>>>>> lang=ru
#### 4. Похожая на число строка остаётся строкой

Замена имеет тип String, а не Integer. Raw-маркер сохраняет это различие. Для числового значения передавайте типизированный Integer.

```rust
let source = "port: 8080\n";
let updated = edit_value(source, "port", &Value::String("9090".into()))?;
```

Результат:

```ktav
port:: 9090
```

#### 5. Многострочная замена: раскрывается только необходимое

Изменённый массив раскрывается с отступом четыре пробела. Нетронутый объект остаётся inline, содержимое verbatim-строки не переотступается.

```rust
let source = "a: [old, {keep: yes}]\n";
let updated = edit_value(source, "a.0", &Value::String("x\ny".into()))?;
```

Результат:

>>>>> lang=zh
#### 4. 看似数字的字符串仍然是字符串

替换值是 String，而不是 Integer。raw 标记保留这种区别；需要数值时请传入类型化 Integer。

```rust
let source = "port: 8080\n";
let updated = edit_value(source, "port", &Value::String("9090".into()))?;
```

结果:

```ktav
port:: 9090
```

#### 5. 多行替换：仅展开必要的容器

修改的数组以四空格缩进展开。未修改的对象保持行内形式，verbatim 字符串内容不会重新缩进。

```rust
let source = "a: [old, {keep: yes}]\n";
let updated = edit_value(source, "a.0", &Value::String("x\ny".into()))?;
```

结果:

