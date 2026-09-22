>>>>> lang=en
## Releasing

A release is **fully automated** for this crate — push a `v*` tag and
the `Release` workflow handles everything:

```
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

That triggers three sequential jobs in `.github/workflows/release.yml`:

1. **`verify`** — re-runs the five pre-push checks above (fmt / clippy
   / test / build / doc) on the tagged commit. No green here, no
   publish.
2. **`publish`** — `cargo publish` via crates.io **Trusted Publishing**
   (OIDC): the workflow trades its short-lived GitHub identity for a
   crates.io token that expires within the run, gated to the
   `crates-io` environment. No long-lived registry token is stored in
   the repository. Confirms the tag matches `Cargo.toml` version
   before uploading.
3. **`github-release`** — creates a GitHub Release for the tag and
   pulls the matching `## [X.Y.Z]` section out of `CHANGELOG.md` for
   the release notes. So **the CHANGELOG entry IS the release notes**:
   write the changelog before tagging.

What this means in practice:

- **Always write the CHANGELOG entry before tagging.** Skipping it
  leaves the GH Release with empty notes. Edit the source under
  `root-docs/CHANGELOG/` and regenerate with
  `node scripts/build-docs.mjs` — never hand-edit `CHANGELOG.md`.
- **Never `cargo publish` from a maintainer's machine** — the
  workflow is the canonical path. (Manual publish would skip the
  verify gate and produce no GH Release.)
- If you need to retry a release without moving the tag, use
  `workflow_dispatch` with the tag name as the input.

>>>>> lang=ru
## Релиз

Релиз этого крейта **полностью автоматизирован** — запушьте тег `v*`,
и воркфлоу `Release` сделает всё сам:

```
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

Это запускает три последовательные job'ы в
`.github/workflows/release.yml`:

1. **`verify`** — заново прогоняет пять проверок перед push выше (fmt
   / clippy / test / build / doc) на затегированном коммите. Не
   зелёно здесь — не будет публикации.
2. **`publish`** — `cargo publish` через crates.io **Trusted
   Publishing** (OIDC): воркфлоу обменивает свою короткоживущую
   identity GitHub на токен crates.io, действующий только в рамках
   прогона, привязанный к окружению `crates-io`. Долгоживущий токен
   реестра в репозитории не хранится. Перед загрузкой подтверждается,
   что тег совпадает с версией в `Cargo.toml`.
3. **`github-release`** — создаёт GitHub Release для тега и вытаскивает
   соответствующий раздел `## [X.Y.Z]` из `CHANGELOG.md` в качестве
   release notes. Так что **запись CHANGELOG И ЕСТЬ release notes**:
   пишите changelog до тега.

Что это значит на практике:

- **Всегда пишите запись CHANGELOG до тега.** Пропуск этого шага
  оставит GH Release с пустыми notes. Правьте источник в
  `root-docs/CHANGELOG/` и пересобирайте через
  `node scripts/build-docs.mjs` — никогда не правьте `CHANGELOG.md`
  вручную.
- **Никогда не запускайте `cargo publish` с машины мейнтейнера** —
  канонический путь — это воркфлоу. (Ручная публикация пропустила бы
  verify-гейт и не создала бы GH Release.)
- Если нужно повторить релиз, не двигая тег, используйте
  `workflow_dispatch` с именем тега в качестве входа.

>>>>> lang=zh
## 发布

本 crate 的发布**完全自动化**——推送一个 `v*` tag,`Release`
工作流会处理一切:

```
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

这会在 `.github/workflows/release.yml` 中触发三个依次执行的 job:

1. **`verify`**——在已打 tag 的提交上重新运行上面的五项推送前检查
   (fmt / clippy / test / build / doc)。此处不通过,就不会发布。
2. **`publish`**——通过 crates.io **Trusted Publishing**(OIDC)执行
   `cargo publish`:工作流用其短期有效的 GitHub 身份换取一个仅在本次
   运行期间有效的 crates.io 令牌,绑定到 `crates-io` 环境。仓库中不
   存储任何长期有效的注册表令牌。上传前会确认 tag 与 `Cargo.toml`
   中的版本一致。
3. **`github-release`**——为该 tag 创建 GitHub Release,并从
   `CHANGELOG.md` 中提取对应的 `## [X.Y.Z]` 小节作为发布说明。因此
   **CHANGELOG 条目本身就是发布说明**:打 tag 之前先写好 changelog。

这在实践中意味着:

- **务必在打 tag 之前写好 CHANGELOG 条目。** 跳过这一步会让 GH
  Release 的说明留空。请编辑 `root-docs/CHANGELOG/` 下的源文件,并通过
  `node scripts/build-docs.mjs` 重新生成——绝不要手动修改
  `CHANGELOG.md`。
- **绝不要在维护者本机运行 `cargo publish`**——工作流才是规范路径。
  (手动发布会跳过 verify 关卡,也不会产生 GH Release。)
- 如果需要在不移动 tag 的情况下重试发布,使用 `workflow_dispatch`,
  以 tag 名称作为输入。

