# Graphs: one schema, any source, one component

Anything with nodes and links can be visualised: code, goals and habits, dependencies, a flow. Sources produce the **schema**; the `graph`
component (Cytoscape.js, vendored in `ui/vendor`, offline) renders it; `graph-view` is the cached surface; `morc graph` ties them together.

```
 source ──▶ schema JSON ──▶ morc graph ──▶ host (use graph-view, data /graph) ──▶ Store ──▶ graph component ──▶ Cytoscape
 code-graph index (tools/graph_export.py)        or morc show/patch with your own JSON at /graph
```

## Schema
```json
{ "title": "optional",
  "nodes":  [ {"id": "src/host.rs", "label": "host.rs", "group": "src", "size": 24} ],
  "edges":  [ {"source": "src/host.rs", "target": "src/surface.rs", "weight": 22, "kind": "calls"} ],
  "flow":   ["ui/components/button.js", "ui/core/render.js"],
  "groups": [ {"id": "src", "label": "Rust host"} ] }
```
`group` becomes a labelled layer box (colors walk round from the accent); `size` scales the dot; `weight` thickens the link; `flow` is an ordered route drawn as the
orange dashed, slowly marching path with its nodes labelled; `groups` is optional (derived from the nodes); a node with `x` and `y` (canvas px) keeps that spot instead of the packed grid (draw trees and diagrams yourself); `"labels": "all"` names every dot, not only the route. Unknown fields are ignored.

## Use it
| goal | command |
|---|---|
| files of a project, with a route highlighted | `morc graph --root morc/ --flow report` |
| directories as nodes | `morc graph --root tools/ --langs python --level dir` |
| call neighbourhood of a symbol | `morc graph --symbol layout --depth 2` |
| another repo (index it first: `code-graph-mcp incremental-index` at its root) | `morc graph --repo <path> --root app/ --langs python --level dir --min-weight 5` |
| any graph you computed | `morc graph --file mygraph.json --title "Goals and habits"` |
| live update a graph on screen | `morc patch graph /graph '<json>'` |

Interactions: drag a dot, pan the background, wheel to zoom, double-click to fit, hover lights a node's neighbourhood (uses X, used by Y).

## Add a source
Write a script that prints the schema (see `tools/graph_export.py`: ~100 lines); allowlist it in `actions.json` if a button should run it (`run` + `into:"/graph"`); done.
Ideas: goals -> habits -> tasks; dependency trees between your own notes or projects; a process or service tree.

## Keeping the code graph fresh
The index is `<repo>/.code-graph/index.db`, built by the code-graph-mcp tool (build it from source and register it as an MCP server); `morc graph` reads it read-only.
Refresh: `code-graph-mcp incremental-index` at the repo root. Named routes live in `tools/flows.json`; trace one with `code-graph-mcp callgraph <symbol> --file <path>`.
