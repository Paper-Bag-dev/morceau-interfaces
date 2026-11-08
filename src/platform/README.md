# src/platform: the Windows-specific parts

Everything that talks to the operating system directly lives here, so a macOS port replaces this folder (see BACKLOG.md).

| file | responsibility |
|---|---|
| `window_style.rs` | window shape (hit regions), no-activate, topmost, the move cursor |
| `backdrop.rs` | sample the average color behind a window so the card can pick contrasting colors |
| `hotkeys.rs` | global hotkeys: dismiss all, summon |
| `foreground_watch.rs` | notice when the active window changes |

`placement/desktop.rs` also uses the Windows API (windows, monitors, the active window); it belongs here in a port.