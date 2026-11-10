//! The chore agent's settings and senses. It is a low-cost headless model (Haiku by default) that is NOT a daemon:
//! it is spawned only when the gates in wake.rs allow, does a few quick chores, and exits (= sleeps).
//!
//! prefs/worker.json (hot-reloaded, written with defaults on first use; edit it, nothing needs a rebuild):
//!   enabled            when the background agent may be woken (it costs tokens):
//!                        "no_session" (the default) = only while no interactive Claude session is live (a session's `morc hook` touched morc
//!                                     within session_fresh_s); while one is, every click goes to that session instead
//!                        true / "on"  = any time the other gates allow;  false / "off" = never
//!   model              the `claude --model` value for the chore agent
//!   persona            file (relative to the morc folder) appended as its system prompt: docs/agent.md
//!   idle_s             the PC counts as idle after this many seconds without keyboard/mouse input: the agent stays asleep
//!   defer_to_session   false (the default): the background agent and a live interactive Claude session work at the same time; true = the agent stays asleep while a session touched morc within session_fresh_s
//!   session_fresh_s    how recent a session's last `morc hook` must be to count as live ("no_session" and defer_to_session)
//!   max_per_hour, min_gap_s     hard rate limit on any headless wake
//!   chore_cooldown_min          at least this long between chore wakes (stats accumulate in between)
//!   max_turns          cap on the agent's turns per wake
//!   track_titles       log the active window (exe + title) on every change to activity.jsonl so the agent can see what you work on
//!   redact             titles containing any of these words are logged as "(redacted)"

use crate::{inbox, paths, placement, util};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

/// When the background agent may run. In prefs/worker.json: true ("on"), false ("off") or "no_session" (the default).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    On,
    Off,
    NoSession,
}

impl Serialize for Mode {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Mode::On => s.serialize_bool(true),
            Mode::Off => s.serialize_bool(false),
            Mode::NoSession => s.serialize_str("no_session"),
        }
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(match Value::deserialize(d)? {
            Value::Bool(true) => Mode::On,
            Value::Bool(false) => Mode::Off,
            Value::String(s) if s == "on" => Mode::On,
            Value::String(s) if s == "off" => Mode::Off,
            _ => Mode::NoSession,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Worker {
    pub enabled: Mode,
    pub model: String,
    pub persona: String,
    pub idle_s: u64,
    pub defer_to_session: bool,
    /// Seconds the agent stays awake after its last interaction (a finished turn or a new message), waiting for more work, before it sleeps.
    pub linger_s: u64,
    pub session_fresh_s: u64,
    pub max_per_hour: usize,
    pub min_gap_s: u64,
    pub chore_cooldown_min: f64,
    pub max_turns: u32,
    pub track_titles: bool,
    pub redact: Vec<String>,
}

impl Worker {
    /// Did an interactive Claude session touch morc (through `morc hook`) within session_fresh_s?
    pub fn session_live(&self) -> bool {
        inbox::session_age_s().map(|age| age < self.session_fresh_s).unwrap_or(false)
    }
}

impl Default for Worker {
    fn default() -> Self {
        Worker {
            enabled: Mode::NoSession, // by default the background agent covers only for an absent Claude session: with one open, your clicks go to it. "on" / true or "off" / false in prefs/worker.json.
            model: "claude-haiku-5-5".into(),
            persona: "docs/agent.md".into(),
            idle_s: 180,
            defer_to_session: false,
            linger_s: 600, // awake for 10 minutes after the last interaction; every new one restarts the timer
            session_fresh_s: 600,
            max_per_hour: 6,
            min_gap_s: 20,
            chore_cooldown_min: 30.0,
            max_turns: 0,
            track_titles: true,
            redact: ["password", "passcode", "otp", "bank", "login", "sign in", "private", "incognito"].iter().map(|s| s.to_string()).collect(),
        }
    }
}

static CACHE: Mutex<Option<(SystemTime, Worker)>> = Mutex::new(None);

fn file() -> PathBuf {
    let d = paths::prefs_dir();
    let _ = fs::create_dir_all(&d);
    d.join("worker.json")
}

pub fn load() -> Worker {
    let path = file();
    let m = fs::metadata(&path).and_then(|m| m.modified()).ok();
    let mut g = CACHE.lock().unwrap();
    if let (Some(m), Some((t, w))) = (m, g.as_ref()) {
        if *t == m {
            return w.clone();
        }
    }
    let w = match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str::<Worker>(s.trim_start_matches('\u{feff}')).unwrap_or_else(|e| {
            util::log(&format!("worker.json is invalid ({e}); using defaults"));
            Worker::default()
        }),
        Err(_) => {
            let d = Worker::default();
            if let Ok(s) = serde_json::to_string_pretty(&d) {
                let _ = fs::write(&path, s);
            }
            d
        }
    };
    if let Some(m) = fs::metadata(&path).and_then(|m| m.modified()).ok() {
        *g = Some((m, w.clone()));
    }
    w
}

/// Seconds since the last keyboard or mouse input anywhere in Windows.
#[cfg(windows)]
pub fn idle_seconds() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    unsafe {
        let mut li = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        if GetLastInputInfo(&mut li).as_bool() {
            (GetTickCount().wrapping_sub(li.dwTime) / 1000) as u64
        } else {
            0
        }
    }
}

/// macOS: seconds since the last keyboard or mouse input anywhere (CoreGraphics event source; no permission needed).
#[cfg(target_os = "macos")]
pub fn idle_seconds() -> u64 {
    crate::platform::macos::sys::idle_seconds()
}

fn activity_file() -> PathBuf {
    paths::runtime_dir().join("activity.jsonl")
}

/// Called on every active-window change: remember what you are working in (exe + title), de-duplicated and capped.
pub fn note_foreground() {
    let w = load();
    if !w.track_titles {
        return;
    }
    let exe = placement::fg_exe().unwrap_or_default();
    if exe.is_empty() || exe == "morc.exe" || exe == "morc" {
        return;
    }
    let raw = placement::fg_title().unwrap_or_default();
    let low = raw.to_lowercase();
    let title: String = if w.redact.iter().any(|r| !r.is_empty() && low.contains(&r.to_lowercase())) { "(redacted)".into() } else { raw.chars().take(120).collect() };
    let f = activity_file();
    let body = fs::read_to_string(&f).unwrap_or_default();
    if let Some(last) = body.lines().last().and_then(|l| serde_json::from_str::<Value>(l).ok()) {
        if last["exe"] == json!(exe) && last["title"] == json!(title) {
            return;
        }
    }
    if let Ok(mut fh) = fs::OpenOptions::new().create(true).append(true).open(&f) {
        let _ = writeln!(fh, "{}", json!({"ts": util::now_ms() as u64, "exe": exe, "title": title}));
    }
    if body.len() > 256_000 {
        let keep: Vec<&str> = body.lines().rev().take(400).collect();
        let _ = fs::write(&f, keep.into_iter().rev().collect::<Vec<_>>().join("\n") + "\n");
    }
}

/// The last `n` distinct things you were in, oldest first, as "exe: title".
pub fn recent_activity(n: usize) -> Vec<String> {
    let body = fs::read_to_string(activity_file()).unwrap_or_default();
    let mut out: Vec<String> = vec![];
    for l in body.lines().rev().take(80) {
        if let Ok(v) = serde_json::from_str::<Value>(l) {
            let s = format!("{}: {}", v["exe"].as_str().unwrap_or("?"), v["title"].as_str().unwrap_or(""));
            if !out.contains(&s) {
                out.push(s);
            }
        }
        if out.len() >= n {
            break;
        }
    }
    out.reverse();
    out
}

/// How many window changes were logged after `ts` (ms): "enough data has accumulated" for a wake flag's min_activity.
pub fn activity_since(ts: u64) -> usize {
    let body = fs::read_to_string(activity_file()).unwrap_or_default();
    body.lines().rev().take(400).filter_map(|l| serde_json::from_str::<Value>(l).ok()).filter(|v| v["ts"].as_u64().unwrap_or(0) > ts).count()
}

fn chore_file() -> PathBuf {
    paths::runtime_dir().join("chore.last")
}

pub fn last_chore_ms() -> u128 {
    fs::read_to_string(chore_file()).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

pub fn mark_chore() {
    let _ = fs::write(chore_file(), util::now_ms().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(json: &str) -> Mode {
        serde_json::from_str::<Worker>(json).unwrap().enabled
    }

    #[test]
    fn enabled_reads_all_three_modes() {
        assert_eq!(mode(r#"{"enabled": true}"#), Mode::On);
        assert_eq!(mode(r#"{"enabled": "on"}"#), Mode::On);
        assert_eq!(mode(r#"{"enabled": false}"#), Mode::Off);
        assert_eq!(mode(r#"{"enabled": "off"}"#), Mode::Off);
        assert_eq!(mode(r#"{"enabled": "no_session"}"#), Mode::NoSession);
        assert_eq!(mode(r#"{}"#), Mode::NoSession); // the default
    }
}
