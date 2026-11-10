//! Child processes and the shell on Windows: hidden spawns, the detached host, opening a link or file, the clipboard, the runtime folder.

use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

/// Spawn with no console window flash.
pub fn hidden(cmd: &mut Command) -> &mut Command {
    cmd.creation_flags(0x0800_0000) // CREATE_NO_WINDOW
}

/// Start the host detached and windowless. The detached host must not inherit the pipes this CLI was started with. Windows gives a child
/// EVERY inheritable handle of its parent, not just the std ones the caller nulls, so a shell, a script or an agent capturing this CLI's
/// output would wait for the host to exit (it never does) and the first command would appear to hang for minutes. Mark our own std
/// handles non-inheritable before spawning.
pub fn detach(cmd: &mut Command) -> &mut Command {
    unsafe {
        use windows::Win32::Foundation::{SetHandleInformation, HANDLE_FLAGS, HANDLE_FLAG_INHERIT};
        use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
        for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            if let Ok(h) = GetStdHandle(id) {
                if !h.is_invalid() {
                    let _ = SetHandleInformation(h, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
                }
            }
        }
    }
    cmd.creation_flags(0x0800_0008) // CREATE_NO_WINDOW | DETACHED_PROCESS
}

/// Open an http(s) URL or an existing path with its default app.
pub fn open_target(target: &str) {
    let _ = hidden(Command::new("rundll32").args(["url.dll,FileProtocolHandler", target])).spawn();
}

/// Nothing to do on the host: the page's browser clipboard works in WebView2 (http://morc.localhost is a secure context).
pub fn copy_text(_text: String) {}

/// Where per-machine runtime state lives: %LOCALAPPDATA% (the morc folder is added by data/paths.rs).
pub fn runtime_base() -> PathBuf {
    PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into()))
}
