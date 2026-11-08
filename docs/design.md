# morc design notes (11-08, frontend-design pass)

**Subject:** tiny, glanceable instruments floating over arbitrary pixels, for one person. Job: be read in under a second, then leave.
**One memorable thing per surface:** the hero numeral with a hairline needle (`stat`). Everything else stays quiet.

| token | value | why |
|---|---|---|
| accent | coral `14 90% 63%` (#F4764E) | the user's explicit pick; one accent everywhere, pin/dismiss included |
| ink (floating text) | light, dark outline | subtitle style reads on bright and dark pages; dark text with a light halo did not |
| surfaces | warm dark `20 9% 9%` / warm paper `30 14% 96%` | flips against the page behind (anti-color), warm so it sits with the coral |
| display | Bahnschrift, width axis 92 | DIN-like numerals; ships with Windows, no network |
| body | Segoe UI Variable Text | legible small; clearly different from the display face |
| scale | 10.5 / 12 / 15 / 22 / 32 px | tabular figures on anything that changes |

**Rules:** no all-caps labels, no eyebrows, no middle-dot meta strings, no mono data labels, no arrows on buttons. Copy says what a button does
("Close", "Show 7-day report"), sentence case. Non-user-triggered motion only when it earns attention: the shimmer plays only on an urgent ask.
User-triggered motion (ripple, press-bounce, tick, hover lift, springs on arrival/exit) stays, because he asked for it and it shows what changed.
**Drag glow:** components glow orange, strongest on the biggest block, weaker as they get smaller; no outer ring.
**Tried and cut:** random/daily accent rotation (he wants one consistent coral); green/red yes-no (breaks the theme); shimmer on every primary button.

**Fun to read:** a report or recap is a few big numbers and one-line captions, never a text list. Prefer hero numerals over sentences, at most 3-4 short items, biggest type that fits, a little personality in the wording. If it reads like notes, cut it down or turn it into numbers.
