# src/platform: the operating-system specific parts

Shared code never names an OS. Each platform folder exports the SAME modules with the SAME functions, and `mod.rs` picks one as
`crate::platform::os`. A new platform is a new folder with these modules; no shared file changes.

| module | responsibility |
|---|---|
| `app` | event-loop setup, window and webview builder extras, the native window handle, the page URL |
| `window_style` | overlay styles, show without focus, hide, the content shape (click-through), the move cursor |
| `desktop` | the windows, monitors, the active window, its app and title (`placement/desktop.rs` re-exports and snapshots them) |
| `backdrop` | the average colour behind a card (macOS: none, needs Screen Recording) |
| `hotkeys` | dismiss all, summon |
| `foreground_watch` | notice when the active window changes |
| `process` | hidden spawns, detaching the host, open a link or file, host-side copy, the runtime folder |
| `clock` | local time |
| `sys` | idle time, the mouse button, the beep (macOS: also the raw AppKit / CoreGraphics calls) |

`windows/` uses the `windows` crate; `macos/` uses objc2 and small extern blocks (see `macos/README.md`). A function with nothing to do on
one OS is a documented no-op there (e.g. `app::configure_event_loop` on Windows, `process::copy_text` on Windows, `app::webview_builder` on macOS).
CI enforces the split ("Platform code stays in src/platform").
