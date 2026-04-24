# 安全策略

**语言:** [English](SECURITY.md) · [Русский](SECURITY.ru.md) · **简体中文**

## 支持的版本

本 crate 仍处于 pre-1.0 阶段，仅维护**最新发布的次版本**。安全修复
会进入 `main`，并在数日内以 PATCH 发布。

| 版本    | 支持                   |
|---------|------------------------|
| 0.1.x   | ✅                     |
| 更早    | ❌ —— 请先升级         |

## 上报漏洞

**请不要为安全问题开公开 issue。**

请发邮件至 **phpcraftdream@gmail.com**，并提供:

- 对漏洞的简短描述。
- 最小复现（触发该行为的 Ktav 输入；受影响的 API ——
  `parse` / `from_str` / `to_string` / `render`；预期 vs 实际）。
- 观察到问题时所用的版本（通常 `cargo tree -p ktav` 的输出就够了），
  以及非标准的 Rust toolchain（如有）。
- 你偏好的披露时间线（如有）。

你应在 **72 小时**内收到确认。对于高影响问题，已发布的修复通常在
**一周**内跟进；如果修复需要与某个绑定或格式规范协同推进，则可能更久。

## 范围

这是所有其他绑定都封装的参考 Rust crate。这里的真实问题通常会同时
影响 `ktav-lang/python`、`ktav-lang/js`、`ktav-lang/golang` —— 请据此对待。

以下问题会按本 crate 的安全问题处理:

- `parse` / `from_str` / `to_string` / `render` 在构造输入下 panic。
  downstream 绑定采用 `panic = "abort"` 构建，panic 会终止 consumer 进程。
- 构造输入导致的失控内存或 CPU 消耗（二次复杂度、无界分配、死循环）。
- `unsafe` 正确性：`unsafe` 块中的任何 soundness 漏洞 —— 越界访问、
  UB、aliasing 违例 —— 即使没有明显的利用路径。
- 任何允许构造的 Ktav 输入产生文档化语法之外 `Value` 的行为（丢数据
  或伪造数据的 lossy round-trip）。

以下**不**算本 crate 的安全问题 —— 请走普通 issue:

- 无 DoS 特征的性能回归。
- 解析器错误信息不清晰或不精确。
- Ktav 格式本身的问题 —— 这类问题属于
  [`ktav-lang/spec`](https://github.com/ktav-lang/spec)。
