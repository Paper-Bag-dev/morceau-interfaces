# src/placement: where cards go

| file | responsibility |
|---|---|
| `mod.rs` | declarations and re-exports (`R`, `foreground`, `pick`, ...) |
| `rect.rs` | a screen rectangle in physical pixels |
| `desktop.rs` | what the desktop looks like now (Windows API): windows, monitors, the active window, fullscreen, its exe and title |
| `regions.rs` | the region model: the 3x3 around the active window, tiers, thirds when a fullscreen app is active |
| `scored.rs` | scored placement, the fallback when the regions do not fit |
| `policy.rs` | the weights and anchors, from `prefs/placement.json` (hot-reloaded) |
| `remembered.rs` | the spots you carried cards to |