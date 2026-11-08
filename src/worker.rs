//! The chore agent's settings and senses. It is a low-cost headless model (Haiku by default) that is NOT a daemon:
//! it is spawned only when the gates in wake.rs allow, does a few quick chores, and exits (= sleeps).
//!
//! prefs/worker.json (hot-reloaded, written with defaults on first use; edit it, nothing needs a rebuild):
//!   enabled            false (the default) = never wake a headless agent; set true to opt in
//!   model              the `claude --model` value for the chore agent
//!   persona            file (relative to the morc folder) appended as its system prompt: docs/agent.md
//!   idle_s             the PC counts as idle after this many seconds without keyboard/mouse input: the agent stays asleep
//!   session_fresh_s    a live interactive Claude session touched morc this recently: the agent stays asleep (the session does the work)
//!   max_per_hour, min_gap_s     hard rate limit on any headless wake
//!   chore_cooldown_min          at least this long between chore wakes (stats accumulate in between)
//!   max_turns          cap on the agent's turns per wake
//!   track_titles       log the active window (exe + title) on every change to activity.jsonl so the agent can see what you work on
//!   redact             titles containing any of these words are logged as "(redacted)"

use crate::{paths, placement, util};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Worker {
    pub enabled: bool,
    pub model: String,
    pub persona: String,
    pub idle_s: u64,
    pub session_fresh_s: u64,
    pub max_per_hour: usize,
    pub min_gap_s: u64,
    pub chore_cooldown_min: f64,
    pub max_turns: u32,
    pub track_titles: bool,
    pub redact: Vec<String>,
}

impl Default for Worker {
    fn default() -> Self {
        Worker {
            enabled: false, // opt-in: a fresh install never spawns a headless agent (it costs the owner tokens). Set "enabled": true in prefs/worker.json.
            model: "claude-haiku-5-5".into(),
            persona: "docs/agent.md".into(),
            idle_s: 180,
            session_fresh_s: 600,
            max_per_hour: 6,
            min_gap_s: 20,
            chore_cooldown_min: 30.0,
            max_turns: 8,
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
    if exe.is_empty() || exe == "morc.exe" {
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
