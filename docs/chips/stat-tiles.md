# stat-tiles

Four big numbers together in ONE text card (2 by 2), each with a label that explains it and a unit, plus Close. Layout: `cache/seed/stat-tiles.json`.

**Philosophy:** numbers (text) go together in one card; pictures (charts) go in their own separate cards (`donut-card`, `week-chart`). Do not mix them in one card.

**Data:** `{"a": {"label": "Tokens, all time", "value": 808, "unit": "M"}, "b": {"label": "New today", "value": 4.5, "unit": "M"}, "c": {...}, "d": {...}}`
**Push:** `morc use stat-tiles --id <id> --data @file.json --region auto --importance 1.2`.
**Size:** dark text cards read small next to light charts: 1.2 to 1.35; smaller 1.0.

**Actions (0.2):** `set {key: a..d, value}`, `label {key, text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.

## Changes and learned (read before every use)
- 11-16, user: "its all crowded". Labels are 2-3 words (18 characters at most), units short (`tok`, `%`, `min`), never a sentence. If a number needs explaining, the card is the wrong place.
- 11-16: a card opened with `morc use` has NO actions (`label` fails with "this card has: none"). Fix data in place with `MSYS_NO_PATHCONV=1 morc patch <id> /a/label '"text"'`, or open it with `morc card` + `morc push` to get the actions.
- 11-16: about 840x235 px at importance 1.1 to 1.2. His drag for `stats.tiles`: bottom centre (806,836).
- 11-16, user: "text kinda leaks outta the black rounded bg". Fixed in the components, not the card: `stat/stat.css` + `plate/plate.css` now size tiles from their text (see their md). Long units are fine now; still keep labels 2-3 words.
