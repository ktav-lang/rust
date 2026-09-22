>>>>> lang=en
## [0.1.2] — 2026-04-26

Re-publish of 0.1.1's contents with the source tree run through
`cargo fmt`. 0.1.1 was yanked because the new files (`benches/vs_json.rs`,
`src/thin/event*.rs`, `src/thin/fast_num.rs`) hadn't been formatted
through rustfmt before publish, which tripped the CI lint check on the
tag push. **Functionally identical to 0.1.1** — only whitespace differs.

>>>>> lang=ru
## [0.1.2] — 2026-04-26

Перевыпуск содержимого 0.1.1 после прогона `cargo fmt`. 0.1.1 был
отозван (yanked), потому что новые файлы (`benches/vs_json.rs`,
`src/thin/event*.rs`, `src/thin/fast_num.rs`) не были отформатированы
через rustfmt перед публикацией, что обвалило CI lint при пуше тега.
**Функционально идентично 0.1.1** — отличается только пробелами.

>>>>> lang=zh
## [0.1.2] —— 2026-04-26

0.1.1 内容的重新发布,源码经过 `cargo fmt` 处理。0.1.1 被 yank,因为
新增文件(`benches/vs_json.rs`, `src/thin/event*.rs`,
`src/thin/fast_num.rs`)在发布前未经 rustfmt 处理,导致 CI lint 在
tag push 时失败。**功能与 0.1.1 完全一致** —— 仅空白字符不同。

