# src/data: where things live

| file | responsibility |
|---|---|
| `paths.rs` | the install folder, your data folder, the per-machine runtime folder, and every file path in them |
| `cache.rs` | the LFU component cache: use counts, decay, pruning, the registry |
| `assets.rs` | the page files (`ui/`) served to the cards from disk |