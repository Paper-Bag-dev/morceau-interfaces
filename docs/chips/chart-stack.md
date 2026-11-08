# chart-stack

Two charts in ONE window (a donut over a bar chart) plus Close. Use it for a tall free side: charts are pictures, so they may share a window. Layout: `cache/seed/chart-stack.json`.

**Card:** `morc card tok-charts --chip chart-stack --align right --importance 0.7 --min-importance 0.7 --set pie.title="Tokens by model" --set pie.values=[370,281,146,9] --set pie.labels=[Sonnet 5.5,Opus 5.5,Opus 4.8,Haiku 5.5] --set bars.title="New tokens per day" --set bars.values=[0.4,0.1,0.3,0.2,4.7] --set bars.labels=[Wed,Thu,Fri,Sat,Sun]`
**Show:** `morc push tok-charts`. At most 5 or 6 bars so it stays narrow; labels 12 characters or fewer.
