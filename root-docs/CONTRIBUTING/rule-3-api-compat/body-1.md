>>>>> lang=en
### 3. Public API changes note compatibility

If you touch anything under `pub` in `lib.rs`, in the PR description
say whether it's:

- **semver-compatible** (additions, looser bounds, doc changes); or
- **semver-breaking** (renamed / removed items, changed signatures,
  tightened bounds) — in which case the version bump goes into the
  next `MINOR` while we're pre-1.0.

Add a CHANGELOG entry for your change under `root-docs/CHANGELOG/` in
the same PR (regenerate the artifacts with `node scripts/build-docs.mjs`
— never hand-edit `CHANGELOG.md`).

>>>>> lang=ru
### 3. Изменения публичного API отмечают совместимость

Если вы трогаете что-либо под `pub` в `lib.rs`, в описании PR
укажите — это:

- **semver-совместимо** (добавления, ослабленные bounds, правки
  документации); или
- **ломает semver** (переименования / удаления, изменённые сигнатуры,
  ужесточённые bounds) — в этом случае bump версии идёт в следующий
  `MINOR`, пока мы pre-1.0.

Добавьте запись CHANGELOG для вашего изменения в `root-docs/CHANGELOG/`
в том же PR (пересоберите артефакты через
`node scripts/build-docs.mjs` — никогда не правьте `CHANGELOG.md`
вручную).

>>>>> lang=zh
### 3. 公共 API 变更要标注兼容性

若你改动了 `lib.rs` 中任何 `pub` 下的项,请在 PR 描述中声明它属于:

- **semver 兼容**(新增、放宽 bound、文档改动);或
- **破坏 semver**(重命名 / 删除项、改签名、收紧 bound)——在
  pre-1.0 阶段,版本号递进进入下一个 `MINOR`。

请在同一个 PR 中于 `root-docs/CHANGELOG/` 下为你的改动添加一条
CHANGELOG 记录(通过 `node scripts/build-docs.mjs` 重新生成产物——
绝不要手动修改 `CHANGELOG.md`)。

