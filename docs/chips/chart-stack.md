# chart-stack

Two charts in ONE window (a donut over a bar chart) plus Close. Use it for a tall free side: charts are pictures, so they may share a window. Layout: `cache/seed/chart-stack.json`.

**Card:** `morc card tok-charts --chip chart-stack --align right --importance 0.7 --min-importance 0.7 --set pie.title="Tokens by model" --set pie.values=[370,281,146,9] --set pie.labels=[Sonnet 5.5,Opus 5.5,Opus 4.8,Haiku 5.5] --set bars.title="New tokens per day" --set bars.values=[0.4,0.1,0.3,0.2,4.7] --set bars.labels=[Wed,Thu,Fri,Sat,Sun]`
**Show:** `morc push tok-charts`. At most 5 or 6 bars so it stays narrow; labels 12 characters or fewer.
**Size (11-12):** donut and bars are 200 px tall (were 140 / 130), min 240x440: the user asked for bigger charts; place on the tall left/right side.
**Units (11-12):** each chart's unit is data, `pie.unit` / `bars.unit` (default `M`). The seed binds it (`"unit": {"bind": "/pie/unit"}`); `chart.js` accepts a string or a bind. Pass `--set pie.unit=""` for plain counts, `--set bars.unit=%` for percents.

**Actions (0.2):** `setPie {title?, values?, labels?, unit?}`, `setBars {...}` (merged into what is there). Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.

## Changes and learned (read before every use)
- 11-16: it renders about 480-500 px wide at importance 0.7 (240 is its floor, not its size), so it does NOT fit a 342 px side column. It covered the terminal by about 170 px. Check that the free side is at least 500 px wide before choosing it; otherwise hold it or pick another chip.
- 11-16: the card owns its width once shown: shorter labels after the push, or a same-id re-push, do not narrow it. Get the data right before the first push.
- 11-16: `morc push NAME --region right` reads `right` as a card name. Set the region in the card file (`morc card NAME --region right`).
- 11-16, user: "it says min in the pie and graphs". CAUSE FOUND: the cached copy `cache/components/chart-stack.json` predated the 11-12 seed (unit typed as "min", heights 140/130), so the card's `pie.unit`/`bars.unit` were ignored and the donut read "48.8min". FIX: `morc save cache/seed/chart-stack.json` (old copy backed up), then same-id `morc push <id> --force` refreshed it in place (the donut centre then read "48.8k"). The stale literal was visible in the data all along: compare cache with seed before every push. After any chip seed changes, re-save its cache the same way. Verify from data, never by screenshot (user: "thats kinda cheating"): every unit, title and label in `cache/components/chart-stack.json` must be a `{bind}`, every bound path present in the card data, and the cache equal to `cache/seed/chart-stack.json` (see ~/.claude/RunningMemory.md).
