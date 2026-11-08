# Learnings (things we hit, and the rule each one became)

Short, so an agent or contributor can read it before touching the UI or the wake path.

## Cards that clip
- **Symptom:** a chart's white background (or its legend) is cut off at the right edge; worse on hover.
- **Cause:** the canvas was a fixed default width (280 px) with content drawn at fractions of it. Labels longer than the slot ran past the edge, and
  a canvas wider than its content left an empty strip that the window could cut off.
- **Rule:** a chart is exactly as wide as its content needs. Each chart module may export `width(s)` (donut: ring + legend column + padding; bar: a slot
  per bar wide enough for its label). `ui/components/chart.js` calls it whenever the labels or the kind change, and the canvas takes that width. A kind
  without `width()` keeps the default. New chart kinds must export `width()` if their content can be wider than the default.
- **Never:** absolute positions for content, fractions of a fixed width for text, a default width that pads a card out.
- **Hover:** growth is a smooth scale from the old size to the new one, and the window is resized to the final size first, so nothing is cut off mid-way.
- **Check before calling a card done:** a long label, a narrow box, a hover. Hover can be played without touching the mouse:
  `morc patch <id> /__hover true` (and `false`).
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
