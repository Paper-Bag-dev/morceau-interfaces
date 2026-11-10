//! Child processes and the shell on macOS: hidden spawns, the detached host, opening a link or file, the clipboard, the runtime folder.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// A child process never opens a console window on macOS, so there is nothing to hide.
pub fn hidden(cmd: &mut Command) -> &mut Command {
    cmd
}

/// Start the host in its own process group, so closing the terminal (SIGHUP to the foreground group) or Ctrl+C does not take it down.
/// Unix children only inherit descriptors without close-on-exec, and Rust opens everything close-on-exec, so no handle dance is needed.
pub fn detach(cmd: &mut Command) -> &mut Command {
    use std::os::unix::process::CommandExt;
    cmd.process_group(0)
}

/// Open an http(s) URL or an existing path with its default app.
pub fn open_target(target: &str) {
    let _ = Command::new("open").arg(target).spawn();
}

/// Put text on the clipboard with pbcopy (WKWebView may refuse the browser clipboard on the morc:// page). Off the calling thread.
pub fn copy_text(text: String) {
    std::thread::spawn(move || {
        if let Ok(mut child) = Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    });
}

/// Where per-machine runtime state lives: ~/Library/Application Support (the morc folder is added by data/paths.rs).
pub fn runtime_base() -> PathBuf {
    std::env::var("HOME").map(|h| PathBuf::from(h).join("Library").join("Application Support")).unwrap_or_else(|_| PathBuf::from("."))
}
