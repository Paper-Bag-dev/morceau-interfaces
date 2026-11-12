//! The local socket the CLI and the host talk over (see src/ipc/transport.rs). Same functions on every OS: name, listen, Listener::serve, connect.

use crate::ipc::transport::Conn;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::Duration;

/// The runtime folder, unless that path is too long for a socket (macOS allows 104 bytes): then the per-user temp folder ($TMPDIR on
/// macOS is private to the account, mode 0700), so "only this user" still holds.
fn path() -> PathBuf {
    let name = format!("morc-{}.sock", crate::paths::port());
    let p = crate::paths::runtime_dir().join(&name);
    if p.as_os_str().len() < 100 {
        p
    } else {
        std::env::temp_dir().join(name)
    }
}

pub fn name() -> String {
    format!("unix socket {}", path().display())
}

pub struct Listener(UnixListener);

pub fn listen() -> io::Result<Listener> {
    let p = path();
    if p.exists() {
        if UnixStream::connect(&p).is_ok() {
            return Err(io::Error::new(io::ErrorKind::AddrInUse, "a host is already running"));
        }
        let _ = std::fs::remove_file(&p);   // left behind by a host that died
    }
    let l = UnixListener::bind(&p)?;
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600))?;   // only this user may connect
    Ok(Listener(l))
}

impl Listener {
    pub fn serve(self, f: impl Fn(Box<dyn Conn>) + Send + Sync + Clone + 'static) {
        for s in self.0.incoming().flatten() {
            let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
            let f = f.clone();
            std::thread::spawn(move || f(Box::new(s)));
        }
    }
}

pub fn connect() -> io::Result<Box<dyn Conn>> {
    let s = UnixStream::connect(path())?;
    s.set_read_timeout(Some(Duration::from_secs(8))).ok();
    Ok(Box::new(s))
}
