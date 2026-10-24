>>>>> lang=en
# Ktav (כְּתָב)

[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

> A plain configuration format. JSON5-shaped, but without the quotes,
> without the commas, with dotted keys for nesting. Native `serde`
> integration.

**Languages:** **English** · [Русский](README.ru.md) · [简体中文](README.zh.md)

**Playground:** convert JSON / YAML / TOML / INI ⇄ Ktav in your browser at **[ktav-lang.github.io](https://ktav-lang.github.io/)**.

**Version in preparation: 0.9.0, not published.** This branch combines
lossless source editing by path and the new source-error location model.
`Cargo.toml` targets the draft spec corpus `versions/0.9/tests`; published
0.8.1 remains unchanged. See [`ktav-lang/spec`](https://github.com/ktav-lang/spec)
for the specification and [`CHANGELOG.md`](CHANGELOG.md) for the migration.

---

>>>>> lang=ru
# Ktav (כְּתָב)

[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

> Простой формат конфигурации. Формы JSON5, но без кавычек, без
> запятых, с точечными ключами для вложенности. Нативная интеграция
> с `serde`.

**Languages:** [English](README.md) · **Русский** · [简体中文](README.zh.md)

**Песочница:** конвертация JSON / YAML / TOML / INI ⇄ Ktav прямо в браузере — **[ktav-lang.github.io](https://ktav-lang.github.io/)**.

**Готовится версия 0.9.0, ещё не опубликована.** Эта ветка объединяет
lossless-редактирование исходного текста по пути и новую модель координат
ошибок. `Cargo.toml` указывает на draft corpus спеки `versions/0.9/tests`;
опубликованная 0.8.1 не меняется. Спецификация —
[`ktav-lang/spec`](https://github.com/ktav-lang/spec), миграция —
[`CHANGELOG.ru.md`](CHANGELOG.ru.md).

---

>>>>> lang=zh
# Ktav (כְּתָב)

[![Crates.io](https://img.shields.io/crates/v/ktav?style=flat-square&logo=rust&label=crates.io)](https://crates.io/crates/ktav)
[![docs.rs](https://img.shields.io/docsrs/ktav?style=flat-square&label=docs.rs)](https://docs.rs/ktav)
[![CI](https://img.shields.io/github/actions/workflow/status/ktav-lang/rust/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/ktav-lang/rust/actions)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)
[![Playground](https://img.shields.io/badge/playground-try%20online-7c3aed?style=flat-square&logo=rocket&logoColor=white)](https://ktav-lang.github.io/)

> 一种朴素的配置格式。形态上接近 JSON5,但不带引号、不用逗号,
> 以点分键表达嵌套。原生 `serde` 集成。

**Languages:** [English](README.md) · [Русский](README.ru.md) · **简体中文**

**演练场：** 在浏览器中互转 JSON / YAML / TOML / INI ⇄ Ktav — **[ktav-lang.github.io](https://ktav-lang.github.io/)**。

**0.9.0 正在准备中，尚未发布。** 本分支同时包含按路径无损编辑源文本
及新的源错误坐标模型。`Cargo.toml` 指向规范草案语料 `versions/0.9/tests`；
已发布的 0.8.1 保持不变。规范见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec)，迁移说明见
[`CHANGELOG.zh.md`](CHANGELOG.zh.md)。

---

