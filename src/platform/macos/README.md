# src/platform/macos: the macOS parts

Same modules and function signatures as the Windows files one folder up, so the host does not know which one it runs on.

| file | responsibility |
|---|---|
| `window_style.rs` | float level on all Spaces, no shadow, show without activating, and click-through outside the content (a main-thread timer flips `ignoresMouseEvents`, because macOS has no window regions) |
| `backdrop.rs` | returns None for now: sampling other apps' pixels needs the Screen Recording permission |
| `hotkeys.rs` | Ctrl+Option+X dismiss all, Ctrl+Option+Space summon (global-hotkey crate) |
| `foreground_watch.rs` | polls the window server every 400 ms and sends `Ev::Foreground` when the front window changes |
| `sys.rs` | every raw call in one place: CGWindowList, NSScreen, NSWorkspace, idle time, mouse button, beep, `on_main` (libdispatch) |

Coordinates: AppKit uses points from the bottom-left of the primary screen, CoreGraphics window bounds use points from the top-left, and the host
uses physical pixels from the top-left. `sys::screens()` flips to top-left; callers multiply by `sys::scale()`.
