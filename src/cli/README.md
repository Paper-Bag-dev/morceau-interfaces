# src/cli: the command-line client (`morc ...`)

| file | responsibility |
|---|---|
| `mod.rs` | `run()`: the command dispatcher and the help text (its `//!` header is printed by `morc help`) |
| `input.rs` | reading a JSON argument (inline, `@file`, stdin), flags and values |
| `connect.rs` | talking to the host over localhost; starting it detached when it is not running |
| `plugins.rs` | CLI plugins: any script in `tools/cli` (data folder first) becomes `morc <name>` |
| `stream.rs` | `morc stream`: a spec shown piece by piece, skeleton cards first |
| `annoy.rs` | `morc annoy`: read and nudge the annoyance vector |
| `mcp.rs` | `morc mcp`: the stdio MCP server, six fixed tools over the same requests (docs/mcp.md) |
