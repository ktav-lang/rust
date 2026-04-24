# Security Policy

**Languages:** **English** · [Русский](SECURITY.ru.md) · [简体中文](SECURITY.zh.md)

## Supported versions

While this crate is pre-1.0 only the **latest published minor** is
maintained. Security fixes land on `main` and ship in a PATCH
release within a few days.

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | ✅                 |
| older   | ❌ — upgrade first |

## Reporting a vulnerability

**Please do not open a public issue for security problems.**

Email **phpcraftdream@gmail.com** with:

- A short description of the vulnerability.
- A minimal reproducer (Ktav input that triggers the behaviour, the
  affected API — `parse` / `from_str` / `to_string` / `render`, and
  expected vs actual).
- The ktav version you observed it on (`cargo tree -p ktav` output is
  usually enough), plus the Rust toolchain if non-standard.
- Your disclosure timeline preference, if you have one.

You should get an acknowledgement within **72 hours**. A published
fix typically follows within **a week** for high-impact issues, longer
if the fix needs to coordinate with a binding or the format spec.

## Scope

This is the reference Rust crate that every other binding wraps. A
real issue here usually affects `ktav-lang/python`, `ktav-lang/js`,
`ktav-lang/golang` at once — treat it accordingly.

Issues that count as security problems for this crate:

- Panics on crafted input reaching `parse` / `from_str` / `to_string`
  / `render`. The crate targets `panic = "abort"` builds in downstream
  bindings, so a panic terminates the consumer process.
- Runaway memory or CPU (quadratic behaviour, unbounded allocation,
  infinite loops) on crafted input.
- `unsafe` correctness: any soundness hole in `unsafe` blocks —
  out-of-bounds access, UB, aliasing violations — even if no obvious
  exploit path exists.
- Any behaviour that allows crafted Ktav input to produce a `Value`
  outside the documented grammar (lossy round-trips that drop or
  forge data).

Issues that are **not** security problems here — please use regular
issues for these:

- Performance regressions without a DoS-shape characteristic.
- Parser error messages being unclear or imprecise.
- Problems in the Ktav format itself — those belong in
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec).
