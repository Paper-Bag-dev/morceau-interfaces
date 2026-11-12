# line-chart

A short series as a smooth line; on the glass skin the stroke and its fading fill run through the theme palette (accent first).
Copy of `week-chart` with `kind: "line"`. Layout: `cache/seed/line-chart.json`, `opts.size: "large"` (big, grows on hover and then labels every point).

**Data:** `{"title": "Calls per hour", "values": [4, 9, 14, 11, 18, 22, 15], "labels": ["9","10","11","12","13","14","15"], "unit": "calls"}`
**Push:** `morc use line-chart --id <id> --data @file.json --region auto`.

**Actions (0.2):** `set {values, labels}`, `addPoint {value, label}`, `title {text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
