>>>>> lang=en
## Getting the code

The spec conformance suite lives in the `spec/` git submodule
([`ktav-lang/spec`](https://github.com/ktav-lang/spec)). Clone with
submodules so `cargo test` can run it:

```
git clone --recurse-submodules https://github.com/ktav-lang/rust
```

If you already cloned without `--recurse-submodules`:

```
git submodule update --init
```

>>>>> lang=ru
## Получение кода

Набор тестов соответствия спецификации находится в git-субмодуле `spec/`
([`ktav-lang/spec`](https://github.com/ktav-lang/spec)). Клонируйте
вместе с субмодулем, чтобы `cargo test` мог его запустить:

```
git clone --recurse-submodules https://github.com/ktav-lang/rust
```

Если репозиторий уже склонирован без `--recurse-submodules`:

```
git submodule update --init
```

>>>>> lang=zh
## 获取代码

规范一致性测试套件位于 git 子模块 `spec/`
([`ktav-lang/spec`](https://github.com/ktav-lang/spec))。克隆时带上
子模块，以确保 `cargo test` 能正常运行：

```
git clone --recurse-submodules https://github.com/ktav-lang/rust
```

若已在不带 `--recurse-submodules` 的情况下克隆：

```
git submodule update --init
```

