//! Talking to the host: send a request over the transport (ipc/transport.rs: a local socket by default), and start the host detached when
//! it is not running. Split out of cli/mod.rs.

use super::*;

pub(super) fn send(req: Value) -> Result<Value, String> {
    let token = std::fs::read_to_string(paths::token_file()).map_err(|_| "no token".to_string())?;
    let mut s = crate::transport::connect().map_err(|e| format!("{} ({})", e, crate::transport::describe()))?;
    writeln!(s, "{}", json!({"token": token.trim(), "req": req})).map_err(|e| e.to_string())?;
    s.flush().ok();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| format!("bad reply: {e}"))
}

/// Make sure a host is up, starting a hidden detached one if needed. Every failure says why and where the log is.
pub(super) fn ensure_host() -> Result<(), String> {
    if crate::transport::alive() && paths::token_file().exists() {
        return Ok(());
    }
    if crate::transport::alive() {
        // something answers on the endpoint but it is not a morc we can talk to (no token in this runtime folder)
        return Err(format!(
            "{} is held by another program (or a morc host for another runtime folder {}). Close it, or run `morc stop` from that setup, then try again",
            crate::transport::describe(),
            paths::runtime_dir().display()
        ));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("host").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    crate::platform::os::process::detach(&mut cmd); // detached, windowless, no inherited handles (platform/<os>/process.rs)
    cmd.spawn().map_err(|e| e.to_string())?;
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(10) {
        if crate::transport::alive() && paths::token_file().exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let log = paths::runtime_dir().join("host.log");
    let last = std::fs::read_to_string(&log).ok().and_then(|s| s.lines().last().map(|l| l.to_string())).unwrap_or_default();
    Err(format!(
        "the host did not start within 10 s. Last line of {}: {}",
        log.display(),
        if last.is_empty() { "(empty: the host could not even write its log)".to_string() } else { last }
    ))
}

