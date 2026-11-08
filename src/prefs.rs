//! The annoyance vector: persistent knobs that say how loud, big, frequent and flashy the UI may be.
//! Lives in prefs/annoyance.json (synced, so both machines share your taste); every change is logged to
//! prefs/annoyance_log.md with who changed it and why. The host reads it before every push.
//!
//! Who writes it: (1) Claude, when you complain ("too big on the video player" -> `morc annoy nudge size_bias 0.85 --why ...`),
//! (2) the host itself, slowly, from passive feedback (3 dismissals in a row = smaller + rarer, 4 ignored in a row =
//! maybe not noticed, a bit bigger but rarer). Explicit complaints always win: auto steps never undo a user step.

use crate::{engagement, paths, util};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::sync::Mutex;
use std::time::SystemTime;

/// name, default, min, max, meaning
pub const PARAMS: &[(&str, f64, f64, f64, &str)] = &[
    ("size_bias", 1.12, 0.5, 1.8, "multiplies surface size (zoom); capped to 1/4 of the screen (default a tad bigger, 11-08)"),
    ("ttl_bias", 1.0, 0.4, 3.0, "multiplies how long surfaces stay with no interaction"),
    ("frequency", 1.0, 0.4, 6.0, "multiplies auto-push cooldowns (higher = rarer)"),
    ("motion", 1.0, 0.0, 1.0, "animation intensity; below 0.3 = reduced motion"),
    ("sound", 1.0, 0.0, 1.0, "above 0.5 urgent surfaces may beep"),
    ("boldness", 1.0, 0.3, 1.2, "color saturation of accents"),
    ("simplicity", 0.0, 0.0, 1.0, "above 0.7 unfamiliar chart kinds fall back to plain ones"),
    ("overlap", 0.6, 0.0, 1.0, "tolerance for covering non-active windows (0 = avoid everything)"),
    ("crowd", 0.5, 0.15, 0.95, "how full the free space may get before crowd handling starts: the share of the room around the active window that our cards cover (learned: 'too crowded' lowers it, 'fine' raises it)"),
    ("dock", 0.7, 0.0, 1.0, "cohesion: snap flush to neighbouring windows and screen edges instead of floating (0 = float freely)"),
    ("morph", 0.6, 0.0, 1.0, "cohesion: reshape around neighbours - glanceable part docks beside the active window, the rest tucks below (L-shape / split)"),
    ("harmony", 0.6, 0.0, 1.0, "cohesion: line up with my own other surfaces (shared edges, tidy seams, same tone)"),
    ("bundle", 0.5, 0.0, 1.0, "cohesion for the agent: merge related pushes into one surface instead of many (read by Claude, see docs/agent.md)"),
    ("max_visible", 3.0, 1.0, 6.0, "most auto surfaces on screen at once"),
    ("mute", 0.0, 0.0, 1.0, "topic only: above 0.5 auto pushes for this topic are silenced"),
];

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Prefs {
    pub params: BTreeMap<String, f64>,
    pub topics: BTreeMap<String, BTreeMap<String, f64>>,
    pub quiet: Option<[String; 2]>,
    pub mute_apps: Vec<String>,
    /// Default accent for every surface: a theme name (coral by default) or "auto" (topic + day). Unset = coral.
    pub theme: Option<String>,
}

static CACHE: Mutex<Option<(SystemTime, Prefs)>> = Mutex::new(None);

fn file() -> std::path::PathBuf {
    let d = paths::prefs_dir();
    let _ = fs::create_dir_all(&d);
    d.join("annoyance.json")
}

pub fn load() -> Prefs {
    let m = fs::metadata(file()).and_then(|m| m.modified()).ok();
    let mut g = CACHE.lock().unwrap();
    if let (Some(m), Some((t, p))) = (m, g.as_ref()) {
        if *t == m {
            return p.clone();
        }
    }
    let p: Prefs = fs::read_to_string(file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    if let Some(m) = m {
        *g = Some((m, p.clone()));
    }
    p
}

fn spec(name: &str) -> Option<&'static (&'static str, f64, f64, f64, &'static str)> {
    PARAMS.iter().find(|p| p.0 == name)
}

/// Effective value: topic override, else global, else default; clamped to its range.
pub fn get(p: &Prefs, topic: &str, name: &str) -> f64 {
    let Some(s) = spec(name) else { return 0.0 };
    let v = p.topics.get(topic).and_then(|t| t.get(name)).or_else(|| p.params.get(name)).copied().unwrap_or(s.1);
    v.clamp(s.2, s.3)
}

pub fn quiet_now(p: &Prefs) -> bool {
    let Some([a, b]) = &p.quiet else { return false };
    let (Some(a), Some(b)) = (util::clock_to_min(a), util::clock_to_min(b)) else { return false };
    let now = util::local_minutes();
    if a <= b { now >= a && now < b } else { now >= a || now < b }
}

pub fn app_muted(p: &Prefs, exe: &str) -> bool {
    p.mute_apps.iter().any(|m| m.eq_ignore_ascii_case(exe))
}

fn save(p: &Prefs) -> Result<(), String> {
    fs::write(file(), serde_json::to_string_pretty(p).unwrap()).map_err(|e| e.to_string())
}

fn log(who: &str, what: &str, why: &str) {
    let day = {
        use windows::Win32::System::SystemInformation::GetLocalTime;
        let t = unsafe { GetLocalTime() };
        format!("{:04}-{:02}-{:02} {:02}:{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute)
    };
    let p = file().with_file_name("annoyance_log.md");
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p) {
        let _ = writeln!(f, "- {day} [{who}] {what}{}", if why.is_empty() { String::new() } else { format!(" — {why}") });
    }
}

/// Set a parameter (globally or for a topic). Returns the clamped value actually stored.
pub fn set(topic: Option<&str>, name: &str, value: f64, who: &str, why: &str) -> Result<f64, String> {
    let s = spec(name).ok_or_else(|| format!("unknown parameter '{name}' (see `morc annoy`)"))?;
    let v = value.clamp(s.2, s.3);
    let mut p = load();
    let old = get(&p, topic.unwrap_or(""), name);
    match topic {
        Some(t) => { p.topics.entry(t.to_string()).or_default().insert(name.to_string(), v); }
        None => { p.params.insert(name.to_string(), v); }
    }
    save(&p)?;
    log(who, &format!("{}{} {:.2} -> {:.2}", topic.map(|t| format!("{t}: ")).unwrap_or_default(), name, old, v), why);
    Ok(v)
}

/// Multiply the current effective value by `factor`.
pub fn nudge(topic: Option<&str>, name: &str, factor: f64, who: &str, why: &str) -> Result<f64, String> {
    let cur = get(&load(), topic.unwrap_or(""), name);
    set(topic, name, cur * factor, who, why)
}

pub fn set_quiet(range: Option<(String, String)>, why: &str) -> Result<(), String> {
    let mut p = load();
    p.quiet = range.clone().map(|(a, b)| [a, b]);
    save(&p)?;
    log("user", &match range { Some((a, b)) => format!("quiet hours {a}-{b}"), None => "quiet hours off".into() }, why);
    Ok(())
}

pub fn set_theme(name: &str, why: &str) -> Result<(), String> {
    let mut p = load();
    p.theme = Some(name.to_string());
    save(&p)?;
    log("user", &format!("default theme -> {name}"), why);
    Ok(())
}

pub fn mute_app(exe: &str, on: bool, why: &str) -> Result<(), String> {
    let mut p = load();
    p.mute_apps.retain(|m| !m.eq_ignore_ascii_case(exe));
    if on { p.mute_apps.push(exe.to_lowercase()); }
    save(&p)?;
    log("user", &format!("{} auto pushes while {exe} is in front", if on { "silence" } else { "allow" }), why);
    Ok(())
}

/// Passive feedback: only acts on a fresh run of outcomes (after the last adaptation for the topic).
pub fn adapt(topic: &str) {
    if topic.is_empty() { return; }
    let p = load();
    let since = p.topics.get(topic).and_then(|t| t.get("_adapt_ts")).copied().unwrap_or(0.0) as u64;
    let (kind, len) = engagement::streak(topic, since);
    let did = match (kind.as_str(), len) {
        ("dismissed", n) if n >= 3 => {
            let _ = nudge(Some(topic), "size_bias", 0.9, "auto", &format!("{n} dismissals in a row: smaller"));
            let _ = nudge(Some(topic), "frequency", 1.25, "auto", "and rarer");
            true
        }
        ("ignored", n) if n >= 4 => {
            if get(&load(), topic, "size_bias") < 1.5 {
                let _ = nudge(Some(topic), "size_bias", 1.12, "auto", &format!("{n} ignored in a row: maybe not noticed, a bit bigger"));
            }
            let _ = nudge(Some(topic), "frequency", 1.15, "auto", "but rarer, so it does not nag");
            true
        }
        _ => false,
    };
    if did {
        let mut p = load();
        p.topics.entry(topic.to_string()).or_default().insert("_adapt_ts".into(), util::now_ms() as f64);
        let _ = save(&p);
    }
}

pub fn describe() -> serde_json::Value {
    let p = load();
    let eff: serde_json::Map<String, serde_json::Value> = PARAMS.iter().map(|s| (s.0.to_string(), serde_json::json!({"value": get(&p, "", s.0), "default": s.1, "range": [s.2, s.3], "meaning": s.4}))).collect();
    serde_json::json!({"params": eff, "topics": p.topics, "quiet": p.quiet, "mute_apps": p.mute_apps, "theme": p.theme.clone().unwrap_or_else(|| "coral".into())})
}
