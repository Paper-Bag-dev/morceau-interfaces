# Morceau docs

| file | read it for |
|---|---|
| `../README.md` | what it is, install, full setup, tour, troubleshooting |
| `protocol.md` | the localhost protocol and the spec format (components, data, `sources`, actions) |
| `extending.md` | adding components, plugins, fragments, recipes, models, CLI plugins |
| `agent.md` | the system prompt for the screen and chore agent (replace it in your data folder to add your own rules) |
| `chore-prompt.md` | the wake prompt the host gives the headless chore agent (placeholders `{reason} {note} {activity} {cli}`) |
| `design.md` | the design tokens and the reasoning (one accent, type floors, what is deliberately absent) |
| `learnings.md` | what went wrong and the rule each thing became: clipping and chart width, placement, the background agent, tooling |
| `charts.md` | choosing a chart by the question; the chart kinds and props |
| `graphs.md` | the node-link graph pipeline: exporter schema, code-graph source, layers and routes |
| `phases.md` | what exists and what is planned |
| `process-model.md` | linked cards: one web process for every card, the hidden root page, the switches |
| `cards.md` | cards as resources: hide, show, move, size, bring back; how records reach disk (the write falloff) |
| `trees.md` | the scene tree, card trees and the merged view; the flags that replace a screenshot check |
| `state.md` | card store, shared store, named actions, history and snapshots |
| `views.md` | Preact components, Tailwind classes, the look preset, the layout rules |
| `plugins.md` | plugins (yours live outside the repo) and reloading them in place |
| `mcp.md` | `morc mcp`: the six MCP tools |
| `../AGENTS.md`, `../CLAUDE.md` | how any coding agent should work on the repository |
| `../BACKLOG.md` | what is next |

## How a surface lives
1. `morc show` / `use` -> the host claims a hidden warm window, the page renders, reports its size and clickable shapes.
2. The host sizes the window to the content, clips it to the shapes (the rest is click-through), picks a spot (regions, then scored placement), shows it without
   taking focus, and the page springs in.
3. It stays about 15 s with no interaction (hover pauses; interaction extends; pin holds; sticky skips the timer), then fades out. Ignored, glanced or engaged is
   logged. Urgent surfaces are ringed and sticky; if untouched they are parked, not lost: `morc summon`.
4. You can hold the left button ~0.1 s on it to pick it up and carry it; the spot is remembered per topic.

## Map of the engine (`src/`)
| file | job |
|---|---|
| `main.rs`, `client.rs`, `server.rs`, `protocol.rs` | entry; the CLI; the localhost server; the request and option types |
| `host.rs`, `surface.rs`, `lifetime.rs` | the long-lived host and its event loop; one WebView window; expiry and engagement rules |
| `placement.rs`, `regions.rs`, `policy.rs` | scored placement; the free-region model and alignment; the data-driven placement policy |
| `src/platform/<os>/` (`window_style`, `foreground_watch`, `hotkeys`, `backdrop`) | window shape and style; active-window events; global hotkeys; backdrop sampling |
| `lint.rs`, `handlers.rs`, `actions` | the spec linter; the action allowlist runner |
| `cache.rs`, `prefs.rs`, `placed.rs`, `engagement.rs`, `literacy.rs` | cached components; the preference vector; remembered spots; behaviour log; what the user has learned |
| `anticipate.rs`, `sources.rs`, `worker.rs`, `wake.rs`, `inbox.rs` | triggers and wake flags; live data sources; the chore agent settings and gates; waking an agent; the event inbox |
| `assets.rs`, `paths.rs`, `util.rs` | serving `ui/`; install / data / runtime folders; helpers |
