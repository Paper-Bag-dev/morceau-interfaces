//! The local socket the CLI and the host talk over (see src/ipc/transport.rs). Same functions on every OS: name, listen, Listener::serve, connect.

use crate::ipc::transport::Conn;
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
