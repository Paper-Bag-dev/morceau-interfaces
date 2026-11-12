# src/surface: one card

| file | responsibility |
|---|---|
| `mod.rs` | the `Surface`: a borderless transparent window plus its webview (built here, or by the root page when cards are linked) |
| `root.rs` | the hidden root page that opens every card on macOS, so all cards share ONE web process and its state (docs/process-model.md) |
| `lifetime.rs` | how long a card lives: ttl, hover, interaction, pin, sticky, urgent |
| `actions.rs` | what a card's buttons do on the host side: run, open, emit, wake, reveal |
| `lint.rs` | the spec linter: no dead controls, answers must resolve, icon-only controls need a title, every number has a unit |