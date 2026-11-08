//! Waking a headless agent (the chore agent, Haiku by default), cheapest and most conservative first.
//! Every wake, whatever asked for it, passes the same gates in this order:
//!   1. a live interactive Claude session touched morc recently -> rest (a hook / Monitor delivers the event to that session)
//!   2. the PC is idle (no keyboard/mouse input for idle_s)      -> rest
//!   3. hard rate limit (max_per_hour, min_gap_s)                -> rest
//!   4. chores only: the chore cooldown has not passed           -> rest
//! Passing them spawns a headless `claude -p` (model, persona, turn cap from prefs/worker.json). It exits when done: that is its sleep.
//! Two ways in: a user action on a widget (maybe_wake) and a chore (chore: from a trigger rule, an external tool, or `morc chore <reason>`).

use crate::{inbox, paths, util, worker};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

/// `direct` = the user just clicked a button that asked for background work (payload "bg": true). They are plainly at the PC, so the
/// live-session and idle gates are skipped; the enabled switch and the rate limit still apply.
fn gate(w: &worker::Worker, chore: bool, direct: bool) -> Result<Vec<u128>, &'static str> {
    if !w.enabled {
        return Err("disabled");
    }
    if !direct {
        if let Some(age) = inbox::session_age_s() {
            if age < w.session_fresh_s {
                return Err("session-live");
            }
        }
        if worker::idle_seconds() > w.idle_s {
            return Err("pc-idle");
        }
    }
    let now = util::now_ms();
    let log = fs::read_to_string(paths::wake_log_file()).unwrap_or_default();
    let recent: Vec<u128> = log.lines().filter_map(|l| l.trim().parse().ok()).filter(|t| now.saturating_sub(*t) < 3_600_000).collect();
    if recent.len() >= w.max_per_hour || recent.iter().any(|t| now.saturating_sub(*t) < (w.min_gap_s as u128) * 1000) {
        return Err("rate-limited");
    }
    if chore && (now.saturating_sub(worker::last_chore_ms()) as f64) < w.chore_cooldown_min * 60_000.0 {
        return Err("chore-cooldown");
    }
    Ok(recent)
}

fn spawn(w: &worker::Worker, prompt: &str, mut recent: Vec<u128>, chore: bool) -> &'static str {
    if std::env::var("MORC_WAKE_DRY").is_ok() {
        let _ = fs::write(paths::runtime_dir().join("wake.dry.txt"), prompt);
        return "dry-run";
    }
    let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "morc".into());
    // the agent's system prompt: your copy in the data folder wins over the shipped one in the install folder
    let persona = [paths::home_dir(), paths::app_dir()].iter().find_map(|d| fs::read_to_string(d.join(&w.persona)).ok()).unwrap_or_default();
    recent.push(util::now_ms());
    let _ = fs::write(paths::wake_log_file(), recent.iter().map(|t| t.to_string()).collect::<Vec<_>>().join("\n"));
    if chore {
        worker::mark_chore();
    }
    let allow = format!("Bash({exe}:*)");
    let turns = w.max_turns.to_string();
    // the agent's own words and errors go to <runtime>/agent.log, so a silent failure can be read afterwards
    let log = fs::OpenOptions::new().create(true).append(true).open(paths::runtime_dir().join("agent.log"));
    let mut cmd = Command::new("claude");
    // Read/Write: it builds spec files to pass to `morc show --file` (inline JSON loses its quotes on Windows)
    cmd.args(["-p", prompt, "--model", &w.model, "--max-turns", &turns, "--allowedTools", &allow, "Read", "Write", "--append-system-prompt", &persona])
        .current_dir(paths::home_dir());
    if let Ok(f) = log {
        if let Ok(f2) = f.try_clone() {
            cmd.stdout(f).stderr(f2);
        }
    }
    let spawned = util::hidden(&mut cmd).spawn();
    if spawned.is_ok() { "headless-spawned" } else { "spawn-failed" }
}

fn exe_path() -> String {
    std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "morc".into())
}

/// The user acted on a widget that needs the agent.
pub fn maybe_wake(payload: &Value) -> &'static str {
    let w = worker::load();
    let direct = payload["bg"].as_bool().unwrap_or(false);
    let recent = match gate(&w, false, direct) {
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
    let recent = match gate(&w, true, false) {
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
