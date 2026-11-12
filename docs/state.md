# State: stores, named actions, the shared store, disk

| home | where | lives | change it with |
|---|---|---|---|
| card store | each card page (`ui/core/store.js`) | while the card is open | `morc patch <card> /path value`, page actions, named actions |
| shared store | the hidden root page (`ui/core/shared.js`), zustand's vanilla API | while the host runs | `morc patch @shared /path value`, `{"do":"set","path":"@/x"}` |
| disk | `<home>/state/<id>.json` + `<id>.history.jsonl` | across closes and restarts | `opts.persist: true` |

## Paths
A pointer is a card-store path (`/items`); with `@` in front it is the shared store (`@/focus`). Both work everywhere a path does:
`{"bind": "@/focus"}`, templates `"focus: {@/focus}"`, `when: {"path": "@/mode"}`, and the actions below. Shared needs linked cards
(docs/process-model.md); unlinked cards (`MORC_LINKED=0`) share through the host instead: a write goes to the host, which keeps the
state and relays it to every other card, and a new card starts from the host's copy. Same API either way.

## Page actions
`set`, `toggle` (as before), and the list and record edits: `push {path, value}`, `pull {path, id | index | value}`,
`merge {path, value}`, `inc {path, by=1}`, `call {name, args}` (a named action). The linter accepts them like `set` (the path must be
shown on the card, except `@/` paths, which another card may show).

## Named actions
Declared in the spec's `opts.actions` (so `morc save` keeps them with the chip). Each is a list of page actions; strings in it take the
call's args: `"$"` all args, `"$.text"` one arg (value kept as is), `"{$.text}"` inside a string, `"$id"` a fresh id (the same for every
step of one call, so paired updates agree), `"$now"` ms time.
```json
"opts": {"persist": true, "actions": {
  "add":    [{"do": "push", "path": "/items", "value": {"id": "$id", "text": "$.text"}}],
  "remove": [{"do": "pull", "path": "/items", "id": "$.id"}],
  "focus":  [{"do": "set",  "path": "@/focus", "value": "$.text"}]}}
```
Run one: `morc do tasks add '{"text":"Buy milk"}'` (or `@args.json`), a button `{"do":"call","name":"add","args":{...}}`, or the MCP
`do_action` tool. The answer is the card's data after it ran, or `{"ok":false,"err":"no action \"x\" (this card has: add, remove)"}`.
Prefer idempotent actions (`setDone`, not `toggle`): a retried call then does no harm. New behaviour arrives as actions (data) or as
component / plugin code; there is no eval of code sent at run time.

## Disk (`opts.persist`)
- The page sends its data ~1 s after the last change; the host writes `{schema, saved_at, data}` to a temp file and renames it (never half a file).
- Every named action is appended to `<id>.history.jsonl` as it runs (`{ts, name, args}`), so nothing is lost between snapshots.
- On the next show with the same id, the snapshot is merged over the spec's starting data.
- No big content or secrets in stores: keep a file path and a version, not the file. Local data folder only. The shared store is not persisted yet.
