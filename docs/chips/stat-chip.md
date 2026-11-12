# stat-chip

ONE number as a simple squircle rectangle: a white hero numeral with its small unit, and a grey one-line caption beside it. Several
numbers go in ONE `stat-tiles` strip instead (a long row, each number in its own small rectangle). No tiles, no
accent edge, no Close (the card's own dismiss is enough). Layout: `cache/seed/stat-chip.json` (component `stat`), `opts.size: "chip"`.

**Data:** `{"label": "Calls today", "value": 128, "unit": ""}`
**Push:** `morc use stat-chip --id stat-calls --data @file.json --region auto`. Sizes: tokens/plate.json `chip-*`.

**Actions (0.2):** `set {value}`, `bump {by}`, `label {text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
