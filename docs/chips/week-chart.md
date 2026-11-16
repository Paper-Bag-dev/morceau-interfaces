# week-chart

One bar (or line) chart with a title and Close. Layout: `cache/seed/week-chart.json`.

**Data:** `{"title": "Study minutes", "values": [30,45,0,60,20,10,50], "labels": ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"], "unit": "min"}`
**Push:** `morc use week-chart --data @file.json`. Line instead of bar: copy the chip and change `kind`, do not edit this one.
**Size:** `--importance 0.8` smaller. Labels 12 characters or fewer; give the unit a plain name.

**Actions (0.2):** `set {values, labels}`, `title {text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.

## Changes and learned (read before every use)
- 11-16: about 595x283 px at default importance: too wide for a narrow side column. The unit is appended raw to every bar value (`3601tokens`), so keep units short and plain.
- 11-16: verify from data, never by screenshot: the cached spec's unit must be a `{bind}`, its path present in the card data, and the cache equal to the seed (~/.claude/RunningMemory.md).
- 11-16: DEFECT found by the data check: the shipped seed (`cache/seed/week-chart.json`, and the cache copy) types the unit as a literal `"m"` on component `c` instead of a bind, so every week-chart reads "m" whatever unit the card sends (same class as the chart-stack "min" bug). Not fixed yet (a2ui seed change, needs his OK); do not use this chip until the unit is bound to `{"bind": "/unit"}`.
- 11-16: FIXED the defect above: the unit is now `{"bind": "/unit"}` in seed and cache, sample data unit "min". Pass `unit` in the card data.
