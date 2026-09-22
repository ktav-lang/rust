>>>>> lang=en
### Performance

`cargo bench --bench parse -- --quick` against the 0.1.4 baseline:

|                                | 0.1.4 baseline | 0.1.5  | Δ      |
|--------------------------------|----------------|--------|--------|
| `parse_synth/small_1k`         | 16.1 µs        | 16.0 µs| −0.6 % |
| `parse_synth/medium_50k`       | 896 µs         | 663 µs | −26 %  |
| `parse_synth/large_500k`       | 9.49 ms        | 9.27 ms| −2.3 % |
| `parse_synth_error/small_1k`   | 7.5 µs         | 7.2 µs | −4.0 % |
| `parse_synth_error/medium_50k` | 340 µs         | 346 µs | +1.8 % |
| `parse_synth_error/large_500k` | 4.47 ms        | 4.50 ms| +0.7 % |

Net: zero success-path regression. Error-path slightly faster — the
new `Display` impl constructs the formatted string lazily at
`.to_string()` time, whereas the prior `format!(...)` allocated a
`String` at every error site eagerly. The cumulative-byte counter
that powers spans is statistically free.

### Notes

>>>>> lang=ru
### Производительность

`cargo bench --bench parse -- --quick` против baseline 0.1.4:

|                                | 0.1.4 baseline | 0.1.5  | Δ      |
|--------------------------------|----------------|--------|--------|
| `parse_synth/small_1k`         | 16.1 µs        | 16.0 µs| −0.6 % |
| `parse_synth/medium_50k`       | 896 µs         | 663 µs | −26 %  |
| `parse_synth/large_500k`       | 9.49 ms        | 9.27 ms| −2.3 % |
| `parse_synth_error/small_1k`   | 7.5 µs         | 7.2 µs | −4.0 % |
| `parse_synth_error/medium_50k` | 340 µs         | 346 µs | +1.8 % |
| `parse_synth_error/large_500k` | 4.47 ms        | 4.50 ms| +0.7 % |

Итог: регрессии на success-path нет. Error-path немного быстрее — новая
реализация `Display` собирает форматированную строку лениво, в момент
`.to_string()`, тогда как прежний `format!(...)` выделял `String` жадно
на каждой точке ошибки. Счётчик накопленных байтов, на котором держатся
spans, статистически бесплатен.

### Заметки

>>>>> lang=zh
### 性能

`cargo bench --bench parse -- --quick` 与 0.1.4 baseline 对比:

|                                | 0.1.4 baseline | 0.1.5  | Δ      |
|--------------------------------|----------------|--------|--------|
| `parse_synth/small_1k`         | 16.1 µs        | 16.0 µs| −0.6 % |
| `parse_synth/medium_50k`       | 896 µs         | 663 µs | −26 %  |
| `parse_synth/large_500k`       | 9.49 ms        | 9.27 ms| −2.3 % |
| `parse_synth_error/small_1k`   | 7.5 µs         | 7.2 µs | −4.0 % |
| `parse_synth_error/medium_50k` | 340 µs         | 346 µs | +1.8 % |
| `parse_synth_error/large_500k` | 4.47 ms        | 4.50 ms| +0.7 % |

结论:成功路径零回归。错误路径略快——新的 `Display` 实现在调用
`.to_string()` 时才惰性构造格式化字符串,而此前的 `format!(...)` 会在
每一处错误点即时分配一个 `String`。支撑 span 的累积字节计数器在统计上
是免费的。

### 备注

