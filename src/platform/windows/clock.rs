//! Local wall-clock time on Windows. Used by util.rs and learning/annoyance.rs.

use windows::Win32::System::SystemInformation::GetLocalTime;

/// Minutes since local midnight.
pub fn local_minutes() -> u32 {
    let t = unsafe { GetLocalTime() };
    t.wHour as u32 * 60 + t.wMinute as u32
}

/// "YYYY-MM-DD HH:MM" in local time.
pub fn stamp() -> String {
    let t = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02} {:02}:{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute)
}
