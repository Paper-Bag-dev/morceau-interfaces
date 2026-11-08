# Choosing a chart (pick by the question, not the habit)

| question | kind | data | notes |
|---|---|---|---|
| how did it change over time | `line` / `area` | numbers | draws itself left to right |
| a tiny trend inside a row of text | `spark` | numbers | no grid |
| compare a few things | `bar` | numbers + labels | bars pop up in turn |
| what share of the whole | `donut` | numbers + labels | slices sweep in, hover pushes out |
| one number vs its limit | `gauge` | single number, `max`, `warnAt`, `dangerAt` | needle overshoots, color goes green/amber/red |
| how balanced across 3+ measures | `radar` (5 = pentagon) | numbers + labels, `max` | blooms from center |
| a grid of intensities (days, slots) | `heatmap` | numbers, `cols` | diagonal ripple |
| when in the day does it happen | `clock` | 24 numbers (hour 0-23) | radial wedges, current hour outlined |
| several goals, each with a target | `rings` | numbers, `targets` | fitness-ring style, second lap = overshoot |
| big names, share of total | `treemap` | numbers + labels | box area = share |

Props: `title`, `data` (bind), `labels` (bind), `unit`, `max`, `targets`, `cols`, `height`, `width` (px, default 280).

## Teaching the unfamiliar ones
`radar`, `heatmap`, `clock`, `rings`, `treemap` show a one-line "how to read this" the first times (`charts/explain.js`).
- **Got it** stops it. After 2 showings it stops anyway.
- **Still confusing** swaps in a plain bar/line chart AND writes a `friction` event to the agent inbox.
- Agent rule: on friction, re-explain differently on screen (one new sentence, with an example from the user's own numbers), then
  prefer a simpler kind for that user until they ask for it. Edit the wording in `explain.js` if a phrasing keeps failing.
- State: `%LOCALAPPDATA%\morc\literacy.json` (`{kind: {seen, got, confused}}`).

Adding a kind = one file in `ui/charts/` exporting `draw(g, s)` and `hit(s, x, y)`, one line in `ui/components/chart/chart.js` (MODS), and
an entry in `explain.js` if it needs teaching. Planned fancy ones: streamgraph, sankey (where time flows), bullet, slope, calendar streaks.
