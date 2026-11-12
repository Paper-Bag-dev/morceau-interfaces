# Cards as resources: hide, show, move, size, bring back

`morc card <id> <op>` is built in when `<op>` is one of get, set, hide, show, front, lower, raise, move, size; any other second word
(`--chip`, `--set`) is the older `card` plugin that writes a card's data file, unchanged.

Every card the host has shown is a record it keeps (live, hidden or closed), so "hide that for a bit", "bring it back", "make it big
and centred" are operations on a card that exists, never a second pop. Code: `src/host/cards.rs`.

| want | command | MCP `card` |
|---|---|---|
| what is there | `morc cards` | `{}` |
| one card's state | `morc card <id> get` | `{"card": id}` |
| put it away (page and data stay alive, clock stopped) | `morc card <id> hide` | `set: {"state": "hidden"}` |
| back (where it was; a closed card comes back from its record) | `morc card <id> show` | `set: {"state": "shown"}` |
| in front, no focus taken | `morc card <id> front` | `set: {"state": "shown"}` |
| move (visible content's top-left, physical px) | `morc card <id> move --at X,Y` | `set: {"at": [x, y]}` |
| centre on its display | `morc card <id> move --center` | `set: {"center": true}` |
| bigger / smaller cap, then centre | `morc card <id> size --frac 0.9 --center` | `set: {"frac": 0.9, "center": true}` |
| one level down the stack / back up (the card's third button, under the x, toggles it) | `morc card <id> lower` / `raise` (one step each) | `set: {"depth": n}` (0 = top), or `{"layer": "below" \| "top"}` |
| every card to the top / to the bottom of the stack | Ctrl+Option+Up / Down (Windows: Ctrl+Win+Up / Down, or Ctrl+Alt if taken) | |
| step through the stack by hand | hover the card, press Down (lower, as far as it goes) / Up (higher, to the top) | |
| pull onto the nearest edge from where it is | `morc card <id> move --snap` | |
| close | `morc close <id>` | `set: {"state": "closed"}` |

`set` takes the state you want and applies what it needs: `{"state": "shown", "frac": 0.9, "center": true}` shows, resizes, then centres
once the new size is measured. `frac` is a cap (fraction of the work area): content smaller than it stays its own size.

Answer of a read: `{"id","state": "shown|hidden|closed|opening|closing","layer": "top|below","rect": [x, y, w, h],"frac","pinned","chip","topic","at"}`.
`rect` is the card's visible PLATE (the page reports it), not the window around it with its buttons: snapping, `covers` and centring use it.
`depth` counts only windows that overlap the card (every step is a visible change); with none, the active window is level 1. The arrow
keys are taken only while the pointer is on a card (registered on hover, released when it leaves), so they reach your apps otherwise.
A lowered card stays where you put it (automatic placement never moves it out from under the window), gets no hover through the window
covering it, keeps its layer when hidden / shown / brought back, and `front` puts it on top again.
A hidden card is skipped by placement, expiry and the scene tree; a moved or resized card is marked as placed by you, so automatic
placement leaves it alone.

## On disk, with a falloff
- `<runtime>/cards.json` holds the records: state, spot, chip, topic, opts, spec, last data (the page sends its data as it leaves).
- Writes fall off: a change only marks the records dirty; the host writes once things are quiet for 1 s, and at most every 5 s while
  they never are. Spots are read from the windows at write time, never stored per frame: 40 moves in 4 s = 2 writes. Temp file + rename.
- The spot of a closing card is taken when the close begins, before the exit animation shrinks it.
- Closed cards are remembered up to 60 (newest first); live and hidden ones always.
- Same falloff elsewhere: page snapshots (`opts.persist`) 1 s quiet / 5 s max; action history appended in 0.5 s batches.

## Snapping (magnetic carry)
While you carry a card (hold it, or hold the pin), the host moves it itself and pulls its plate onto nearby edges: beside the active
window or another card (keeping the `snap-gap` seam, 12 px), above / below one, lined up with its top / bottom or left / right once docked,
and the screen's inner edges. A snapped axis slides along the edge as you move along it and lets go when pulled past `snap-break-px`.
A fast throw (pointer faster than `snap-max-speed`) never snaps: moving a card to the other end of the screen is not caught by every edge
on the way; it snaps once you slow down where you are going. Settings in `ui/limits.json` (hot-reloaded, css px, scaled per display):
`snap-px` 15 (0 = off: the OS drag instead), `snap-break-px` 30, `snap-gap` 12, `snap-max-speed` 900 px/s, `snap-frame-ms` 8. Code: `src/host/snap.rs`
(the magnets are a pure function with tests), followed in `host/tick.rs follow_grabs`. Same on Windows and macOS (the pointer comes from
`platform/<os>/sys.rs cursor`).
