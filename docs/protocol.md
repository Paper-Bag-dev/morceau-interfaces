# Protocol

One JSON object per line over `127.0.0.1:47831`; envelope `{"token": "...", "req": {...}}`. The CLI does this for you.

## Spec (what `show` takes)
```json
{"id": "cpu", "opts": {"ttl": 15, "sticky": false, "urgent": false, "anchor": "near_focus",
                       "topic": "pc.cpu", "theme": "ocean", "auto": false, "force": false},
 "components": [ {"id": "root", "type": "column", "children": ["a", "b"]},
                 {"id": "a", "type": "text", "text": "CPU {/cpu:0}%"},
                 {"id": "b", "type": "progress", "label": "load", "value": {"bind": "/cpu"}, "max": 100, "unit": "%"} ],
 "data": {"cpu": 12},
 "sources": {"/cpu": {"file": "%LOCALAPPDATA%/x/stats.json", "pointer": "/cpu", "every": 2}}}
```
- Components are a **flat list with ids**; `root` is the entry (or the first). Containers list `children` ids.
- Any prop can be a literal, `{"bind": "/path"}`, or a string with templates `{/path}` / `{/path:1}` (decimals).
- Same `id` shown again = refresh in place (restarts the clock), no stacking.

## Opts
`ttl` seconds idle (default 15) | `sticky` no timer (still quietly removed if untouched 10 min) | `urgent` sticky + ring + beep, parked
if ignored | `anchor` near_focus (default) / corner / center / side / bottom | `topic` engagement bucket | `theme` name, `random`, or
omit for topic-matched + daily drift | `auto` set by triggers: throttled by engagement, deferred while a fullscreen app is up | `force` skip that.

## Commands
`show`, `use <name> [--data JSON] [--id] [--ttl] [--sticky] [--urgent] [--anchor] [--topic] [--theme] [--auto]`,
`patch <id> <pointer> <value> ...`, `close <id>`, `save`, `summon`, `list`, `status`, `stop`, `registry`, `inbox [--peek]`, `hook <Event>`.
`--data` on PowerShell: pass a file via `show`/`save` instead (quote mangling), or use single-quoted here-strings.

## Actions (on buttons, list rows, input submit): one object or an array
| do | where | what |
|---|---|---|
| `set` `{path,value}` / `toggle` `{path}` | page | change the data model (instant, animated) |
| `copy` `{text}` | page | clipboard |
| `close` | page | close this surface (counts as engaged) |
| `open` `{target}` | host | http(s) URL or existing path |
| `run` `{id, into?}` | host | run a named command from `actions.json`, put stdout at `into` (default `/_out`) |
| `emit` `{payload}` | host | write to the agent inbox, no wake |
| `wake` `{payload}` | host | inbox + wake the screen agent (see `agent.md`) |
Payload strings may contain `{/path}` templates, e.g. an input's `{/draft}`.

## Page -> host (internal)
`ready`, `layout {w,h,rects}`, `hover`, `pin`, `drag`, `kbd`, `act`, `close`, `dismiss`, `literacy`, `left`, `log`.

## More opts (added)
`importance` low|normal|high|critical|number | `fill` grow to the 1/4-screen cap | `contrast` "off" disables backdrop colors.
Button props: `shape` "soft" | "circle" (default pill). Chart props: see `charts.md`.
CLI: `morc annoy ...` (the annoyance vector, see `agent.md`), `morc use ... --importance high --fill --contrast off`.

## Icons and variants
Button `icon`: check, x, plus, minus, arrow-right, arrow-left, refresh, copy, external, trash, info, play, pause, bell, clock (`ui/core/icons.js`).
Variants: primary, secondary, ghost, good (green), bad (red outline), destructive. `opts.lint:"off"` skips the dead-control linter.
## Conditional visibility and the answers-resolve rule
Any component: `"when": {"path": "/answered", "eq": false}` (also `ne`, `truthy`, `falsy`): shown only while true, with enter/leave animation and the
window re-fitting. Pattern for a question: group with `when answered eq false` containing the text + buttons whose actions `set` a result and `answered=true`;
a result text with `when answered eq true`. Buttons that `set` without hiding themselves (or a parent), closing, or `role:"tab"` are refused.