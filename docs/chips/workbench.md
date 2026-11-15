# workbench (explorer, editor, flow, problems)

A floating, VS Code-like app made of separate cards that move together. Each card is a chip over one generic part; all of them read and write
one nav model in the shared store (`@/nav`: selection, tabs, back/forward), so a click in any card opens the item in every other.

| chip | part (`ui/components/`) | use |
|---|---|---|
| `workbench-explorer` | `explorer` | a grouped, searchable list; `/only` filters (two explorers = files and messages) |
| `workbench-editor` | `inspector` + `codeedit` | tabs, back/forward, breadcrumb, the open item; the whole file in Monaco (vendored), Ctrl+S saves via `wb-open` / `wb-save` |
| `workbench-flow` | `flowmap` | the whole graph on React Flow (vendored): pan, zoom, minimap, drag; the open item's links animate |
| `workbench-problems` | `issues` + `steps` | what is broken or worth checking, and a route to walk step by step |

**Data:** one entity file (schema in `ui/components/nav/README.md`): `{entities, groups, route?}`. Any source can write one.
**Open:** each pane is a normal card: `morc use workbench-explorer --id wb-files --data @pane.json` (and the same for editor, flow, problems).
Lay them out like an IDE with what morc already has: read `morc tree` (work area, windows), size each pane with its chip's `/vars`
(`xp-w`, `xp-min-h`, `ins-w`, `ce-w`, `ce-h`, `fm-w`, `fm-h`, `is-max-h`, ...; `morc patch <id> /vars {...}`), then `morc card <id> move --at X,Y` (it glides).
Merge panes into one card only when there is no room apart (the ladder, INDEX.md rung 6).
**Size:** `large` (zoom never under 1); lists scroll inside.
