# stat-tiles

Four big numbers together in ONE text card (2 by 2), each with a label that explains it and a unit, plus Close. Layout: `cache/seed/stat-tiles.json`.

**Philosophy:** numbers (text) go together in one card; pictures (charts) go in their own separate cards (`donut-card`, `week-chart`). Do not mix them in one card.

**Data:** `{"a": {"label": "Tokens, all time", "value": 808, "unit": "M"}, "b": {"label": "New today", "value": 4.5, "unit": "M"}, "c": {...}, "d": {...}}`
**Push:** `morc use stat-tiles --id <id> --data @file.json --region auto --importance 1.2`.
**Size:** dark text cards read small next to light charts: 1.2 to 1.35; smaller 1.0.

**Actions (0.2):** `set {key: a..d, value}`, `label {key, text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
