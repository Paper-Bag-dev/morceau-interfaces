//! The agent inbox: UI interactions meant for Claude (wake / emit actions, form submits).
//! Append-only JSONL plus a byte cursor, so every reader (hook, Monitor, headless wake) sees each event once.
//! Behavioural stats do NOT go here; see engagement.rs.

use crate::{paths, util};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;

pub fn push(surface: &str, topic: &str, kind: &str, payload: &Value) {
    let line = json!({"ts": util::now_ms() as u64, "surface": surface, "topic": topic, "kind": kind, "payload": payload});
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(paths::inbox_file()) {
        let _ = writeln!(f, "{}", line);
    }
}

fn cursor() -> u64 {
    fs::read_to_string(paths::inbox_cursor_file()).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0)
}

/// Events after the cursor. Advances the cursor unless `peek`.
pub fn unread(peek: bool) -> Vec<Value> {
    let Ok(bytes) = fs::read(paths::inbox_file()) else { return vec![] };
    let start = (cursor() as usize).min(bytes.len());
    let text = String::from_utf8_lossy(&bytes[start..]);
    let out: Vec<Value> = text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
    if !peek {
        let _ = fs::write(paths::inbox_cursor_file(), bytes.len().to_string());
    }
    out
}

/// Remember that a Claude session is alive (hooks call this).
pub fn touch_session() {
    let _ = fs::write(paths::session_file(), util::now_ms().to_string());
}

/// Seconds since a Claude session last touched us, if ever.
pub fn session_age_s() -> Option<u64> {
    let t: u128 = fs::read_to_string(paths::session_file()).ok()?.trim().parse().ok()?;
    Some(((util::now_ms().saturating_sub(t)) / 1000) as u64)
}

/// Text for the Claude Code hook: what the user did on screen while Claude wasn't looking.
pub fn hook_json(event_name: &str) -> String {
    // the background agent runs with the same hooks: it must not count as a live session or take the interactive session's unread events
    if std::env::var("MORC_HEADLESS").is_ok() {
        return String::new();
    }
    touch_session();
    let ev = unread(false);
    if ev.is_empty() {
        return String::new();
    }
    let mut lines = vec![format!(
        "[morc] {} UI interaction(s) from the user's screen while you were away. Act on them as the screen agent (docs: morc/docs/agent.md):",
        ev.len()
    )];
    // a click marked bg:true is already being handled by the background agent: tell the session, do not hand it over as work.
    // With the background agent off (prefs/worker.json "enabled": false) nobody else will act on it, so every event is the session's.
    let agent_on = crate::worker::load().enabled;
    let (bg, mine): (Vec<&Value>, Vec<&Value>) = ev.iter().partition(|e| agent_on && e["payload"]["bg"].as_bool().unwrap_or(false));
    for e in mine.iter().take(20) {
        lines.push(format!("- {}", e));
    }
    if !bg.is_empty() {
        lines.push(format!("- {} click(s) went to the background agent (bg:true); it is handling them. Act on them only if it reports a failure or the user asks.", bg.len()));
    }
    json!({"hookSpecificOutput": {"hookEventName": event_name, "additionalContext": lines.join("\n")}}).to_string()
}
