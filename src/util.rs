//! Tiny shared helpers: clock, env expansion, JSON merge/pointer, hidden process spawn.

use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
}

/// Expand %VAR% segments using the process environment.
pub fn expand_env(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('%') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        match after.find('%') {
            Some(j) => {
                out.push_str(&std::env::var(&after[..j]).unwrap_or_default());
                rest = &after[j + 1..];
            }
            None => {
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Recursively merge `over` into `base` (objects merge, everything else replaces).
pub fn deep_merge(base: &mut Value, over: &Value) {
    match (base, over) {
        (Value::Object(b), Value::Object(o)) => {
            for (k, v) in o {
                deep_merge(b.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (b, o) => *b = o.clone(),
    }
}

/// Spawn a process with no console window flash.
#[cfg(windows)]
pub fn hidden(cmd: &mut std::process::Command) -> &mut std::process::Command {
    use std::os::windows::process::CommandExt;
    cmd.creation_flags(0x0800_0000) // CREATE_NO_WINDOW
}

// macOS: platform/macos/process.rs
#[cfg(target_os = "macos")]
pub use crate::platform::macos::process::hidden;

/// The Python launcher the CLI plugins and `morc graph` run with: the `py` launcher on Windows, `python3` elsewhere.
pub const PYTHON: &str = if cfg!(windows) { "py" } else { "python3" };

/// Append a line to the host log (runtime dir). Keeps the log from growing past ~1 MB.
pub fn log(msg: &str) {
    use std::io::Write;
    let p = crate::paths::runtime_dir().join("host.log");
    if std::fs::metadata(&p).map(|m| m.len() > 1_000_000).unwrap_or(false) {
        let _ = std::fs::remove_file(&p);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
        let _ = writeln!(f, "{} {}", now_ms(), msg);
    }
}

/// "HH:MM" -> minutes since midnight.
pub fn clock_to_min(s: &str) -> Option<u32> {
    let (h, m) = s.split_once(':')?;
    Some(h.trim().parse::<u32>().ok()? * 60 + m.trim().parse::<u32>().ok()?)
}

/// Current local minutes since midnight.
#[cfg(windows)]
pub fn local_minutes() -> u32 {
    use windows::Win32::System::SystemInformation::GetLocalTime;
    let t = unsafe { GetLocalTime() };
    t.wHour as u32 * 60 + t.wMinute as u32
}

// macOS: platform/macos/clock.rs
#[cfg(target_os = "macos")]
pub use crate::platform::macos::clock::local_minutes;
