# src/platform: the operating-system specific parts

Everything that talks to the operating system directly lives here. The files in this folder are the Windows versions (`#[cfg(windows)]`); `macos/`
has the same modules with the same function signatures for macOS, and `mod.rs` picks one set per OS. See "macOS port" in AGENTS.md.

| file | responsibility |
|---|---|
| `window_style.rs` | window shape (hit regions), no-activate, topmost, the move cursor |
| `backdrop.rs` | sample the average color behind a window so the card can pick contrasting colors |
| `hotkeys.rs` | global hotkeys: dismiss all, summon |
| `foreground_watch.rs` | notice when the active window changes |

`placement/desktop.rs` also uses the OS (windows, monitors, the active window): the Windows API in the top half, `mod mac` (using `macos/sys.rs`) at the bottom.