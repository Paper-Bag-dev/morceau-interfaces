# src/host: the host, one responsibility per file

The host owns the windows and the event loop. It used to be one 1000-line `host.rs`; each concern is its own module now (all `impl Host` blocks share the
`Host` struct in `mod.rs`).

| file | responsibility |
|---|---|
| `mod.rs` | the `Host` struct, `run()` (startup, hot-reload watcher, event loop), the request dispatcher `cmd`, window claim/spawn/destroy, summon, replace |
| `show.rs` | showing a card: lint, the fullscreen hold, the annoyance vector, refresh in place or a new window |
| `layout.rs` | a card reported its size: resize the window, hit shape, placement (regions first, scored fallback) |
| `ipc.rs` | messages from the card pages (hover, drag, act, pin, close ...) |
| `tick.rs` | the clock: lifetimes, parked cards, drag bookkeeping, placement nudges |
| `crowd.rs` | crowded screen: the other cards yield room while one is in use |
| `view.rs` | how a card is drawn and sized: scale, size limits (`view_for`), cohesion weights, importance, topic scope |
| `uidata.rs` | the UI as data: tokens, limits, themes, the component map; cached in memory, re-sent when a file changes |
| `tree.rs` | the scene tree (displays, windows front to back, cards as mounts, `covers`) and page answers: card trees, named actions (docs/trees.md) |
| `cards.rs` | cards as resources: hide / show / front / move / size / get / set, the records and their write falloff (docs/cards.md) |
| `state.rs` | `opts.persist`: snapshots and the action history on disk (docs/state.md) |

Rule: a new concern gets a new file here (and a row above); never grow `mod.rs` with a second responsibility.
