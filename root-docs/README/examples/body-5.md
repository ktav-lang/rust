>>>>> lang=en
```text
countries: [
    {
        name: Russia
        cities: [
            {
                name: Moscow
                buildings: [
                    {
                        name: Kremlin
                    }
                    {
                        name: Saint Basil's
                    }
                ]
            }
            {
                name: Saint Petersburg
            }
        ]
    }
    {
        name: France
    }
]
```

### 7. Literal strings: `::`

Some values would otherwise be parsed as compound (because they start
with `{` or `[`): regular expressions, IPv6 addresses, template
placeholders. The double-colon `::` flags them as "raw string, do not
parse further."

>>>>> lang=ru
```text
countries: [
    {
        name: Russia
        cities: [
            {
                name: Moscow
                buildings: [
                    {
                        name: Kremlin
                    }
                    {
                        name: Saint Basil's
                    }
                ]
            }
            {
                name: Saint Petersburg
            }
        ]
    }
    {
        name: France
    }
]
```

### 7. Литеральные строки: `::`

Некоторые значения иначе были бы разобраны как compound (потому что
начинаются с `{` или `[`): регулярные выражения, IPv6-адреса,
placeholders шаблонов. Двойное двоеточие `::` помечает их как «сырая
строка, не разбирать дальше».

>>>>> lang=zh
```text
countries: [
    {
        name: Russia
        cities: [
            {
                name: Moscow
                buildings: [
                    {
                        name: Kremlin
                    }
                    {
                        name: Saint Basil's
                    }
                ]
            }
            {
                name: Saint Petersburg
            }
        ]
    }
    {
        name: France
    }
]
```

### 7. 字面量字符串:`::`

某些值如果不加标记,会被当作复合值解析(因为以 `{` 或 `[` 开头):
正则、IPv6 地址、模板占位符。双冒号 `::` 把它们标记为「原样字符串,
不要继续解析」。

