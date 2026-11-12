# graph-view

A node-link graph (code, goals, dependencies, flows) on Cytoscape: draggable nodes, layers, an optional animated route. Layout: `cache/seed/graph-view.json`.

**Data:** `{"title": "Services", "graph": {...}}`; the graph schema is in `../graphs.md`. `morc graph ...` fills it from the code graph or a JSON file.
**Push:** `morc use graph-view --data @file.json --max-frac 0.8`. A working window: it may be large, give it room.

**Idle fold (11-11):** the seed sets `opts.idle: 0.4`: the card rests at 40% of its size while nobody uses it, springs to full on hover, folds back after `limits.shrink-delay-ms`. Any chip can set `opts.idle`; the resting zoom floor is `limits.idle-min-zoom`.

**Keep it open (11-12):** `morc use` cannot drop the idle fold. For a graph that should stay full size, `morc show` a spec built from the seed: `{"id":"graph-view","opts":<seed opts without idle/ttl, plus "max_frac":0.8>,"components":<seed components>,"data":{"title","graph"}}`.

**First-fit fix (11-12):** the first `cy.fit` could run before the canvas had its size and open at max zoom (small graphs, or ones with `x`/`y`). `graph.js` now fits again on the next frame. Prefer the automatic layout over hand-placed `x`/`y` unless a tree really needs it.

**Actions (0.2):** `setGraph {nodes, edges, ...}`, `title {text}`. Run with `morc do <id> <action> '{...}'` or the MCP `do_action` tool; no raw paths needed.
