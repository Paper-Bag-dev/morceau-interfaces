//! How the CLI reaches the host (docs/protocol.md "Transport"). The default is a LOCAL SOCKET only your user account can open, never a
//! network port:
//!   macOS    a Unix socket file in the runtime folder (`morc-<port>.sock`, mode 0600)
//!   Windows  a named pipe `\\.\pipe\morc-<user>-<port>` that refuses remote clients
//! `MORC_TRANSPORT=tcp` keeps the old localhost TCP port (127.0.0.1:<port>). The protocol above it (one JSON line in, one out, token-checked)
//! is the same on every transport; the token stays as a second lock.

use std::io::{self, Read, Write};
use std::time::Duration;

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

#[cfg(unix)]
mod os {
    use super::Conn;
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
}

#[cfg(windows)]
mod os {
    use super::Conn;
    use std::fs::File;
    use std::io;
    use std::os::windows::io::FromRawHandle;
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{GetLastError, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, INVALID_HANDLE_VALUE};
    use windows::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
    use windows::Win32::System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT};

    fn path() -> String {
        let user: String = std::env::var("USERNAME").unwrap_or_default().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        format!(r"\\.\pipe\morc-{}-{}", user, crate::paths::port())
    }

    pub fn name() -> String {
        format!("named pipe {}", path())
    }

    /// The first pipe instance, made at bind time so a second host fails at once (FILE_FLAG_FIRST_PIPE_INSTANCE).
    pub struct Listener(isize);

    fn instance(first: bool) -> io::Result<isize> {
        let open = if first { PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE } else { PIPE_ACCESS_DUPLEX };
        let h = unsafe {
            CreateNamedPipeW(&HSTRING::from(path()), open, PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS, PIPE_UNLIMITED_INSTANCES, 65536, 65536, 0, None)
        };
        if h == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(h.0 as isize)
    }

    pub fn listen() -> io::Result<Listener> {
        instance(true).map(Listener)
    }

    impl Listener {
        pub fn serve(self, f: impl Fn(Box<dyn Conn>) + Send + Sync + Clone + 'static) {
            let mut next = Some(self.0);
            loop {
                let h = match next.take() {
                    Some(h) => h,
                    None => match instance(false) {
                        Ok(h) => h,
                        Err(_) => {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                    },
                };
                let handle = windows::Win32::Foundation::HANDLE(h as *mut core::ffi::c_void);
                // waits for a client; a client that connected between create and connect reports ERROR_PIPE_CONNECTED, which is fine
                let ok = unsafe { ConnectNamedPipe(handle, None) }.is_ok() || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
                // SAFETY: the pipe handle is owned by this File from here on and closed when it drops
                let file = unsafe { File::from_raw_handle(h as *mut core::ffi::c_void) };
                if ok {
                    let f = f.clone();
                    std::thread::spawn(move || f(Box::new(file)));
                }
            }
        }
    }

    pub fn connect() -> io::Result<Box<dyn Conn>> {
        for _ in 0..50 {
            match std::fs::OpenOptions::new().read(true).write(true).open(path()) {
                Ok(f) => return Ok(Box::new(f)),
                // every instance is busy serving another call for a moment: try again shortly
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => std::thread::sleep(std::time::Duration::from_millis(20)),
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(io::ErrorKind::TimedOut, "the host's pipe stayed busy"))
    }
}
