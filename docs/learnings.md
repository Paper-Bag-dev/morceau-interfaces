# Learnings (things we hit, and the rule each one became)

Short, so an agent or contributor can read it before touching the UI or the wake path.

## Cards that clip
- **Symptom:** a chart's white background (or its legend) is cut off at the right edge; worse on hover.
- **Cause:** the canvas was a fixed default width (280 px) with content drawn at fractions of it. Labels longer than the slot ran past the edge, and
  a canvas wider than its content left an empty strip that the window could cut off.
- **Rule:** a chart is exactly as wide as its content needs. Each chart module may export `width(s)` (donut: ring + legend column + padding; bar: a slot
  per bar wide enough for its label). `ui/components/chart/chart.js` calls it whenever the labels or the kind change, and the canvas takes that width. A kind
  without `width()` keeps the default. New chart kinds must export `width()` if their content can be wider than the default.
- **Never:** absolute positions for content, fractions of a fixed width for text, a default width that pads a card out.
- **Hover:** growth is a smooth scale from the old size to the new one, and the window is resized to the final size first, so nothing is cut off mid-way.
- **Check before calling a card done:** a long label, a narrow box, a hover. (Hover scaling was removed 11-11; cards keep one size.)
- **Arrival was invisible (11-11):** the page measured the card mid-arrival (scaled to 8%) and the window shape was cut to those tiny
  rects, so the whole spring was clipped and the card popped in. While any finite animation runs, the window shape stays the full
  window; the tight shape comes back once everything rests (`bridge.js` measure).
- **Also:** hover shadows must stay inside the 8 px window margin, or they clip on light cards.

## Metrics are a dashboard, not a story
- A vertical list of one thing after another is good for telling a story and bad for numbers the user wants to see at once. For metrics build a
  **dashboard**: a row of big stat tiles, then a row with charts side by side, one short muted line of explanation, one Close. The card may be wide
  (`max_frac` up to 0.85, `min` about [640,300]); rows wrap when the box is narrow.
- Tile labels do the explaining ("Re-used from cache", "New today"), so the caption can stay one line.
- **Type floors:** card text 15 px (small 14, large 18, hero 36), chart titles 15 px, chart text never below 13 px (`FLOOR` in `ui/charts/util.js`).
  The global size bias is a learned preference (`morc annoy nudge size_bias`); he asked for bigger text and cards on 11-08.
- **Two layout bugs that clipped charts:** `#surface` got a `max-width` from the wrap width and squeezed a rigid chart canvas, so it now has
  `min-width: min-content` (min beats max); and a chart in a row could flex-shrink below its canvas, so `.c-chart` is `flex: none` and rows wrap.

## Content that is too vague
- Every metric gets a one-line plain-language caption (what is measured, over what, what good looks like). No internal names ("cold start").

## Alignment follows the active window (11-09)
A fixed `align: right` in a card file was right when the active window was on the right (card on its left) and wrong when the window moved left: the card then sat on the right with everything right-aligned. The host now picks the alignment from the region it placed the card in (`place_named` -> `align` message): contents start at the edge nearest the active window, left-aligned by default. Card files should not set `align` (`morc card X --align auto`).

## Fullscreen apps and the thirds mode (11-08)
When the active window is a fullscreen app (a game) the host reports `screen-thirds`: no window to dock beside, so my side/corner counting does not apply, and cards landed in the thirds over the game, at inconsistent sizes. Rules now: the host holds every non-urgent push while a fullscreen app is active (`show.rs`); a card is sized to the room really left after the cards already on screen (`size_for` takes the current widgets), so it shrinks to fit a narrow side or third; the ladder's look step reads `mode` first.

## The ladder (what the overload test taught)
Pushing 12 full-size cards with no sums first piled them onto the terminal and into corners while a side still had room, and cards already placed moved when new ones arrived. So: count the room before you push, give things up in a fixed order (merge, trim data, shrink to the floor, remaining Tier 0, Tier 1 background windows only, hold back), push in small batches. Full ladder: docs/chips/INDEX.md. When space is short, merge same-kind cards and make the card expandable (compact by default, detail on interaction, others yield space, reset afterwards, expire when idle): rung 6b; only part of it is built (hover growth, prefetch, reveal).

## Sides first, merge first, animate (11-08)
Fill the four main sides (left, right, above, below) before any corner or mixed quadrant, and use a free top row before anything else; the host now orders candidate regions that way (`regions.rs`). Before opening a card, try merging it into one already on screen. Every size or position change animates with the default card motion (`setViewSmooth` in `bridge.js`, same ease as hover growth); a jump is a bug.

## The design mentality (the loop that works)
Look first (`morc free`). Biggest free area first, as a side, not a corner: rearrange or merge cards so they fit it; corners only after the sides are used. Group by kind: text as one flex row in a strip above/below the active window (`stat-strip`), charts merged in one window on a tall side (`chart-stack`). Start at the very edge of the quadrant next to the active window (card `dx`/`dy`). Compromise on size down to `min_importance` and on data (fewer bars), never on text size. Check the rects after pushing; if anything overlaps, change the card JSON and re-push. Full text: docs/chips/INDEX.md.

## Placement
- A remembered spot, or a region, that overlaps one of our own cards is rejected; the scored fallback treats overlap with own cards as costlier than
  covering any window (`mine_overlap` 5000). A new card goes to another side before it covers an existing one.

## The background agent
- **One agent, one process.** The first wake starts a headless `claude -p` that reads work from stdin (stream-json). Later wakes are written to the
  same process and queue behind what it is doing. It stays awake `linger_s` (10 minutes) after its last interaction, then the host closes its input.
- **It works alongside the interactive session** (`defer_to_session` false). A click with `bg:true` belongs to the agent; the session's hook only
  reports that it went there.
- **A user click is never rate-limited.** The limit (`max_per_hour`, `min_gap_s`) only guards wakes the host starts by itself (prefetch, triggers).
- **No turn cap by default** (`max_turns` 0). 8 turns was too few to build popups.
- **Permissions:** the rule `Bash(morc:*)` and `PowerShell(morc:*)` match the command text. A Windows path with backslashes never matched, so the agent
  calls plain `morc` with the exe folder on its PATH.
- **Hooks:** the agent runs with `MORC_HEADLESS=1` so its own hooks do not count as a live session or take the session's unread events.
- **Its output** goes to `%LOCALAPPDATA%\morc\agent.log`; its scratch files to `%LOCALAPPDATA%\morc\agent` (never the repo).
- **Prefetch:** a card with a show-more declares `opts.prefetch`; the host wakes the agent when the card opens so the extra popups are cached before the click.
- **Placeholders:** when the result opens in a new popup, open empty `skeleton-card` popups first under the ids the real cards will use, then build the
  real ones with the same ids; they replace the placeholders in place. Skeletons inside a card are only for content that lands in that same card.
- **Cards the agent opens expire** (short `ttl`, never sticky).

## Tooling
- **JSON files written by PowerShell carry a BOM.** The cache loader strips it; keep it that way.
- **`morc use` on a fragment** fails: fragments are built with `morc compose`. The error now says so and lists similar names.
- **Never write a literal `\n` into a file with a scripted replace.** Use the editor tool for multi-line edits, check `git diff -U0` for stray `\n`,
  and run `node --check` / `cargo build`.
- **Push `main` from a separate worktree** (`git worktree add <dir> main`), not by switching branches in the working folder; switching moves
  untracked personal files such as `prefs/` and can silently turn features off.

- **A trailing // comment inside a condition broke every card (11-11):** a comment typed mid-line swallowed the rest of an `if (...) {`; `node --check` still passed because it parses the file as a script. Check UI modules as modules: `node --input-type=module -e "import('file:///…/ui/core/bridge.js')"` (a `matchMedia is not defined` error means it parsed fine), and never put a comment in the middle of a condition.

## The workbench session (11-14): reuse first, reload right, never touch a game
- **Check what exists before building (cost me most of a day).** I wrote a placing command, a screen-geometry helper, a merge rule, a
  file-write engine path and hand-drawn flow curves; morc already had `morc card`/`push`, `morc card <id> set|move|size`, `morc tree`/`space`,
  the ladder's merge rung, `opts.persist` + `actions.json` (file in / out with fixed argv), `data-nodrag`, and a vendored Cytoscape. Rule:
  read every md here and run code-graph first; then vendored library -> npm package -> only then your own code. Never touch `src/` or
  `ui/core` for a feature without asking.
- **New components and open cards.** A card page (and the warm spare) loads `components.json` once. After `morc ui build` adds a type,
  open pages draw "unknown type"; `morc refresh <name>` re-imports the JS but does NOT add a new stylesheet, and other modules keep
  importing the old copy of a shared module (`../nav/nav.js` without the new export). New component or changed shared module = `morc stop`,
  then `morc card <id> show` for each card (they come back with their data) and `morc card <id> move` into place.
- **A laid-out card keeps its width.** `morc patch <id> /vars {...}` changes the tokens but the width stays locked; a same-id
  `morc use <chip> --id <id> --data @file` re-measures in place (no new window). Sizes per slot: chip `/vars` (`xp-w`, `ce-h`, `fm-w` ...).
- **Vendored bundles: pin by hash in the import URL** (`reactflow.bundle.js?v=<sha8>`): a page caches a failed module by URL forever.
  Build bundles from a script (`vendor/reactflow/build.mjs`), not a shell one-liner: PowerShell ate the quotes in esbuild's
  `--define:process.env.NODE_ENV="production"` and every card threw `production is not defined`.
- **Git Bash rewrites `/paths`.** `morc patch @shared /nav ...` from Git Bash wrote the key `C:/Program Files/Git/nav`. Prefix
  `MSYS_NO_PATHCONV=1`.
- **Clicks reach a session only on the next prompt.** An `emit` is just an inbox row; the hooks inject it when the user types. For a button that
  must answer now (expand), keep a Monitor on `%LOCALAPPDATA%/morc/inbox.jsonl` while a session is live, and use `wake` (not `emit`) so the
  background agent answers when no session is.
- **Layout like an app, in place, animated.** Slots from `morc tree` (work area, active window), sizes from `/vars`, then
  `morc card <id> move --at X,Y` (glides). Users liked it; close + show (a pop-in) reads as a glitch.
- **Never touch the screen while a game runs** (any exe under a configured games folder). Videos and the browser are fine. Hold the screen
  part until the game exits; hide (never show or move) cards that a job already put over it.

## 11-15: video, expand, music
- **The expand button needs someone listening.** It only wakes the agent, and a session reads the wake on its next prompt, so the button
  looked broken. `morc expand watch` (tools/cli/expand.py) answers expand / shrink at once; run it in the background for every session.
- **Shrink: a smaller content does not shrink a locked card.** The engine locks a card's width after layout, so putting `/vars` back left
  a big empty card. Set a new cap (`morc card <id> size --frac <the cap from before>`): a new size unlocks, re-measures and locks again;
  then move it back to where it was.
- **Grow the content, not just the cap.** A chip whose content has a fixed size (a 16:9 player) declares `opts.expand_vars` in its seed
  (video-card: `{"w": 1150}`); expand sets them, shrink clears them.
- **A component may own an element `el` refuses.** `el` never makes an iframe; the `video` component makes its own YouTube embed. That is
  a component, not an engine change.
- **Playback climbs the tree.** The video component writes `/playback` into its card's data (persisted, visible in `morc tree`), mirrors it
  to `@/media` for other cards, then tells the agent: playing / paused as `emit` rows (no wake), `ended` as the one wake.
- **CSS motion on a plate is clipped by the window.** The window shape is cut from the plate's rect, so a bounce or an outer glow on
  `.card-plate` barely shows. Glow inward (`inset` box-shadow) and move the WINDOW (`morc card <id> move`) for a real dance (`morc beat --dance`).
- **System audio is per OS.** Windows: WASAPI loopback (`pyaudiowpatch`). macOS has no system loopback: route output through BlackHole and
  read it with `sounddevice`. Without either, `morc beat --demo` is a synthetic beat and says so.
- **Flags are not names.** `morc ui new --help` created a component called `--help`; `ui new` now rejects anything but `[a-z][a-z0-9-]*`.
