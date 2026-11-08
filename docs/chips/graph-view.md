# graph-view

A node-link graph (code, goals, dependencies, flows) on Cytoscape: draggable nodes, layers, an optional animated route. Layout: `cache/seed/graph-view.json`.

**Data:** `{"title": "Services", "graph": {...}}`; the graph schema is in `../graphs.md`. `morc graph ...` fills it from the code graph or a JSON file.
**Push:** `morc use graph-view --data @file.json --max-frac 0.8`. A working window: it may be large, give it room.
