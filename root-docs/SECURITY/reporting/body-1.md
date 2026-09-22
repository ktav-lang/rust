>>>>> lang=en
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

>>>>> lang=ru
## Сообщение об уязвимости

**Пожалуйста, не открывайте публичные issue по проблемам безопасности.**

Напишите на **phpcraftdream@gmail.com** и укажите:

- Краткое описание уязвимости.
- Минимальный воспроизводитель (Ktav-вход, запускающий поведение;
  затронутый API — `parse` / `from_str` / `to_string` / `render`;
  ожидаемое против фактического).
- Версию, на которой наблюдалось (обычно достаточно вывода
  `cargo tree -p ktav`) плюс Rust-тулчейн, если нестандартный.
- Предпочтительный таймлайн раскрытия, если у вас он есть.

Подтверждение получите в течение **72 часов**. Опубликованный фикс
обычно выходит в течение **недели** для высокоприоритетных проблем,
дольше — если фикс нужно согласовать с каким-то биндингом или со
спецификацией формата.

>>>>> lang=zh
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

