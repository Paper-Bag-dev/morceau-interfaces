# src: the Rust engine, one folder per responsibility

Every folder has its own README with a table of its files. A new concern gets a new file (or folder); nothing grows a second responsibility.

| folder | responsibility |
|---|---|
| `cli/` | the command-line client: parsing, talking to the host, plugins, stream, annoy |
| `host/` | the windows and the event loop: showing cards, layout, the page messages, the clock, crowd, view, UI data |
| `placement/` | where cards go: regions, the placement policy, scored fallback, the desktop (windows, monitors, fullscreen), remembered spots |
| `surface/` | one card: its window, lifetime, button actions, the spec linter |
| `agent/` | the background agent and what feeds it: wake (gates, persistent process), worker settings, inbox, triggers, live sources |
| `learning/` | what the host learns: the annoyance vector, engagement, chart literacy |
| `ipc/` | the CLI <-> host protocol: request types, the TCP server, the host's events |
| `data/` | where things live and are stored: paths, the LFU component cache, served page files |
| `platform/` | the Windows-specific parts (window styles, backdrop sampling, hotkeys, foreground watch); a macOS port replaces this folder |
| `util.rs` | a few small shared helpers |

Short module names used across the code (`crate::paths`, `crate::prefs`, `crate::wake` ...) are defined in `main.rs` from these folders.