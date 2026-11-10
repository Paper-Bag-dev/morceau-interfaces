# UI chips: read this first

A **chip** is a ready-made card: a JSON file (the layout, already built) plus a short md next to this index. You do not write component code.

**STRICT RULE: no visual size is hardcoded in CSS or code.** Every size that is a choice (min widths, radii, type sizes, paddings, text floors) is a token in
`ui/tokens.json` (overridable in your `prefs/ui.json`), sent to each card as a CSS variable and read as `var(--name, fallback)` / `floor()`. Need a new size? Add
a token and read it; never type the number into `theme.css` or a chart file. Chart heights and widths are fields of the chip's JSON.
**Plate by default:** every card's root layout sits on one frosted plate (`.card-plate`); do not add your own background. `"plate": false` on the root only when a card must float (a 3D model, a bare badge).

**Where the UI rules live (each in its own small JSON, hot-reloaded):** `ui/tokens/<component>.json` (chart, text, plate ...: sizes, radii, text floors, as CSS variables), `ui/limits.json` (min and max sizes and timings: leave alone unless the user points it out or annoyance says so), `ui/themes.json` (accents, text and surface colors, light/dark schemes). Override any of them in your data folder (`prefs/tokens/<component>.json`, `prefs/ui.json`). The host caches them in memory and re-sends them to every card when a file changes. One file per component, never one big file.

**STRICT RULE: every component is modular and controlled by a JSON file.** Whenever you make or change any component or chip: its layout is one JSON
in `cache/seed/`, and everything that can vary (data, `align` left/right, `importance`, `min_importance`, `region`, `ttl`, `chip`) is a field in the card's
JSON (`cards/<name>.json`, written with `morc card`). Nothing about one component is hardcoded in Python or Rust. A new knob is a new field read
generically for every card, never a branch for one chip. Fields not set fall back to defaults; you only write what differs. A new component is done only
when it has: its layout JSON, a short md in `docs/chips/`, a line in this index, and works through `morc card` + `morc push`.

**Alignment follows the active window (default):** a card's contents start at the edge NEAREST the active window. A card to the right of it, above it or below it is left-aligned (flex from left to right); a card to the left of it is right-aligned (it hugs the window). The host decides this when it places the card and tells the page (`align` message); you do not set it. Only an explicit `--align left|right` in the card file overrides it (`--align auto` removes the override).

**Group by kind (the user has said this many times):** numbers and text go together in ONE card (`stat-tiles`); pictures (charts) each go in their own
separate card (`donut-card`, `week-chart`). Never mix a picture and a text list in one card, never stack everything as one long vertical story, never
one big blocker. Several small cards spread into the free space (`--region auto`), sized in between (about importance 1.0; text cards 0.85 to 1.0).

## THE LADDER (always follow it, in this order, every time you put cards on screen)
1. **Need.** List the cards you want and their kind: text/numbers or a picture (chart). Rank them: what matters most goes first.
2. **Look.** `morc space` (or `morc free`): the `mode` and the free regions with their sizes. `mode` is `around-active` (the active window is the centre of the 3x3 grid and the free sides and corners are around it) or `screen-thirds` (a FULLSCREEN app is active: no window to dock beside, the screen is split into left, mid and right thirds). In `screen-thirds` the host HOLDS every push that is not urgent or `--force` (`morc push` prints `held <name>: ...`); report what is waiting and push when the fullscreen app is gone. Never push over a game or a video.
3. **Count BEFORE you push.** (A card is sized to the room that is really left after the cards already on screen, a side or a third, and shrinks down to its floor to fit it.) For each card take its size at `min_importance` (text at the floor). Add them up against the free regions. Never open more
   than fits; never push first and look after.
4. **Group by kind.** Text becomes one flex row (`stat-strip`) for a strip above/below; charts become one merged window (`chart-stack`) for a tall side.
5. **Assign.** Always fill the FOUR MAIN SIDES first (left, right, above, below; resize and merge cards to fit them) and only then expand into the corners
   or mixed quadrants. Never start in a corner. Fill every side before reaching for a corner (a free top row must be used before anything goes elsewhere).
   Before opening a new card ask: can it be merged into a card already on screen (a donut into the older chart window, a fact into an existing row)? Merge first.
   Biggest free side first (corners only after the sides are used). Each card starts at the very edge of its quadrant next to the
   **Why cards leak to corners:** a chip whose min size (times the display scale, plus margin and gap) is wider than the side it is aimed at does not fit that side, so the host falls back to scored placement and the card lands in a corner. Keep each chip's min and its charts narrow enough for the narrowest side (about 320 px of room on a 344 px side), and keep chart minimum widths small. Text facts are SEPARATE small cards: in the top strip they flex from the left while the numbers row flexes from the right.
   active window (`dx`/`dy`), then fills outward. Tier 0 = free of every window and card.
6. **Not enough room? Give up things in THIS order:** (a) merge cards of the same kind; (b) trim data (5 bars, not 7); (c) shrink the card down to its
   `min_importance` (never the text floor, never icon or component minimums); (d) use the remaining Tier 0 spaces, corners included; (e) Tier 1: the free
   sides of the active window, covering only background windows, never the active window and never one of our own cards; (f) hold the rest back: open
   fewer cards and say what is waiting (a "more" button) instead of overlapping.
   **Merge rule (once):** cards of the same kind are merged into one card whenever there is not room for them apart (charts into one window, numbers into one row).
6b. **Crowded or short on space: update the cards you already have, with commands (no coding).**
   1. `morc status` lists the cards on screen (id, rect, visible).
   2. Make the one in use bigger and the rest smaller: `morc card <id> --importance X` for each (never below its `min_importance`), then ONE command re-pushes
      any number of them in place: `morc push a b c d` (same id = refreshed where it stands, so cards shrink away and the active one takes the room).
   3. When the interaction is over, set the importances back with `morc card` and `morc push` the same list. A card nobody touches for too long expires by
      its `ttl`; close the rest with `morc close <id>`.
   4. The host also does the shrink-and-restore by itself when the screen is crowded. Crowded is NOT a card count: it is how much of the free room (the work area
      minus the active window) our cards already cover, compared with the learned `crowd` limit in the annoyance vector (default 0.5). While you hover one card the
      others shrink to `crowd_yield` (0.8 in `prefs/placement.json`, never below the zoom floor) and return when the pointer leaves. Teach it what you treat as
      crowded: `morc annoy nudge crowd 0.8 --why "too crowded"` lowers the limit, `morc annoy nudge crowd 1.2 --why "fine"` raises it (per topic or global).
   Compact by default with the detail behind it ("show more" via prefetch + `reveal`) is how a card is made expandable.
6c. **Tier 1 means you must readjust the OLDER cards, not just the new one.** When the free sides are used up, re-pack what is already on screen first: merge
   same-kind cards, trim data, shrink to `min_importance`, move them (`morc card` + `morc push` the whole list), animated (rung 9). Only after that is
   there room, or is it clear there is none.
6d. **No room even after re-packing: you MAY put the card on top of the active window,** but it is temporary: give it a SHORT `ttl` (about 20 s), keep it
   small, never on top of one of our own cards, and let it go when the user looks away. It is the last step, not a habit.
6e. **Attention costs annoyance.** The learned `crowd` limit and the user's patience are budgets. Once they are used up, do not open another card: send the
   blue pulse on an existing relevant card (`morc patch <id> /__pulse true`) or hold the item and pulse later, so the user chooses when to look. Long things
   (a paper) wait behind that nudge. Live status from other apps (downloads, an update) goes in ONE `progress-card`, updated with `morc patch`, not a new card each time.
7. **Write, then push in small batches.** `morc card` for each, then `morc push` three or four at a time. Slowly: re-count after each batch.
8. **Verify with numbers.** `morc status` rects: no overlap with the active window or another card. Look at the screen only when the user has asked.
9. **Motion shows intent.** Every size or position change a card makes (shrinking to give another card room, growing on interaction, moving to make space)
   is animated with the default card motion (the ease used for hover growth: scale over about 220 ms), never an instant jump. Shrink eases down first and then the
   window follows; grow makes the window big first and eases up to it. A different animation is allowed only if it says something else on purpose.
10. **Fix in JSON.** If something is off, change the card's file (chip, region, importance, dx/dy, data), re-push, and write down what worked.
Skipping a rung is how we got 12 oversized cards piled on the terminal. Do not skip rungs.

**DEFAULT PLACEMENT RULE (user, 11-09):** put stat and progress cards in the TOP strip (`--region above`) as ONE flex row (`stat-strip`, or `stat-line` for thin strips), expanding along that row. Do not put them on the right or below by default: they clip into the terminal. Only tall things (charts, `chart-stack`, long lists) go on a side. Use the bottom strip (`status-row`) only after the top strip is full. Never `--force` a card into a place that is not Tier 0 (free of every window and card); if there is no Tier 0 room, hold it and say so.

**PLACE ONCE (user, 11-09, strict): never iterate a card's placement.** Get the free regions (`morc free`) and the card sizes from live numbers, work out the size and position, push ONCE, and stop. No loop of push, measure, nudge, push. If the user then drags or corrects the card, that drag is the feedback: read the `[morc]` drag event (`from`, `to`, `screen_zone`), write the offset into this file (below) and into the topic's card JSON (`dx/dy/importance`), and use it on the next push of that topic. Count first, with the numbers already known (region rect, the card's `min` size in its chip JSON, importance), so the first push fits. Do not claim a fit you did not measure once after the single push; report the rect and wait.
- Facts that hold on any screen: explicit `--dx/--dy` replace the auto margin and only apply on a fresh push after `morc close <id>` (a re-push of the same id ignores them); pushing a new card can re-flow the cards already open, so push ONE card per command and do not re-push a card that must stay put; `stat-strip` needs `--importance 0.6` to fit a strip about 157 px high.

**SHOW A MIX, NOT JUST TEXT (user, 11-09, strict).** Whenever you show the user data (progress, stats, plans, reports) make it a mix of pictures and text: a chart card wherever the data can be a chart (bars for per-item numbers, donut for a share, line for a trend, progress bars for goals; charts are cheap to make now) next to the text numbers in one `stat-strip`. Never only text. Make every card self-explanatory by itself: clear title, unit and label on every number, so it reads without a sentence. Add an explanation line ONLY when the thing is not obvious from the card; most of the time there should be none.

**MIX AND MERGE BY DEFAULT (user, 11-09, strict, for every session and every agent).** (1) Mix: every report is charts plus text, never one kind alone. (2) Merge with what is already on screen before opening anything new: run `morc status` first; if a card of the same kind or topic is open, update it in place (`morc patch <id> /path value`, or re-push the same id with new data) or fold the new numbers into it; open a new card only when nothing open can take it. Text facts join the open `stat-strip`/`stat-tiles`, a new series joins the open chart, a new to-do joins the open checklist. (3) Prefer live data: bind cards to a JSON file with `sources` (file, pointer, every) so they update themselves and stay in sync with the stored data, instead of static pushes that go stale. (4) Anything that is a list of things to do is a tickable checklist (`checklist` component, one `tick-<id>` action per item that writes its state to JSON), never a static list.

**HARD RULE, FOLLOW THE LADDER BEFORE EVERY PUSH, AND VERIFY.** Before pushing any card run `morc status` and `morc free`, read the free rects and the open cards' rects, pick the region and size from those numbers in the ladder's order, push ONCE, then check `morc status` for `visible: true` and a rect inside free space. Never say you did not look at the screen or hand the checking back to the user: look, and if a push is held (fullscreen) say so and wait. The user must never have to readjust a card; his drags are only the learning signal (a region name can land somewhere other than you expect, so read the rect after the push, and remember a drag as the preferred spot for that topic). Load this rule into every session (global instructions, the morc-screen skill, memory) so a new session reads it before its first push.

**LOAD THE FRONTEND-DESIGN PLUGIN BY DEFAULT (user, 11-09).** Before designing or changing any card, component or chip, load the `frontend-design:frontend-design` skill and follow it (a deliberate palette and type scale, one memorable element, structure that carries information, motion only when it answers an action) together with docs/design.md. The user called a static, plain card unacceptable.

**The design mentality (confirmed working, 11-08; follow it for every push):**
- **Look first.** `morc free` shows the free regions around the active window (the centre of the 3x3 grid): the four sides and the four corners.
- **Biggest free area first, as a side, not a corner.** Fill the biggest free side before any corner. Do not let cards fall into a corner while a side still has
  room: rearrange (merge) the cards so they fit the side properly. Corners come only after the sides are used.
- **Group by kind, then fit the shape of the space.** Text/numbers go as ONE flex row in a long strip above or below the window (`stat-strip`). Pictures (charts)
  may be merged into ONE window on a tall side (`chart-stack`), even when they are different charts. A tall side takes a column, a strip takes a row.
- **Start at the very edge of the quadrant,** next to the active window (hug it), then fill outward. `morc push` makes a card with `align: left|right` sit flush with that edge on its own (dx = minus or plus the margin from `prefs/placement.json`); an explicit `dx` in the card file wins.
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
| `stat-line` | up to six short text facts in ONE low row, for the THIN strips (about 100 px high) | t1 .. t6 (strings) | [stat-line.md](stat-line.md) |
| `progress-card` | loading / download status: up to three live progress bars (update with `morc patch <id> /a/value N`) | title, a, b, c | [progress-card.md](progress-card.md) |
| `todo-list` | a TICKABLE to-do list (component `checklist`): count, hairline fill, round checkboxes, due rings; ticks write JSON and tick Todoist | title, items [{id,text,sub,badge,done}], `sources` for live state | [todo-list.md](todo-list.md) |
| `status-row` | a thin row for the strip above/below: one spinner + up to two live progress bars, no buttons | s, a, b | [status-row.md](status-row.md) |
| `stat-card-min` | one big number with a label and unit | h.label, h.value, h.unit | (data only: `{"h": {"label": "New today", "value": 4.4, "unit": "M"}}`) |
| `week-chart` | any 7-point bar or line chart | title, values, labels, unit | [week-chart.md](week-chart.md) |
| `ask-permission` | an urgent yes/no for the user | title, body, yes, no, ctx | [ask-permission.md](ask-permission.md) |
| `confirm-run` | ask once, then the button runs a staged command itself (no model wakes) | title, body, run (staged id), yes, no, ctx | [confirm-run.md](confirm-run.md) |
| `ask-many` | several yes/no questions on one card, one Send, one wake | title, body, ctx, items [{id, q}] | [ask-many.md](ask-many.md) |
| `paste-blocks` | paste blocks to hand over in order, each with a Copy button | title, body, items [{title, text, note}] | [paste-blocks.md](paste-blocks.md) |
| `graph-view` | a node-link graph (code, goals, flows) | title, graph | [graph-view.md](graph-view.md) |
| `bike-viewer` | the interactive 3D bike demo | title, paint, spin, explode | [bike-viewer.md](bike-viewer.md) |
| `skeleton-card` | an empty shimmering placeholder popup, shown under the id the real card will use | none | [skeleton-card.md](skeleton-card.md) |

**Rules that apply to every chip** (details in `../design.md` and `../learnings.md`): never clip, size within the container, text at least 15 px, explain each
metric in a line, one card per kind of thing (do not stack unlike things in one card), short `ttl`, never cover your own cards.

