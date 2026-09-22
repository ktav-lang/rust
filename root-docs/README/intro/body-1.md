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

**Specification:** this crate implements **Ktav 0.8.0**, the version
named by `[package.metadata.ktav] spec-version` in `Cargo.toml`. The
format is versioned and maintained independently of this crate — the two
numbers move apart on purpose, since a crate release that changes no
format behaviour leaves `spec-version` where it was. See
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) for the formal
document, and [`CHANGELOG.md`](CHANGELOG.md) for this crate's history.

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

**Спецификация:** этот crate реализует **Ktav 0.8.0** — версию,
указанную в `[package.metadata.ktav] spec-version` в `Cargo.toml`.
Формат версионируется и поддерживается независимо от crate-а, и два
номера намеренно расходятся: выпуск crate-а, не меняющий поведение
формата, оставляет `spec-version` на месте. См.
[`ktav-lang/spec`](https://github.com/ktav-lang/spec) для канонического
документа и [`CHANGELOG.ru.md`](CHANGELOG.ru.md) — историю crate-а.

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

**规范:** 本 crate 实现 **Ktav 0.8.0**,即 `Cargo.toml` 中
`[package.metadata.ktav] spec-version` 所指定的版本。格式与 crate 彼此
独立地版本化与维护,两个号码会有意分开——不改变格式行为的 crate 发布会
让 `spec-version` 保持原样。规范正文见
[`ktav-lang/spec`](https://github.com/ktav-lang/spec),crate 的历史见
[`CHANGELOG.zh.md`](CHANGELOG.zh.md)。

---

