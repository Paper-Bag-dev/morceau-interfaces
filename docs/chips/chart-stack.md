# chart-stack

Two charts in ONE window (a donut over a bar chart) plus Close. Use it for a tall free side: charts are pictures, so they may share a window. Layout: `cache/seed/chart-stack.json`.

**Card:** `morc card tok-charts --chip chart-stack --align right --importance 0.7 --min-importance 0.7 --set pie.title="Tokens by model" --set pie.values=[370,281,146,9] --set pie.labels=[Sonnet 5.5,Opus 5.5,Opus 4.8,Haiku 5.5] --set bars.title="New tokens per day" --set bars.values=[0.4,0.1,0.3,0.2,4.7] --set bars.labels=[Wed,Thu,Fri,Sat,Sun]`
**Show:** `morc push tok-charts`. At most 5 or 6 bars so it stays narrow; labels 12 characters or fewer.
**Size (11-12):** donut and bars are 200 px tall (were 140 / 130), min 240x440: the user asked for bigger charts; place on the tall left/right side.
**Units (11-12):** each chart's unit is data, `pie.unit` / `bars.unit` (default `M`). The seed binds it (`"unit": {"bind": "/pie/unit"}`); `chart.js` accepts a string or a bind. Pass `--set pie.unit=""` for plain counts, `--set bars.unit=%` for percents.

**Actions (0.2):** `setPie {title?, values?, labels?, unit?}`, `setBars {...}` (merged into what is there). Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
