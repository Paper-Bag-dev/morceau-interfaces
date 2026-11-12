# Trees: what is on screen, without a screenshot

Two kinds of tree, each owned by one side and written only there; a merged view for reading both at once.

| tree | owner | what it holds | read it |
|---|---|---|---|
| scene | host (`src/host/tree.rs`) | screen → displays → windows front to back → cards as **mount** nodes; `covers` edges | `morc tree` |
| card | the card's page (`ui/core/tree.js`) | its components: id, type, rect, flags, text; its data keys, shared keys, named actions | `morc tree <card>` |
| merged | read-only | the scene with each card's tree under its mount node (`tree`) | `morc tree --all [--depth N]` |

Options: `--depth N` cuts a card tree at depth N (`more: <count>` marks what was left out); `--node <id>` starts a card tree at that component.
MCP: the `scene` and `card_tree` tools (docs/mcp.md).

## Scene tree
```json
{"id":"screen","units":"physical px, [x, y, w, h], z 0 = front","children":[
  {"id":"display:0","kind":"display","rect":[0,0,2940,1912],"work":[...],"children":[
    {"id":"card:tasks","kind":"card","card":"tasks","chip":"","topic":"adhoc","mount":true,"z":0,"rect":[2020,818,570,548]},
    {"id":"win:68","kind":"window","app":"Google Chrome","z":3,"rect":[0,66,2940,1846]}]}],
 "covers":[{"above":"card:tasks","below":"win:68","px":348992}]}
```
- Built fresh on every call (no live model to keep in sync). Windows come from the OS front to back (`platform/<os>/desktop.rs scene_now`).
- A card's rect is its **visible content**, not the see-through window around it, so `covers` between cards means they really overlap.
- `covers` lists only pairs with a card on one side (window-on-window overlap is the user's business). Acyclic by construction (front to back).
- App names only (macOS: the owning app; Windows: the exe, e.g. `chrome.exe`); window titles are left out (they can name clients).
  Both platforms give full display frames + work areas and put our cards in the real stacking order.

## Card tree
```json
{"card":"tasks","size":[260,287],"zoom":1,"data":{"items":"array(4)"},"shared":["focus"],"actions":["add","focus","remove"],
 "root":{"id":"root","type":"column","rect":[23,34,239,230],"children":[
   {"id":"l","type":"list","rect":[38,125,209,126],"flags":["clipped","cut"],
    "issues":[{"flag":"cut","text":"A very long task title that will certainly not fit on one..."}]}]}}
```
Rects are css px inside the card. **Flags** are what a screenshot would have shown:

| flag | means | the card rule it breaks |
|---|---|---|
| `cut` | text cut off sideways (ellipsis / overflow hidden) | everything reachable |
| `clipped` | content hidden below with no way to scroll to it | everything reachable |
| `scrolls` | content continues, reachable by scrolling | fine (rule 2 allows it) |
| `folded` | a compact card resting small hides rows; it opens on hover | fine (rule 3: rests small when unused) |
| `offcard` | sticks out of the card window | everything reachable |
| `hidden` | not shown right now (a `when`, display none) | none: state |
| `empty` | a leaf that shows nothing (no text, picture or painted background) | every row holds real content |
| `not-rendered` / `missing` | in the spec but not in the DOM / a child id with no component | a spec bug |

So the check after a push is `morc tree <card>` and a look at the flags; a screenshot is for looks only (colors, contrast, a new component).

## How a query runs
The host sends `{"t":"tree","rid"}` to each card asked; each page answers `{"t":"reply","rid","body"}`. The host answers the CLI when every
card has answered, or after 1.2 s with what arrived (a silent card shows `"tree": "no reply"`). Reply ids stay under 2^52: a JS number
holds 53 bits exactly, and a bigger id came back rounded and matched nothing.
