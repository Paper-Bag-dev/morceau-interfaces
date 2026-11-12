# bike-viewer

An interactive 3D bike (three.js): orbit, zoom, explode, spin, repaint. Layout: `cache/seed/bike-viewer.json`. Any model in `ui/models/` works via the `model3d` component.

**Data:** `{"title": "Bike", "paint": "#f4764e", "spin": true, "explode": 0}`
**Push:** `morc bike [--paint "#hex"]` or `morc use bike-viewer --data @file.json --max-frac 0.8 --ttl 600`.
**Cost:** animated 3D defaults to lite quality and a capped fps; measure CPU before leaving it running.

**Actions (0.2):** `paint {color}`, `spin {on}`, `explode {on}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
