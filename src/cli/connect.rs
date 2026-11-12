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

/// Make sure a host is up, starting a hidden detached one if needed.
pub(super) fn ensure_host() -> Result<(), String> {
    if crate::transport::alive() && paths::token_file().exists() {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("host").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    crate::platform::os::process::detach(&mut cmd); // detached, windowless, no inherited handles (platform/<os>/process.rs)
    cmd.spawn().map_err(|e| e.to_string())?;
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(6) {
        if crate::transport::alive() && paths::token_file().exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("host did not start".into())
}

