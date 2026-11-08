//! Waking a headless agent (the chore agent, Haiku by default), cheapest and most conservative first.
//! Every wake, whatever asked for it, passes the same gates in this order:
//!   1. only if prefs/worker.json says defer_to_session: a live interactive session touched morc recently -> rest. By default the agent and the session work together
//!   2. the PC is idle (no keyboard/mouse input for idle_s)      -> rest
//!   3. hard rate limit (max_per_hour, min_gap_s)                -> rest
//!   4. chores only: the chore cooldown has not passed           -> rest
//! Passing them starts ONE headless `claude -p` (model, persona, turn cap from prefs/worker.json) that reads its work from stdin. Later wakes are
//! written to that same process and queue behind what it is doing. It stays awake `linger_s` after its last turn, then the host closes its input: its sleep.
//! Two ways in: a user action on a widget (maybe_wake) and a chore (chore: from a trigger rule, an external tool, or `morc chore <reason>`).
//!
//! Files: gate.rs (the gates), process.rs (the persistent agent process and its queue); this file builds the prompts and is the way in.

mod gate;
mod process;
use gate::gate;
use process::spawn;

use crate::{inbox, paths, util, worker};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

/// What the headless agent types to run the CLI: plain `morc` (spawn puts the exe's folder on its PATH).
fn exe_path() -> String {
    "morc".to_string()
}

/// The user acted on a widget that needs the agent.
pub fn maybe_wake(payload: &Value) -> &'static str {
    let w = worker::load();
    let direct = payload["bg"].as_bool().unwrap_or(false);
    let click = direct && !payload["prefetch"].as_bool().unwrap_or(false);   // a prefetch is the host's own idea, a click is the user's
    let recent = match gate(&w, false, direct, click) {
        Ok(r) => r,
        Err(why) => return why,
    };
    // an optional "task" in the payload says exactly what to build (e.g. "add 3 graph cards from FILE, patch them into the surface")
    let task = payload["task"].as_str().map(|t| format!("\nTask: {t}")).unwrap_or_default();
    let prompt = format!(
        "The user just acted on a floating morc window on their screen. Event: {payload}{task}\n\
         Handle it as the screen agent. Use only the CLI at {} (show/patch/use/close). Show things as graphs, big numbers and pictures, not text. \
         Be brief; reply on screen, not in text. Patch the surface named in the event; when done set its /state to \"ready\".",
        exe_path()
    );
    spawn(&w, &prompt, recent, false)
}

/// A chore wake: the PC is in use, no interactive session is open, and something (a trigger rule, an external tool) says it is time.
pub fn chore(reason: &str, note: &Value) -> &'static str {
    let w = worker::load();
    let recent = match gate(&w, true, false, false) {
        Ok(r) => r,
        Err(why) => return why,
    };
    let acts = worker::recent_activity(8);
    // the prompt is a file (placeholders {reason} {note} {activity} {cli}): yours in the data folder, else the shipped docs/chore-prompt.md
    let template = [paths::home_dir(), paths::app_dir()]
        .iter()
        .flat_map(|d| ["chore-prompt.md", "docs/chore-prompt.md"].iter().map(move |r| d.join(r)))
        .find_map(|p| fs::read_to_string(p).ok().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "CHORE WAKE ({reason}). Note: {note}\nRecent activity:\n{activity}\nCLI: {cli}\nDo at most 2 quick chores, then stop.".to_string());
    let prompt = template
        .replace("{reason}", reason)
        .replace("{note}", &json!(note).to_string())
        .replace("{activity}", &if acts.is_empty() { "(nothing tracked yet)".to_string() } else { acts.join("\n") })
        .replace("{cli}", &exe_path());
    spawn(&w, &prompt, recent, true)
}
