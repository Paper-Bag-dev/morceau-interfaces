//! Where things live. Three places: the INSTALL (app_dir), YOUR DATA (home_dir) and per-machine RUNTIME state (runtime_dir).
//! The install is read-only shipped content; your data survives upgrades; runtime state (token, logs, cursors) is under %LOCALAPPDATA%\morc.

use std::path::PathBuf;

pub const HOST: &str = "127.0.0.1";
pub const PORT: u16 = 47831;

/// The INSTALL folder: the shipped UI, tools, docs and generic components. MORC_APP, else the nearest folder above the exe that has
/// ui/index.html (npm and release layouts and `target/release` inside a checkout), else the path in <runtime>/app.txt, else the current folder.
pub fn app_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MORC_APP") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut d = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..6 {
            match d {
                Some(x) if x.join("ui").join("index.html").exists() => return x,
                Some(x) => d = x.parent().map(|p| p.to_path_buf()),
                None => break,
            }
        }
    }
    // last resort: a path recorded in <runtime>/app.txt (never the build machine's path baked into the exe), else the current folder
    if let Ok(s) = std::fs::read_to_string(runtime_dir().join("app.txt")) {
        let t = s.trim().trim_start_matches('\u{feff}');
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    PathBuf::from(".")
}

/// YOUR data folder: prefs/, triggers/, actions.json, cache/components, your own recipes and fragments, and optional overrides of the
/// agent persona (docs/agent.md) and chore prompt (chore-prompt.md). MORC_HOME, else the path inside <runtime>/home.txt, else the install
/// folder itself (portable mode). Keeping it separate means upgrading the install never touches your data.
pub fn home_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MORC_HOME") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Ok(s) = std::fs::read_to_string(runtime_dir().join("home.txt")) {
        let t = s.trim().trim_start_matches('\u{feff}');
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    app_dir()
}

pub fn runtime_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    let d = PathBuf::from(base).join("morc");
    let _ = std::fs::create_dir_all(&d);
    d
}

pub fn ui_dir() -> PathBuf {
    app_dir().join("ui")
}
pub fn cache_dir() -> PathBuf {
    home_dir().join("cache")
}
pub fn components_dir() -> PathBuf {
    cache_dir().join("components")
}
pub fn triggers_dir() -> PathBuf {
    home_dir().join("triggers")
}
pub fn prefs_dir() -> PathBuf {
    home_dir().join("prefs")
}
pub fn actions_file() -> PathBuf {
    home_dir().join("actions.json")
}
pub fn token_file() -> PathBuf {
    runtime_dir().join("token")
}
pub fn inbox_file() -> PathBuf {
    runtime_dir().join("inbox.jsonl")
}
pub fn inbox_cursor_file() -> PathBuf {
    runtime_dir().join("inbox.cursor")
}
pub fn engagement_file() -> PathBuf {
    runtime_dir().join("engagement.jsonl")
}
pub fn session_file() -> PathBuf {
    runtime_dir().join("session.seen")
}
pub fn usage_file() -> PathBuf {
    runtime_dir().join("usage.json")
}
pub fn trigger_state_file() -> PathBuf {
    runtime_dir().join("trigger_state.json")
}
pub fn wake_log_file() -> PathBuf {
    runtime_dir().join("wake.log")
}
