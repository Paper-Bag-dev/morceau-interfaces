# UI chips: read this first

A **chip** is a ready-made card: a JSON file (the layout, already built) plus a short md next to this index. You do not write component code.

**STRICT RULE: every component is modular and controlled by a JSON file.** Whenever you make or change any component or chip: its layout is one JSON
in `cache/seed/`, and everything that can vary (data, `align` left/right, `importance`, `min_importance`, `region`, `ttl`, `chip`) is a field in the card's
JSON (`cards/<name>.json`, written with `morc card`). Nothing about one component is hardcoded in Python or Rust. A new knob is a new field read
generically for every card, never a branch for one chip. Fields not set fall back to defaults; you only write what differs. A new component is done only
when it has: its layout JSON, a short md in `docs/chips/`, a line in this index, and works through `morc card` + `morc push`.

**Group by kind (the user has said this many times):** numbers and text go together in ONE card (`stat-tiles`); pictures (charts) each go in their own
separate card (`donut-card`, `week-chart`). Never mix a picture and a text list in one card, never stack everything as one long vertical story, never
one big blocker. Several small cards spread into the free space (`--region auto`), sized in between (about importance 1.0; text cards 0.85 to 1.0).

**The design mentality (confirmed working, 11-08; follow it for every push):**
- **Look first.** `morc free` shows the free regions around the active window (the centre of the 3x3 grid): the four sides and the four corners.
- **Biggest free area first, as a side, not a corner.** Fill the biggest free side before any corner. Do not let cards fall into a corner while a side still has
  room: rearrange (merge) the cards so they fit the side properly. Corners come only after the sides are used.
- **Group by kind, then fit the shape of the space.** Text/numbers go as ONE flex row in a long strip above or below the window (`stat-strip`). Pictures (charts)
  may be merged into ONE window on a tall side (`chart-stack`), even when they are different charts. A tall side takes a column, a strip takes a row.
- **Start at the very edge of the quadrant,** next to the active window (hug it), then fill outward. Use the card's `dx`/`dy` to sit flush with the edge.
- **Size is what you compromise, never text.** Shrink a card down to its `min_importance` (text stays at the floor) before moving it to a worse place;
  trim data (5 bars instead of 7) before shrinking text.
- **Check after pushing:** `morc status` rects and a look at the screen: nothing overlaps the active window or another card. If it does, change the card JSON
  (chip, region, importance, dx/dy, fewer bars), re-push, and note what worked here.

**The loop (whenever you or the background agent push chips):**
0. `morc free` and decide which cards go in which region (rows for strips, merged charts for a side).
1. Decide every card you want to show and write each card once with `morc card <name> --chip <chip> --set key=value ...` (one short command per card; it is
   saved in `cards/<name>.json` and remembered; see [cards.md](cards.md)). Later change only what changed: `morc card <name> --set a.value=820`.
2. Next turn show them all in one command: `morc push <name> <name> ...` or `morc push --all --prefix tok-`. Do not pick positions yourself; the host does
   it with the Tier 0 rule below.
3. Do not edit scripts or code to tweak a card, and do not regenerate a big JSON. Each card keeps a size floor (`min_importance`, default 0.85) that you do
   not go under unless the user points it out: text, icon and component size are never what you compromise on.

**Tier 0 placement (how the host fills free space, in the 3x3 grid with the active window as the centre cell):**
- Look for free space beside the active window (free of every other window and card). Active window on the right and the left free: fill the left, starting
  at the edge **next to the active window** and working outward. Once that is full, take the next free cell and start again at its corner **closest to the
  active window** (bottom-left free: its top-right corner; top-left: bottom-right; top-right: bottom-left; bottom-right: top-left). Active window below:
  start from the top (top-left or top-right) and so on, for every free quadrant.
- A big background window never blocks this: the free sides of the ACTIVE window count as free.
- If Tier 0 cannot be followed, populate right (or the scored fallback). If a card can never fit any free space, start from the right.
- Compromise on **size**, never on text: cards shrink to fit the free space, but text stays at or above the type floor (15 px card text, 13 px chart text).
  If it would go below that, the card goes elsewhere instead of shrinking further.

**To show something:**
1. Find the chip below. Open its md (one screen) for the params and the size knobs.
2. Change only its **data** (a small JSON file with the params) and, if needed, a size knob. The layout JSON stays untouched.
3. Push it: `morc use <chip> --data @file.json` (add `--importance 0.7` to make it smaller, `--importance 1.2` bigger).
4. If no chip fits, make one: write its layout JSON in `cache/seed/`, a short md here, and one line below. Do not write a one-off spec.

| chip | use it for | params (data) | md |
|---|---|---|---|
| `tokens-dashboard` | Claude token usage: `morc tokens` pushes SEVERAL SMALL cards into free space (default); the one big card only with `--one-card` | total, today, reuse, replies, models, days | [tokens-dashboard.md](tokens-dashboard.md) |
| `donut-card` | a share-of-a-whole donut | title, values, labels | [donut-card.md](donut-card.md) |
| `chart-stack` | two charts in ONE window (donut over bars) for a tall free side | pie, bars | [chart-stack.md](chart-stack.md) |
| `stat-strip` | four numbers in ONE flex row for the strip above or below the active window | a, b, c, d | [stat-strip.md](stat-strip.md) |
| `stat-card-min` | one big number with a label and unit | h.label, h.value, h.unit | (data only: `{"h": {"label": "New today", "value": 4.4, "unit": "M"}}`) |
| `week-chart` | any 7-point bar or line chart | title, values, labels, unit | [week-chart.md](week-chart.md) |
| `ask-permission` | an urgent yes/no for the user | title, body, yes, no, ctx | [ask-permission.md](ask-permission.md) |
| `graph-view` | a node-link graph (code, goals, flows) | title, graph | [graph-view.md](graph-view.md) |
| `bike-viewer` | the interactive 3D bike demo | title, paint, spin, explode | [bike-viewer.md](bike-viewer.md) |
| `skeleton-card` | an empty shimmering placeholder popup, shown under the id the real card will use | none | [skeleton-card.md](skeleton-card.md) |

**Rules that apply to every chip** (details in `../design.md` and `../learnings.md`): never clip, size within the container, text at least 15 px, explain each
metric in a line, one card per kind of thing (do not stack unlike things in one card), short `ttl`, never cover your own cards.
