//! What the desktop looks like right now (Windows API here; macOS in platform/macos/desktop.rs): the visible windows, the monitors, the active window and whether it is fullscreen, its exe and title.

#[cfg(windows)]
use std::ffi::c_void;
#[cfg(windows)]
use windows::core::BOOL;
#[cfg(windows)]
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
#[cfg(windows)]
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::*;
#[cfg(windows)]
use windows::Win32::System::Threading::GetCurrentProcessId;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::*;

use super::R;

#[cfg(windows)]
fn hwnd_rect(hwnd: HWND) -> Option<R> {
    let mut r = RECT::default();
    unsafe {
        if DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut r as *mut _ as *mut c_void, std::mem::size_of::<RECT>() as u32).is_err() {
            GetWindowRect(hwnd, &mut r).ok()?;
        }
    }
    Some(R::from(r))
}

#[cfg(windows)]
fn is_cloaked(hwnd: HWND) -> bool {
    let mut c: u32 = 0;
    unsafe {
        DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut c as *mut _ as *mut c_void, 4).is_ok() && c != 0
    }
}

#[cfg(windows)]
unsafe extern "system" fn collect(hwnd: HWND, lp: LPARAM) -> BOOL {
    let out = &mut *(lp.0 as *mut Vec<R>);
    if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() || is_cloaked(hwnd) {
        return BOOL(1);
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == GetCurrentProcessId() {
        return BOOL(1);
    }
    let mut cls = [0u16; 64];
    let n = GetClassNameW(hwnd, &mut cls) as usize;
    let cls = String::from_utf16_lossy(&cls[..n]);
    if matches!(cls.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd") {
        return BOOL(1);
    }
    if let Some(r) = hwnd_rect(hwnd) {
        if r.w() > 80 && r.h() > 40 {
            out.push(r);
        }
    }
    BOOL(1)
}

#[cfg(windows)]
unsafe extern "system" fn collect_monitors(mon: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> BOOL {
    let out = &mut *(lp.0 as *mut Vec<(HMONITOR, R)>);
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    if GetMonitorInfoW(mon, &mut mi).as_bool() {
        out.push((mon, R::from(mi.rcWork)));
    }
    BOOL(1)
}

#[derive(Clone)]
pub struct Foreground {
    pub rect: Option<R>,
    pub work: R,
    pub fullscreen: bool,
}

#[cfg(windows)]
/// The active window and the work area of the monitor it is on (primary if none). Uncached: callers use `foreground()`.
pub fn foreground_now() -> Foreground {
    unsafe {
        let mut mons: Vec<(HMONITOR, R)> = vec![];
        let _ = EnumDisplayMonitors(None, None, Some(collect_monitors), LPARAM(&mut mons as *mut _ as isize));
        let fg = GetForegroundWindow();
        // Our own window can be "active" while you type in a field; it is not something to avoid.
        let mut fg_pid = 0u32;
        GetWindowThreadProcessId(fg, Some(&mut fg_pid));
        let own = fg_pid == GetCurrentProcessId();
        // Clicking one of our no-activate windows can briefly make the desktop/taskbar the "active" window. Those are
        // not something you are working in: treat them as no active window (never as "a window covering everything").
        let mut cls0 = [0u16; 64];
        let n0 = if fg.0.is_null() { 0 } else { GetClassNameW(fg, &mut cls0) as usize };
        let shell = matches!(String::from_utf16_lossy(&cls0[..n0]).as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd" | "NotifyIconOverflowWindow" | "TopLevelWindowForOverflowXamlIsland" | "XamlExplorerHostIslandWindow");
        let fg_rect = if fg.0.is_null() || own || shell { None } else { hwnd_rect(fg) };
        let mon = if fg.0.is_null() { MonitorFromWindow(HWND(std::ptr::null_mut()), MONITOR_DEFAULTTOPRIMARY) } else { MonitorFromWindow(fg, MONITOR_DEFAULTTOPRIMARY) };
        let work = mons.iter().find(|(m, _)| *m == mon).map(|(_, r)| *r).or(mons.first().map(|(_, r)| *r)).unwrap_or(R { l: 0, t: 0, r: 1920, b: 1080 });
        // Fullscreen = the active window covers its entire monitor (not just the work area): a game or a video.
        let mut full = false;
        if let Some(r) = fg_rect {
            let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
            if GetMonitorInfoW(mon, &mut mi).as_bool() {
                let m = R::from(mi.rcMonitor);
                full = r.l <= m.l && r.t <= m.t && r.r >= m.r && r.b >= m.b && r.area() > 0;
                // The desktop itself and the shell are not "fullscreen apps".
                let mut cls = [0u16; 64];
                let n = GetClassNameW(fg, &mut cls) as usize;
                let cls = String::from_utf16_lossy(&cls[..n]);
                if matches!(cls.as_str(), "Progman" | "WorkerW") {
                    full = false;
                }
            }
        }
        Foreground { rect: fg_rect, work, fullscreen: full }
    }
}

#[cfg(windows)]
/// Every visible, non-minimised top-level window except ours and the shell (physical px). Uncached: callers use `windows()`.
pub fn windows_now() -> Vec<R> {
    let mut wins: Vec<R> = vec![];
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut wins as *mut _ as isize));
    }
    wins
}

// One snapshot per placement: a single placement asks for the active window and the window list many times (regions, docking, covers,
// work areas). Each answer is kept for SNAPSHOT, so every check in one placement sees the same screen and the OS is asked once.
// `invalidate` drops it the moment the active window changes (host on_foreground).
const SNAPSHOT: std::time::Duration = std::time::Duration::from_millis(150);
static FG_SNAP: std::sync::Mutex<Option<(std::time::Instant, Foreground)>> = std::sync::Mutex::new(None);
static WIN_SNAP: std::sync::Mutex<Option<(std::time::Instant, Vec<R>)>> = std::sync::Mutex::new(None);

fn snapshot<T: Clone>(slot: &std::sync::Mutex<Option<(std::time::Instant, T)>>, now: fn() -> T) -> T {
    let Ok(mut g) = slot.lock() else { return now() };
    if let Some((at, v)) = g.as_ref() {
        if at.elapsed() < SNAPSHOT {
            return v.clone();
        }
    }
    let v = now();
    *g = Some((std::time::Instant::now(), v.clone()));
    v
}

/// The active window and the work area of its monitor (snapshot, see above).
pub fn foreground() -> Foreground {
    snapshot(&FG_SNAP, foreground_now)
}

/// Every visible window except ours and the shell (snapshot, see above).
pub fn windows() -> Vec<R> {
    snapshot(&WIN_SNAP, windows_now)
}

/// The desktop changed (a new active window): the next question asks the OS again.
pub fn invalidate() {
    if let Ok(mut g) = FG_SNAP.lock() {
        *g = None;
    }
    if let Ok(mut g) = WIN_SNAP.lock() {
        *g = None;
    }
}

/// Free height under the active window (physical px); 0 = no neighbour (or fullscreen). Lets a long list flow into the strip below it.
pub fn dock_below() -> i32 {
    let fg = foreground();
    match fg.rect {
        Some(f) if !fg.fullscreen => (fg.work.b - f.b).max(0),
        _ => 0,
    }
}

/// Free width beside the active window (physical px): the room a docked surface may use, on the roomier side. 0 = no neighbour.
pub fn dock_room() -> i32 {
    let fg = foreground();
    match fg.rect {
        Some(f) if !fg.fullscreen => (fg.work.r - f.r).max(f.l - fg.work.l).max(0),
        _ => 0,
    }
}

#[cfg(windows)]
/// Work area of the monitor that contains this point (primary if none).
pub fn work_at(x: i32, y: i32) -> R {
    unsafe {
        let mon = MonitorFromPoint(windows::Win32::Foundation::POINT { x, y }, MONITOR_DEFAULTTOPRIMARY);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(mon, &mut mi).as_bool() { R::from(mi.rcWork) } else { R { l: 0, t: 0, r: 1920, b: 1080 } }
    }
}

/// Nudge a rect so it sits fully inside the work area of the monitor it mostly lives on. Returns the new top-left.
pub fn clamp_inside(r: &R) -> (i32, i32) {
    let work = work_at((r.l + r.r) / 2, (r.t + r.b) / 2);
    let m = 8;
    let x = r.l.min(work.r - r.w() - m).max(work.l + m);
    let y = r.t.min(work.b - r.h() - m).max(work.t + m);
    (x, y)
}

#[cfg(windows)]
/// Executable name (lowercase) of the active window's process, e.g. "chrome.exe".
pub fn fg_exe() -> Option<String> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, Some(&mut pid));
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 520];
        let mut n = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut n).is_ok();
        let _ = CloseHandle(h);
        if !ok {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..n as usize]);
        path.rsplit('\\').next().map(|s| s.to_lowercase())
    }
}

#[cfg(windows)]
/// Title of the active window (what you are looking at), if it has one.
pub fn fg_title() -> Option<String> {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return None;
        }
        let mut buf = [0u16; 256];
        let n = GetWindowTextW(fg, &mut buf);
        (n > 0).then(|| String::from_utf16_lossy(&buf[..n as usize]))
    }
}

/// Does this rect sit on top of the active window right now?
pub fn covers_foreground(r: &R) -> bool {
    foreground().rect.map(|f| r.overlap(&f) > 0).unwrap_or(false)
}

// macOS: the same functions live in platform/macos/desktop.rs.
#[cfg(target_os = "macos")]
pub use crate::platform::macos::desktop::{fg_exe, fg_title, foreground_now, windows_now, work_at};
