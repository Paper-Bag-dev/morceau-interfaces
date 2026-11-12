# week-chart

One bar (or line) chart with a title and Close. Layout: `cache/seed/week-chart.json`.

**Data:** `{"title": "Study minutes", "values": [30,45,0,60,20,10,50], "labels": ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"], "unit": "min"}`
**Push:** `morc use week-chart --data @file.json`. Line instead of bar: copy the chip and change `kind`, do not edit this one.
**Size:** `--importance 0.8` smaller. Labels 12 characters or fewer; give the unit a plain name.

**Actions (0.2):** `set {values, labels}`, `title {text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
