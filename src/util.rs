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

/// macOS: a child process never opens a console window, so there is nothing to hide.
#[cfg(not(windows))]
pub fn hidden(cmd: &mut std::process::Command) -> &mut std::process::Command {
    cmd
}

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

/// macOS: current local minutes since midnight.
#[cfg(not(windows))]
pub fn local_minutes() -> u32 {
    let (_, _, _, h, m) = local_time();
    h * 60 + m
}

/// macOS: local wall-clock time (year, month, day, hour, minute) from the C library, so no date crate is needed.
#[cfg(not(windows))]
pub fn local_time() -> (i32, u32, u32, u32, u32) {
    #[repr(C)]
    struct Tm {
        sec: i32,
        min: i32,
        hour: i32,
        mday: i32,
        mon: i32,
        year: i32,
        wday: i32,
        yday: i32,
        isdst: i32,
        gmtoff: std::ffi::c_long,
        zone: *const std::ffi::c_char,
    }
    extern "C" {
        fn time(t: *mut i64) -> i64;
        fn localtime_r(t: *const i64, out: *mut Tm) -> *mut Tm;
    }
    unsafe {
        let now = time(std::ptr::null_mut());
        let mut tm: Tm = std::mem::zeroed();
        if localtime_r(&now, &mut tm).is_null() {
            return (1970, 1, 1, 0, 0);
        }
        (tm.year + 1900, (tm.mon + 1) as u32, tm.mday as u32, tm.hour as u32, tm.min as u32)
    }
}
