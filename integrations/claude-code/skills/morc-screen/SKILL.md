---
name: morc-screen
description: Put floating, interactive, self-expiring UI on the user's screen (charts, stat cards, forms, asks, 3D, graphs) through the local morc host. Use when the user asks to be shown something visually, or when something needs their attention or a decision.
---

# Morceau screen agent

Show things on the user's screen with the `morc` CLI instead of writing text when a visual is better. Full rules: `docs/agent.md` in the Morceau install
(the file the background agent also uses as its system prompt). The short version:

1. **Reuse first.** `morc list`, then `morc use <name> --data '{...}'` (about 80 tokens). Next `morc compose --recipe FILE --use` (fragments, cached by
   structure; per-push `--set part.param=value` and `--data part.field=value`). Only then a new spec (`morc save`), always with `opts.min`.
2. **Place by name.** `--region right|below|left|above|top-right|bottom-right|bottom-left|top-left|auto`, `--align`, `--dx/--dy`. The host does the geometry.
3. **Rules the linter enforces:** no dead controls; answers resolve (the question disappears, the result shows); every number has a unit, a label and a
   real value; yes/no are check/x icons; actions only from `actions.json`.
4. **Design:** one hero numeral, one accent, rounded, small, sentence case; no all-caps, eyebrows, mono labels or arrows on buttons; type floors 11.5/12.5/15/22/32.
5. **Events.** The hooks inject `[morc] N UI interaction(s)` lines: the user's clicks and drags. Act on them (answer, patch a surface), then stop.
6. **Complaints** about size, frequency or motion change the preference vector first: `morc annoy nudge size_bias 0.85 --why "<their words>"`.
7. Verify what you push by looking (ask for a screenshot if you cannot); never claim a visual result you did not check; never move their cursor.
