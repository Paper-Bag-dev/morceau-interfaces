//! Waking a headless agent (the chore agent, Haiku by default), cheapest and most conservative first.
//! Every wake, whatever asked for it, passes the same gates in this order:
//!   1. only if prefs/worker.json says defer_to_session: a live interactive session touched morc recently -> rest. By default the agent and the session work together
//!   2. the PC is idle (no keyboard/mouse input for idle_s)      -> rest
//!   3. hard rate limit (max_per_hour, min_gap_s)                -> rest
//!   4. chores only: the chore cooldown has not passed           -> rest
//! Passing them starts ONE headless `claude -p` (model, persona, turn cap from prefs/worker.json) that reads its work from stdin. Later wakes are
//! written to that same process and queue behind what it is doing. It stays awake `linger_s` after its last turn, then the host closes its input: its sleep.
//! Two ways in: a user action on a widget (maybe_wake) and a chore (chore: from a trigger rule, an external tool, or `morc chore <reason>`).

use crate::{inbox, paths, util, worker};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

/// `direct` = the user just clicked a button that asked for background work (payload "bg": true). They are plainly at the PC, so the
/// live-session and idle gates are skipped; the enabled switch and the rate limit still apply.
fn gate(w: &worker::Worker, chore: bool, direct: bool, click: bool) -> Result<Vec<u128>, &'static str> {
    if !w.enabled {
        return Err("disabled");
    }
    if !direct {
        if w.defer_to_session {
            if let Some(age) = inbox::session_age_s() {
                if age < w.session_fresh_s {
                    return Err("session-live");
                }
            }
        }
        if worker::idle_seconds() > w.idle_s {
            return Err("pc-idle");
        }
    }
    let now = util::now_ms();
    let log = fs::read_to_string(paths::wake_log_file()).unwrap_or_default();
    let recent: Vec<u128> = log.lines().filter_map(|l| l.trim().parse().ok()).filter(|t| now.saturating_sub(*t) < 3_600_000).collect();
    // The rate limit guards against the host waking the agent by itself (triggers, prefetch). A click the user just made is never limited:
    // they asked, and a limit that eats their click leaves skeletons on screen.
    if !click && (recent.len() >= w.max_per_hour || recent.iter().any(|t| now.saturating_sub(*t) < (w.min_gap_s as u128) * 1000)) {
        return Err("rate-limited");
    }
    if chore && (now.saturating_sub(worker::last_chore_ms()) as f64) < w.chore_cooldown_min * 60_000.0 {
        return Err("chore-cooldown");
    }
    Ok(recent)
}

/// ONE agent for everything, and it stays awake for a while.
/// The first wake starts a `claude -p` that reads its work from stdin (stream-json). Every later wake is written to that same process, so a
/// burst of clicks never starts a burst of agents: they queue up in its input and it works through them one after another. When it has
/// finished everything and nothing new came for `linger_s`, the host closes its input and it exits: that is its sleep.
struct Agent {
    alive: bool,
    stdin: Option<std::process::ChildStdin>,
    /// messages written whose turn has not finished yet
    in_flight: usize,
    last_done: std::time::Instant,
}
static AGENT: std::sync::Mutex<Option<Agent>> = std::sync::Mutex::new(None);

fn user_line(prompt: &str) -> String {
    let mut s = json!({"type": "user", "message": {"role": "user", "content": prompt}}).to_string();
    s.push('\n');
    s
}

fn spawn(w: &worker::Worker, prompt: &str, mut recent: Vec<u128>, chore: bool) -> &'static str {
    use std::io::Write;
    if std::env::var("MORC_WAKE_DRY").is_ok() {
        let _ = fs::write(paths::runtime_dir().join("wake.dry.txt"), prompt);
        return "dry-run";
    }
    // already awake: hand it the work, it queues behind what it is doing
    {
        let mut g = AGENT.lock().unwrap();
        if let Some(a) = g.as_mut() {
            if a.alive {
                if let Some(si) = a.stdin.as_mut() {
                    if si.write_all(user_line(prompt).as_bytes()).and_then(|_| si.flush()).is_ok() {
                        a.in_flight += 1;
                        a.last_done = std::time::Instant::now(); // any new interaction restarts the awake timer
                        return "queued";
                    }
                }
                a.alive = false; // its input is gone: start a new one below
            }
        }
    }
    recent.push(util::now_ms());
    let _ = fs::write(paths::wake_log_file(), recent.iter().map(|t| t.to_string()).collect::<Vec<_>>().join("\n"));
    if chore {
        worker::mark_chore();
    }
    let mut child = match launch(w) {
        Ok(c) => c,
        Err(_) => return "spawn-failed",
    };
    let Some(mut stdin) = child.stdin.take() else { return "spawn-failed" };
    let stdout = child.stdout.take();
    if stdin.write_all(user_line(prompt).as_bytes()).and_then(|_| stdin.flush()).is_err() {
        let _ = child.kill();
        return "spawn-failed";
    }
    *AGENT.lock().unwrap() = Some(Agent { alive: true, stdin: Some(stdin), in_flight: 1, last_done: std::time::Instant::now() });

    // reader: copy what the agent says to agent.log, and notice when a turn finishes
    if let Some(out) = stdout {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let mut log = fs::OpenOptions::new().create(true).append(true).open(paths::runtime_dir().join("agent.log")).ok();
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if line.contains("\"type\":\"result\"") {
                    if let Some(a) = AGENT.lock().unwrap().as_mut() {
                        a.in_flight = a.in_flight.saturating_sub(1);
                        a.last_done = std::time::Instant::now();
                    }
                    if let (Some(f), Ok(v)) = (log.as_mut(), serde_json::from_str::<Value>(&line)) {
                        let _ = writeln!(f, "{}", v["result"].as_str().unwrap_or("").trim());
                    }
                } else if line.contains("\"is_error\":true") {
                    // a refused or failed tool call: keep one short line (what was refused), not the whole event
                    if let (Some(f), Ok(v)) = (log.as_mut(), serde_json::from_str::<Value>(&line)) {
                        let why = v["message"]["content"][0]["content"].as_str().unwrap_or("error");
                        let _ = writeln!(f, "tool error: {}", why.chars().take(200).collect::<String>());
                    }
                }
            }
            if let Some(a) = AGENT.lock().unwrap().as_mut() {
                a.alive = false; // the process ended (slept, crashed or was killed)
            }
        });
    }
    // sleeper: once idle for linger_s with nothing in flight, close its input so it exits
    let linger = std::time::Duration::from_secs(w.linger_s.max(5));
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let mut g = AGENT.lock().unwrap();
            let Some(a) = g.as_mut() else { break };
            if !a.alive {
                break;
            }
            if a.in_flight == 0 && a.last_done.elapsed() >= linger {
                a.stdin = None; // dropping stdin closes it: the agent finishes and exits
                a.alive = false;
                break;
            }
        }
        let _ = child.wait();
    });
    "headless-spawned"
}

fn launch(w: &worker::Worker) -> std::io::Result<std::process::Child> {
    let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_else(|_| "morc".into());
    // the agent's system prompt: your copy in the data folder wins over the shipped one in the install folder
    let persona = [paths::home_dir(), paths::app_dir()].iter().find_map(|d| fs::read_to_string(d.join(&w.persona)).ok()).unwrap_or_default();
    // The permission rule matches the command text, and a Windows path with backslashes never matched ("requires approval" for every
    // call, found in agent.log). So the agent calls plain `morc` (its exe folder is put on the child's PATH) and the rule says the same.
    let allow = "Bash(morc:*)".to_string();
    let allow_ps = "PowerShell(morc:*)".to_string(); // the agent may reach for either shell; both are limited to the morc CLI
    let exe_dir = std::path::Path::new(&exe).parent().map(|p| p.display().to_string()).unwrap_or_default();
    let child_path = format!("{exe_dir};{}", std::env::var("PATH").unwrap_or_default());
    // max_turns 0 (the default) = no turn cap: the agent works until it is done. A positive number caps it.
    let turns = w.max_turns.to_string();
    let mut cmd = Command::new("claude");
    // Read/Write: it builds spec files to pass to `morc show --file` (inline JSON loses its quotes on Windows)
    cmd.args(["-p", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose", "--model", &w.model]);
    // its own scratch folder: the temp spec files it writes never end up in the repo or your data folder
    let work = paths::runtime_dir().join("agent");
    let _ = fs::create_dir_all(&work);
    cmd.args(["--allowedTools", &allow, &allow_ps, "Read", "Write", "--append-system-prompt", &persona])
        .current_dir(&work)
        .env("PATH", child_path)
        .env("MORC_HEADLESS", "1") // so its own hooks do not steal the interactive session's unread events
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    if w.max_turns > 0 {
        cmd.args(["--max-turns", &turns]);
    }
    util::hidden(&mut cmd).spawn()
}

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
