# src/agent: the background agent and what feeds it

| file | responsibility |
|---|---|
| `wake/` | waking the headless agent: `gate.rs` (the gates), `process.rs` (the one persistent process and its queue), `mod.rs` (the prompts, the way in) |
| `worker.rs` | its settings (`prefs/worker.json`): model, linger, idle, rate limit; activity and title tracking |
| `inbox.rs` | the event inbox the Claude Code hooks read |
| `triggers.rs` | rules that show things before you ask (`triggers/*.json`), wake flags |
| `live_sources.rs` | live data sources (numbers read from files for cards) |