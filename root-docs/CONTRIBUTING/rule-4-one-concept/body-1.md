>>>>> lang=en
### 4. One concept per commit

Commits should be atomic: a bug fix and its test together, a new
feature and its tests together. A rename belongs in its own commit. A
refactor that happens to fix a bug should probably be two commits.

`git log --oneline` reads like a changelog. Write it that way.

>>>>> lang=ru
### 4. Одна концепция — один коммит

Коммиты должны быть атомарными: bug-fix и его тест вместе, новая
фича и её тесты вместе. Переименование — отдельный коммит. Рефакторинг,
который попутно чинит баг, — вероятно, должен быть двумя коммитами.

`git log --oneline` должен читаться как changelog. Пишите именно так.

>>>>> lang=zh
### 4. 一个概念,一个 commit

Commit 应保持原子:bug fix 与其测试一起提交,新特性与其测试一起
提交。重命名单独一个 commit。一个顺带修 bug 的重构,通常应该拆成
两个 commit。

`git log --oneline` 应读起来像一份 changelog。按这个方式写。

