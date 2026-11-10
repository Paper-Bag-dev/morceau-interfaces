//! The ONE persistent background agent: start it, queue later wakes into its input, let it sleep after linger_s, launch the claude -p process. Split out of wake/mod.rs.

use super::*;

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

pub(super) fn spawn(w: &worker::Worker, prompt: &str, mut recent: Vec<u128>, chore: bool) -> &'static str {
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
    let child_path = std::env::join_paths(std::iter::once(std::path::PathBuf::from(&exe_dir)).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())))
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    // max_turns 0 (the default) = no turn cap: the agent works until it is done. A positive number caps it.
    let turns = w.max_turns.to_string();
    let mut cmd = Command::new("claude");
    // Read/Write: it builds spec files to pass to `morc show --file` (inline JSON loses its quotes on Windows)
    cmd.args(["-p", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose", "--model", &w.model]);
    // its own scratch folder: the temp spec files it writes never end up in the repo or your data folder
    let work = paths::runtime_dir().join("agent");
    let _ = fs::create_dir_all(&work);
    // it may read the install and data folders (docs/chips/INDEX.md, the ladder, the cards): its own working folder is only a scratch space
    let (app_dir, home_dir) = (paths::app_dir().display().to_string(), paths::home_dir().display().to_string());
    cmd.args(["--add-dir", &app_dir, "--add-dir", &home_dir]);
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

