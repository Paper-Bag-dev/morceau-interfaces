---
name: morc-screen
description: Put floating, interactive, self-expiring UI on the user's screen (charts, stat cards, forms, asks, 3D, graphs) through the local morc host, and manage cards that are already there (check, hide, bring back, move, resize, raise or lower, change their data). Use when the user asks to be shown something visually, when something needs their attention or a decision, or when they ask to change a card on screen.
---

# Morceau screen agent

Show things on the user's screen with the `morc` CLI (or the `morc mcp` tools) instead of writing text when a visual is better. Full rules:
`docs/agent.md` in the Morceau install. The short version:

1. **Reuse first.** `morc list`, then `morc use <name> --data @file.json` (about 80 tokens). Next `morc compose --recipe FILE --use`. Only then a
   new spec (`morc save`). Building a new component: the `morc-build` skill.
2. **Place by name.** `--region right|below|left|above|top-right|...|auto`, `--align`, `--dx/--dy`. The host does the geometry.
3. **Check with the tree, not a screenshot.** After a push: `morc tree <id>`. Fix anything flagged `cut`, `clipped`, `offcard`, `empty`
   (in the chip's JSON, never by working around it). `morc tree` shows the whole screen: windows front to back, cards, what covers what.
   A screenshot only for looks (colors, contrast), and only if needed.
4. **Never show a card twice.** A card that exists is changed in place:
   | user says | do |
   |---|---|
   | hide it for a bit / bring it back | `morc card <id> hide` / `show` (same spot, same data; a closed card comes back from its record) |
   | make it big and centred | `morc card <id> set '{"frac":0.9,"center":true}'` |
   | move it / put it next to X | `morc card <id> move --at X,Y` / `--center` / `--snap` |
   | get it out of my way / bring it to the front | `morc card <id> lower` (under the window) / `front` |
   | change a value | `morc patch <id> /path value`, or a named action `morc do <id> <action> '{...}'` |
   | what is up / what did I hide | `morc cards`, `morc card <id> get` |
5. **Shared values**: bind several cards to `@/name` and set it once with `morc patch @shared /name value`.
6. **Rules the linter enforces:** no dead controls; answers resolve; every number has a unit, a label and a real value; actions only from
   `actions.json`.
7. **Events.** The hooks inject `[morc] N UI interaction(s)` lines (or the MCP `wait_events` tool returns them): act on them, then stop.
8. **Complaints** about size, frequency or motion change the preference vector first: `morc annoy nudge size_bias 0.85 --why "<their words>"`.
9. Never move their cursor or click for them.
