>>>>> lang=en
## Pre-push local checks (mandatory)

Run **all five** of these locally before `git push`. CI runs the same
commands and a failure mid-pipeline costs more than a few seconds at
the keyboard — and in the case of a tag push that triggers a release,
a fix-and-force-tag-move dance.

```
cargo fmt --all -- --check                                         # canonical formatting
cargo clippy --release --all-features --all-targets -- -D warnings # zero warnings policy
cargo test --release --all-features                                # full suite, release mode
cargo build --release --all-features                               # final binary build
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps      # docs.rs build, zero warnings
```

If any of these are red, the push is **not** ready. The CI in this
repo enforces all five (see `.github/workflows/ci.yml`); pushing a
broken commit guarantees a red main badge and, for tag pushes,
breaks the release pipeline.

This rule applies repo-wide, not just to this crate — every
`ktav-lang/*` binding has its own equivalent local-check incantation
documented in its own `CONTRIBUTING.md`.

>>>>> lang=ru
## Обязательные локальные проверки перед push

Прогоните **все пять** локально перед `git push`. CI выполняет те же
команды, а падение посреди пайплайна стоит дороже нескольких секунд
за клавиатурой — а в случае пуша тега, запускающего релиз, это ещё и
танец «исправить и передвинуть тег force-пушем».

```
cargo fmt --all -- --check                                         # каноническое форматирование
cargo clippy --release --all-features --all-targets -- -D warnings # политика нулевых warning'ов
cargo test --release --all-features                                # полный набор, release-режим
cargo build --release --all-features                               # финальная сборка бинарника
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps      # сборка docs.rs, нулевые warning'и
```

Если хоть одна из них красная, push **не готов**. CI этого
репозитория проверяет все пять (см. `.github/workflows/ci.yml`);
пуш сломанного коммита гарантирует красный бейдж на main, а для
пуша тега — ломает релизный пайплайн.

Это правило действует по всему репозиторию, а не только для этого
крейта — у каждого биндинга `ktav-lang/*` есть свой эквивалент
локальных проверок, описанный в его собственном `CONTRIBUTING.md`.

>>>>> lang=zh
## 推送前的本地检查(强制)

在 `git push` 之前,在本地运行**全部五项**检查。CI 运行相同的命令,
而流水线中途的失败比在键盘前多花几秒钟的代价更大——如果是触发
发布的 tag 推送,还会引出一套「修复后强制移动 tag」的折腾。

```
cargo fmt --all -- --check                                         # 规范格式化
cargo clippy --release --all-features --all-targets -- -D warnings # 零警告策略
cargo test --release --all-features                                # release 模式下的完整套件
cargo build --release --all-features                               # 最终二进制构建
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps      # docs.rs 构建,零警告
```

只要有一项是红的,这次推送就**没有准备好**。本仓库的 CI 会强制
执行全部五项(见 `.github/workflows/ci.yml`);推送一个有问题的
提交必然导致 main 徽章变红,若是 tag 推送,还会破坏发布流水线。

这条规则适用于整个仓库,不仅限于本 crate——每个 `ktav-lang/*`
绑定都在自己的 `CONTRIBUTING.md` 中记录了等价的本地检查咒语。

