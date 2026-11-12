# Changelog

## 0.2.0 (unreleased)

The redesign: Morceau runs on macOS as well as Windows, every card is a resource you can read and change, and a check of what is on
screen no longer needs a screenshot.

### Added
- **macOS.** The same engine and CLI on macOS (WKWebView); the npm package carries an engine per platform (`bin/morc.exe`,
  `bin/morc-darwin-arm64`, `bin/morc-darwin-x64`) and `npx morceau` picks the right one. `scripts/release-build-mac.sh` builds the macOS
  engines with no machine paths inside; the publish check refuses a package missing one.
- **One web process for every card** (linked cards, both systems). A hidden root page opens every card, so 20 cards cost one extra web
  process instead of twenty (measured on macOS: +1 process / +101 MB against +21 / +740 MB). `MORC_LINKED=0` turns it off.
- **Trees** (`morc tree`, docs/trees.md). The scene tree (displays, windows front to back with app names, cards as mount points,
  `covers` edges) and each card's tree (components with rects and flags: `cut`, `clipped`, `scrolls`, `offcard`, `hidden`, `empty`), plus
  the card's data keys, named actions, width owner, interaction state and buttons. `--all` merges them.
- **Cards as resources** (`morc card <id> get|set|hide|show|front|lower|raise|move|size`, `morc cards`, docs/cards.md). Hide a card and
  bring it back where it was; a closed card comes back with its last data, even after a restart. Records are written with a falloff
  (after 1 s of quiet, at most every 5 s): dragging a card around for fun costs one write per 5 s.
- **Window stack.** Each card has a depth: on top, or under the 1st, 2nd ... window that overlaps it. The card's third button sends it
  one level down and back up; `Up` / `Down` with the pointer on the card step it; `Ctrl+Win+Up/Down` (Windows, `Ctrl+Alt` if taken; macOS `Ctrl+Option+Up/Down`) send
  every card to the top / bottom. A card under a window gets no hover through it and is never moved out from under it.
- **Snapping.** A carried card snaps to the active window, other cards and the screen edges, slides along them and lets go when pulled
  away; a fast throw never snaps. Settings in `ui/limits.json`; `morc card <id> move --snap` from the CLI.
- **State** (docs/state.md). Named actions per card (`opts.actions`, `morc do`), list and record edits (`push`, `pull`, `merge`, `inc`),
  a store shared by every card (`@/path` in binds, templates and actions; `morc patch @shared`), and with `opts.persist` snapshots plus
  an action history on disk.
- **MCP** (`morc mcp`, docs/mcp.md). Seven fixed tools: `scene`, `card_tree`, `show_chip`, `patch`, `do_action`, `card`, `wait_events`.
- **Your own look and components** (docs/views.md, docs/plugins.md). Tailwind classes on any component (`"class"`), a generic `el`
  component for layouts written in the card JSON, Preact components (`registerView`), a look preset you own (`ui/preset.css`), plugins
  in your data folder (`<home>/plugins`), and `morc refresh` to reload them in open cards without losing their data.

### Changed
- **No network port.** The CLI and MCP reach the host over a local socket only your user can open (a Unix socket on macOS, a
  named pipe on Windows) instead of localhost TCP; `MORC_TRANSPORT=tcp` keeps the old way. Nothing on the machine's network stack is exposed.
- **The card owns its width.** Once laid out, a card's width is fixed: a long value wraps inside it instead of widening it (a chart that
  needs more room can still grow it). Text breaks inside its box rather than spilling out.
- **The card's buttons.** Pin, dismiss and the new layer button share one look, sit on an arc around the card's top-right corner, come
  out only when the pointer reaches that corner, keep one size whatever the card's zoom, and spring in and squeeze when pressed. Their
  look is a set of `--ctl-*` variables a preset can change.
- **Placement rects** use the card's visible plate (not the window around it), so snapping, `covers` and centring are exact.
- `morc card NAME --chip ... --set ...` (the card data file) is unchanged; `morc card <id> <op>` is the new built-in when `<op>` is one
  of get, set, hide, show, front, lower, raise, move, size.

### Fixed
- Replies from cards could be lost: reply ids larger than 2^53 were rounded by JavaScript and matched nothing.
- The spot recorded for a closing card was taken mid-animation; it is now taken when the close begins.
- After a restart, cards that were on screen were still reported as shown; they are now "closed" (bring them back with `show`).
