//! Talking to the host: send a request over localhost, and start the host detached when it is not running. Split out of cli/mod.rs.

use super::*;

pub(super) fn send(req: Value) -> Result<Value, String> {
    let token = std::fs::read_to_string(paths::token_file()).map_err(|_| "no token".to_string())?;
    let mut s = TcpStream::connect((paths::HOST, paths::PORT)).map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(Duration::from_secs(8))).ok();
    writeln!(s, "{}", json!({"token": token.trim(), "req": req})).map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| format!("bad reply: {e}"))
}

/// Make sure a host is up, starting a hidden detached one if needed.
pub(super) fn ensure_host() -> Result<(), String> {
    if TcpStream::connect_timeout(&format!("{}:{}", paths::HOST, paths::PORT).parse().unwrap(), Duration::from_millis(200)).is_ok() && paths::token_file().exists() {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // The detached host must not inherit the pipes this CLI was started with. Windows gives a child EVERY inheritable handle of its parent,
    // not just the std ones we null below, so a shell, a script or an agent capturing this CLI's output would wait for the host to exit
    // (it never does) and the first command would appear to hang for minutes. Mark our own std handles non-inheritable before spawning.
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::{SetHandleInformation, HANDLE_FLAGS, HANDLE_FLAG_INHERIT};
        use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
        for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            if let Ok(h) = GetStdHandle(id) {
                if !h.is_invalid() {
                    let _ = SetHandleInformation(h, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
                }
            }
        }
    }
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("host").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0008); // CREATE_NO_WINDOW | DETACHED_PROCESS
    }
    // macOS: its own process group, so closing the terminal (SIGHUP to the foreground group) or Ctrl+C does not take the host down.
    // Unix children only inherit descriptors without close-on-exec, and Rust opens everything close-on-exec, so no handle dance is needed.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd.spawn().map_err(|e| e.to_string())?;
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(6) {
        if TcpStream::connect_timeout(&format!("{}:{}", paths::HOST, paths::PORT).parse().unwrap(), Duration::from_millis(200)).is_ok() && paths::token_file().exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("host did not start".into())
}

