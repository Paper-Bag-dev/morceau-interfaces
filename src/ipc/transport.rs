//! How the CLI reaches the host (docs/protocol.md "Transport"). The default is a LOCAL SOCKET only your user account can open, never a
//! network port:
//!   macOS    a Unix socket file in the runtime folder (`morc-<port>.sock`, mode 0600)
//!   Windows  a named pipe `\\.\pipe\morc-<user>-<port>` that refuses remote clients
//! `MORC_TRANSPORT=tcp` keeps the old localhost TCP port (127.0.0.1:<port>). The protocol above it (one JSON line in, one out, token-checked)
//! is the same on every transport; the token stays as a second lock.

use std::io::{self, Read, Write};
use std::time::Duration;

use crate::platform::os::pipe as os;

/// One connection, whatever carries it.
pub trait Conn: Read + Write + Send {}
impl<T: Read + Write + Send> Conn for T {}

fn tcp() -> bool {
    std::env::var("MORC_TRANSPORT").map(|v| v.eq_ignore_ascii_case("tcp")).unwrap_or(false)
}

/// Where the transport lives, for logs and errors.
pub fn describe() -> String {
    if tcp() {
        return format!("tcp {}:{}", crate::paths::HOST, crate::paths::port());
    }
    os::name()
}

pub enum Listener {
    Tcp(std::net::TcpListener),
    Local(os::Listener),
}

/// Bind the host's endpoint. Err = another host already owns it (or it cannot be made).
pub fn listen() -> io::Result<Listener> {
    if tcp() {
        return std::net::TcpListener::bind((crate::paths::HOST, crate::paths::port())).map(Listener::Tcp);
    }
    os::listen().map(Listener::Local)
}

impl Listener {
    /// Accept forever, handing each connection to `f` on its own thread.
    pub fn serve(self, f: impl Fn(Box<dyn Conn>) + Send + Sync + Clone + 'static) {
        std::thread::spawn(move || match self {
            Listener::Tcp(l) => {
                for s in l.incoming().flatten() {
                    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
                    let f = f.clone();
                    std::thread::spawn(move || f(Box::new(s)));
                }
            }
            Listener::Local(l) => l.serve(f),
        });
    }
}

/// Open a connection to the host.
pub fn connect() -> io::Result<Box<dyn Conn>> {
    if tcp() {
        let s = std::net::TcpStream::connect((crate::paths::HOST, crate::paths::port()))?;
        s.set_read_timeout(Some(Duration::from_secs(8))).ok();
        return Ok(Box::new(s));
    }
    os::connect()
}

/// Is a host answering? (cheap: connects and drops)
pub fn alive() -> bool {
    if tcp() {
        let addr = format!("{}:{}", crate::paths::HOST, crate::paths::port());
        return std::net::TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(200)).is_ok();
    }
    os::connect().is_ok()
}

