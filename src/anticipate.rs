//! Anticipation: show things before being asked, and learn when that's welcome.
//! Each triggers/*.json file is one rule. A rule fires when its source value passes its condition, inside its time
//! window, once its cooldown has passed. The cooldown stretches (up to 6x) for topics you keep ignoring and
//! relaxes for topics you act on (engagement.rs). The host may still defer (fullscreen app, too many surfaces).
//!
//! {"name":"video-80","every":60,
//!  "source":{"file":"%LOCALAPPDATA%/myapp/stats.json","pointer":"/today/minutes"},
//!  "when":{"gte":108}, "window":["08:00","23:30"],
//!  "use":"budget-card","data":{},"topic":"stats.minutes","cooldown_min":60,"opts":{"ttl":18}}

use crate::{engagement, ev::{Ev, Proxy}, paths, protocol::{Opts, Request}, sources, util};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn cond_ok(when: &Value, v: &Value) -> bool {
    let n = v.as_f64();
    let num = |k: &str| when[k].as_f64();
    if let (Some(x), Some(t)) = (n, num("gte")) { if !(x >= t) { return false; } }
    if let (Some(x), Some(t)) = (n, num("lte")) { if !(x <= t) { return false; } }
    if let (Some(x), Some(t)) = (n, num("gt")) { if !(x > t) { return false; } }
    if let (Some(x), Some(t)) = (n, num("lt")) { if !(x < t) { return false; } }
    if when.get("eq").is_some() && when["eq"] != *v { return false; }
    if when.get("ne").is_some() && when["ne"] == *v { return false; }
    (when.get("gte").is_none() && when.get("lte").is_none() && when.get("gt").is_none() && when.get("lt").is_none()) || n.is_some()
}

fn in_window(w: &Value) -> bool {
    let (Some(a), Some(b)) = (w[0].as_str().and_then(util::clock_to_min), w[1].as_str().and_then(util::clock_to_min)) else { return true };
    let now = util::local_minutes();
    if a <= b { now >= a && now <= b } else { now >= a || now <= b }
}

fn load_state() -> HashMap<String, u64> {
    fs::read_to_string(paths::trigger_state_file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// Wake flags: the chore agent sets its OWN next wake (prefs/wake_flags.json, via `morc wake set`). A flag is due when its time has passed AND
/// enough has accumulated (min_activity window changes since it was set) AND its optional condition on a number in a JSON file holds. One-shot unless repeat_min.
fn wake_flags(proxy: &Proxy) {
    let path = paths::prefs_dir().join("wake_flags.json");
    let Some(mut flags) = fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str::<Vec<Value>>(&s).ok()) else { return };
    if flags.is_empty() { return; }
    let now = util::now_ms() as u64;
    let mut changed = false;
    let mut i = 0;
    while i < flags.len() {
        let f = flags[i].clone();
        let mut due = now >= f["after_ts"].as_u64().unwrap_or(0);
        let need = f["min_activity"].as_u64().unwrap_or(0) as usize;
        if due && need > 0 { due = crate::worker::activity_since(f["since_ts"].as_u64().unwrap_or(0)) >= need; }
        if due && f["when"].is_object() {
            due = sources::read_pointer(f["when"]["file"].as_str().unwrap_or(""), f["when"]["pointer"].as_str().unwrap_or("")).map(|v| cond_ok(&f["when"], &v)).unwrap_or(false);
        }
        if !due { i += 1; continue; }
        let reason = f["reason"].as_str().or(f["name"].as_str()).unwrap_or("flag").to_string();
        let (tx, rx) = mpsc::channel();
        if proxy.send_event(Ev::Cmd(Request::Chore { reason, note: f.clone() }, Some(tx))).is_err() { return; }
        let fired = rx.recv_timeout(Duration::from_secs(5)).map(|r| r.contains("headless-spawned") || r.contains("dry-run")).unwrap_or(false);
        if fired {
            changed = true;
            match f["repeat_min"].as_f64() {
                Some(m) => { flags[i]["after_ts"] = json!(now + (m * 60_000.0) as u64); flags[i]["since_ts"] = json!(now); i += 1; }
                None => { flags.remove(i); }
            }
        } else {
            i += 1; // gated (idle / live session / cooldown): stays due, retried every 10 s at no cost
        }
    }
    if changed { let _ = fs::write(&path, serde_json::to_string_pretty(&flags).unwrap_or_default()); }
}

pub fn spawn(proxy: Proxy) {
    std::thread::spawn(move || {
        let mut last_eval: HashMap<String, Instant> = HashMap::new();
        loop {
            std::thread::sleep(Duration::from_secs(10));
            wake_flags(&proxy);
            let Ok(rd) = fs::read_dir(paths::triggers_dir()) else { continue };
            for e in rd.filter_map(|e| e.ok()) {
                let Some(t) = fs::read_to_string(e.path()).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()) else { continue };
                let name = t["name"].as_str().unwrap_or("?").to_string();
                if t["disabled"].as_bool().unwrap_or(false) { continue; }
                let every = Duration::from_secs_f64(t["every"].as_f64().unwrap_or(60.0).max(10.0));
                if last_eval.get(&name).map(|l| l.elapsed() < every).unwrap_or(false) { continue; }
                last_eval.insert(name.clone(), Instant::now());

                if t.get("window").is_some() && !in_window(&t["window"]) { continue; }
                let Some(v) = sources::read_pointer(t["source"]["file"].as_str().unwrap_or(""), t["source"]["pointer"].as_str().unwrap_or("")) else { continue };
                if !cond_ok(&t["when"], &v) { continue; }

                let topic = t["topic"].as_str().unwrap_or(&name).to_string();
                let mut state = load_state();
                // engagement stretches it for topics you ignore; the annoyance vector's "frequency" stretches it for what annoys you
                let cooldown_ms = t["cooldown_min"].as_f64().unwrap_or(60.0) * 60_000.0 * engagement::cooldown_factor(&topic) * crate::prefs::get(&crate::prefs::load(), &topic, "frequency");
                let last = state.get(&name).copied().unwrap_or(0) as f64;
                if (util::now_ms() as f64) - last < cooldown_ms { continue; }

                // A chore rule: instead of showing a widget, ask for a chore wake of the background agent (the host applies the gates).
                if let Some(reason) = t["chore"].as_str() {
                    let mut note = t["data"].clone();
                    if note.is_null() { note = json!({}); }
                    note["_trigger"] = json!(v);
                    let (tx, rx) = mpsc::channel();
                    if proxy.send_event(Ev::Cmd(Request::Chore { reason: reason.to_string(), note }, Some(tx))).is_err() { return; }
                    if let Ok(reply) = rx.recv_timeout(Duration::from_secs(5)) {
                        if reply.contains("headless-spawned") || reply.contains("dry-run") {
                            state.insert(name, util::now_ms() as u64);
                            let _ = fs::write(paths::trigger_state_file(), serde_json::to_string(&state).unwrap());
                        }
                    }
                    continue;
                }
                let mut opts: Opts = serde_json::from_value(t["opts"].clone()).unwrap_or_default();
                opts.auto = true;
                opts.topic = Some(topic);
                let mut data = t["data"].clone();
                if data.is_null() { data = json!({}); }
                data["_trigger"] = json!(v); // templates can show the value that tripped the rule: {/_trigger}
                let (tx, rx) = mpsc::channel();
                let req = Request::Use { name: t["use"].as_str().unwrap_or("").to_string(), id: None, opts, data };
                if proxy.send_event(Ev::Cmd(req, Some(tx))).is_err() { return; }
                if let Ok(reply) = rx.recv_timeout(Duration::from_secs(5)) {
                    if reply.contains("\"shown\"") {
                        state.insert(name, util::now_ms() as u64);
                        let _ = fs::write(paths::trigger_state_file(), serde_json::to_string(&state).unwrap());
                    }
                }
            }
        }
    });
}
