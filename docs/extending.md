# Extending morc

Everything below is "drop a file in a folder"; no rebuild unless you touch Rust. Prefer these over editing core files.

| you want | do this | picked up |
|---|---|---|
| a new CLI command | `tools/cli/<name>.py` (or `.ps1` / `.cmd`) with a line `# morc: what it does` near the top | instantly: `morc <name> ...`, listed by `morc commands` |
| a new UI component | `morc ui new <name> --deps plate --types <type>`: a folder `ui/components/<name>/` with its code, styles and `metadata.json`; edit it, run `morc ui build` (index: `ui/COMPONENTS.md`) | next surface (the host lists `plugins/index.json`; main.js imports each before saying ready) |
| a new chart kind | `ui/charts/<kind>.js` exporting `draw(g, s)` + `hit(s, x, y)`; add it to `MODS` in `components/chart/chart.js` (+ `charts/explain.js` if unfamiliar) | next surface |
| a 3D model | `ui/models/<name>.js` exporting `build(THREE, opts)` (see `bike.js`); show with `{type:"model3d", model:"<name>"}` | next surface |
| a program a button may run | add a named entry to `actions.json` (argv + cwd), then `{do:"run", id:"...", into:"/path", then:{...}, fail:{...}}` | immediately (read per click) |
| live numbers on a surface | `sources: {"/x": {file, pointer, every}}` (JSON file + pointer, polled by the host) | on show |
| something that appears unprompted | `triggers/<name>.json` (source + condition + cooldown + a cached component) | within 10 s; cooldown stretches for ignored topics |
| a graph from any source | script that prints the schema in `docs/graphs.md`; show it with `morc graph --file x.json` | immediately |

Extension scripts get `MORC_BIN` (the morc exe) and `MORC_HOME` in their environment, so a script can gather data and call `morc show|use|stream|graph|patch`.
Built-in commands always win over a script with the same name. Scripts run only when someone runs the CLI (never from a spec or a click).

Every extension still obeys the house rules (docs/agent.md): no dead controls (the linter checks `show`, `save` and `stream`), answers resolve, rounded + coral,
text on plates, and the annoyance vector decides size/frequency/motion.

## Streaming builds (`morc stream`)
`morc stream spec.json|- [--delay 200] [--id X]` takes the same spec as `show`, but an empty frame appears first and the components are mounted one at a time,
breadth-first from `root`, each fading in (charts/lists/graphs wait 1.5x longer, and charts play their intro as they arrive). A small "Building" indicator
fades out at the end. Nothing already on screen is rebuilt, so numbers and animations keep running. It is "text generation, but for components".

What suits it: at-a-glance dashboards, reports (hero number first, then detail), checklists and plans appearing as they are worked out, step-by-step explanations
where each step adds a visual, search results, anything where *seeing it assemble* tells the user something is happening.
What does not: urgent asks (they must be instant), widgets of 1-3 pieces (nothing to stream), anything the user is mid-click on.
Rhythm: 6-9 pieces, about 200 ms apart (overlap is fine; the fades and the Building indicator are the feedback), hero first, one idea per piece; slow down for the heavy ones, never stream more than ~4 s in total.
Under the hood: `Request::Patch.attach {parent, before?}` mounts the first component of a patch into an existing parent (`Renderer.attach` in `core/render.js`).