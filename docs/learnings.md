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

## Content that is too vague
- Every metric gets a one-line plain-language caption (what is measured, over what, what good looks like). No internal names ("cold start").

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
