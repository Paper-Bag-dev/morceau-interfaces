# MCP: `morc mcp`

A stdio MCP server (newline-delimited JSON-RPC) over the same host requests the CLI sends; the CLI stays the main interface. Fixed
tools (seven): a new chip, action or plugin never changes the list, it is data the tools take.

| tool | does | same as |
|---|---|---|
| `scene` | the scene tree; `all: true` mounts every card's tree (`depth`) | `morc tree [--all]` |
| `card_tree` | one card's tree (`card`, `depth`, `node`) | `morc tree <card>` |
| `show_chip` | show a cached chip (`name`, `data`, `id`, `opts`) | `morc use` |
| `patch` | change data on a live card (`card`, `data: {"/ptr": v}`); `card: "@shared"` for the shared store | `morc patch` |
| `do_action` | run a named action (`card`, `action`, `args`) | `morc do` |
| `card` | a card as a resource: no `card` = list all; `card` = its state; `card` + `set` = the state you want (docs/cards.md) | `morc cards`, `morc card <id> get|set` |
| `wait_events` | wait up to `timeout_s` (≤ 300) for inbox rows (clicks, answers, copies), optionally from one `surface`; marks them read | `morc inbox` in a loop |

Register it in a client as a stdio server whose command is the `morc` binary with the argument `mcp`. Tool results are the host's JSON
reply as text; `isError` is set when the reply has `ok: false`.
