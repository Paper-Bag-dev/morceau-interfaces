# src/ipc: the CLI <-> host protocol

| file | responsibility |
|---|---|
| `protocol.rs` | request and reply types, `Opts` (everything a push can say) |
| `server.rs` | the localhost TCP server (token-checked) |
| `events.rs` | the events the host's event loop handles |