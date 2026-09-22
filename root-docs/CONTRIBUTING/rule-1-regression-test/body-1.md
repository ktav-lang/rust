>>>>> lang=en
## Core rules

### 1. Every bug fix ships with a regression test

When you find a bug, **before fixing it**, write a test that reproduces
it — the test **must fail on `main`** and pass after the fix. Include
both in the same PR.

The test should live near related tests (e.g. in
`tests/edge_cases/<topic>.rs` or `tests/ser/<topic>.rs`). A short
comment on the test explains the failure mode so a reader ten months
from now understands *why* the case matters.

Why: silent regressions are the deadliest thing that can happen to a
library shipped to many users. A documented trip-wire costs nothing
long-term.

>>>>> lang=ru
## Основные правила

### 1. Каждый bug-fix сопровождается регрессионным тестом

Когда вы нашли баг, **перед тем как чинить**, напишите тест, который
его воспроизводит — тест **должен падать на `main`** и проходить
после фикса. Включите оба изменения в один PR.

Тест должен жить рядом со связанными тестами (например, в
`tests/edge_cases/<topic>.rs` или `tests/ser/<topic>.rs`). Короткий
комментарий к тесту объясняет характер падения, чтобы читатель через
десять месяцев понимал, *почему* этот случай важен.

Почему: молчаливые регрессии — самое смертельное, что может
случиться с библиотекой, отгружаемой множеству пользователей.
Документированная растяжка не стоит ничего в долгосрочной
перспективе.

>>>>> lang=zh
## 核心规则

### 1. 每个 bug fix 都要附带一个回归测试

发现 bug 后,**在动手修复之前**,先写一条能复现它的测试——该测试
**在 `main` 上必须失败**,修复之后通过。两者合并进同一个 PR。

测试应放在相关测试旁边(例如 `tests/edge_cases/<topic>.rs` 或
`tests/ser/<topic>.rs`)。在测试上写一条简短注释,说明故障方式,
好让十个月之后的读者理解这个用例*为什么*重要。

原因:对于一个分发给众多用户的库来说,沉默的回归是最致命的事情。
一条有文档的绊线,长远来看成本为零。

