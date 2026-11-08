# src/surface: one card

| file | responsibility |
|---|---|
| `mod.rs` | the `Surface`: a borderless transparent window plus its webview |
| `lifetime.rs` | how long a card lives: ttl, hover, interaction, pin, sticky, urgent |
| `actions.rs` | what a card's buttons do on the host side: run, open, emit, wake, reveal |
| `lint.rs` | the spec linter: no dead controls, answers must resolve, icon-only controls need a title, every number has a unit |