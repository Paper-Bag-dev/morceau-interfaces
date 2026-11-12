//! What the desktop looks like right now on Windows: the visible windows, the monitors, the active window and whether it is fullscreen,
//! its exe and title. placement/desktop.rs re-exports these; its snapshot wrappers (`foreground()`, `windows()`) call the `_now` ones.

use crate::placement::{Display, Foreground, SceneWin, R};
use std::ffi::c_void;
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::*;

fn hwnd_rect(hwnd: HWND) -> Option<R> {
    let mut r = RECT::default();
    unsafe {
        if DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut r as *mut _ as *mut c_void, std::mem::size_of::<RECT>() as u32).is_err() {
            GetWindowRect(hwnd, &mut r).ok()?;
        }
    }
    Some(rect(r))
}

fn is_cloaked(hwnd: HWND) -> bool {
    let mut c: u32 = 0;
    unsafe {
        DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut c as *mut _ as *mut c_void, 4).is_ok() && c != 0
    }
}

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

unsafe extern "system" fn collect_monitors(mon: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> BOOL {
    let out = &mut *(lp.0 as *mut Vec<(HMONITOR, R)>);
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    if GetMonitorInfoW(mon, &mut mi).as_bool() {
        out.push((mon, rect(mi.rcWork)));
    }
    BOOL(1)
}

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
                let m = rect(mi.rcMonitor);
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

/// Every visible, non-minimised top-level window except ours and the shell (physical px). Uncached: callers use `windows()`.
pub fn windows_now() -> Vec<R> {
    let mut wins: Vec<R> = vec![];
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut wins as *mut _ as isize));
    }
    wins
}

/// Work area of the monitor that contains this point (primary if none).
pub fn work_at(x: i32, y: i32) -> R {
    unsafe {
        let mon = MonitorFromPoint(windows::Win32::Foundation::POINT { x, y }, MONITOR_DEFAULTTOPRIMARY);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(mon, &mut mi).as_bool() { rect(mi.rcWork) } else { R { l: 0, t: 0, r: 1920, b: 1080 } }
    }
}

/// Executable name (lowercase) of the active window's process, e.g. "chrome.exe".
pub fn fg_exe() -> Option<String> {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, Some(&mut pid));
        exe_of(pid)
    }
}

/// Executable name (lowercase) of a process, e.g. "chrome.exe".
fn exe_of(pid: u32) -> Option<String> {
    use windows::core::PWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
    unsafe {
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

fn rect(r: RECT) -> R {
    R { l: r.left, t: r.top, r: r.right, b: r.bottom }
}

/// (frame, work area) of each monitor.
unsafe extern "system" fn collect_frames(mon: HMONITOR, _: HDC, _: *mut RECT, lp: LPARAM) -> BOOL {
    let out = &mut *(lp.0 as *mut Vec<(R, R)>);
    let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
    if GetMonitorInfoW(mon, &mut mi).as_bool() {
        out.push((rect(mi.rcMonitor), rect(mi.rcWork)));
    }
    BOOL(1)
}

/// Every visible top-level window front to back (EnumWindows walks the z-order top down), ours included: (hwnd, pid, rect).
unsafe extern "system" fn collect_scene(hwnd: HWND, lp: LPARAM) -> BOOL {
    let out = &mut *(lp.0 as *mut Vec<(isize, u32, R)>);
    if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() || is_cloaked(hwnd) {
        return BOOL(1);
    }
    let mut cls = [0u16; 64];
    let n = GetClassNameW(hwnd, &mut cls) as usize;
    if matches!(String::from_utf16_lossy(&cls[..n]).as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd") {
        return BOOL(1);
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if let Some(r) = hwnd_rect(hwnd) {
        out.push((hwnd.0 as isize, pid, r));
    }
    BOOL(1)
}

/// Displays (full frame + work area) and every visible window front to back, ours included (`mine`), with the app's exe name; for the
/// scene tree (host/tree.rs). Window titles stay out (they can name clients).
pub fn scene_now() -> (Vec<Display>, Vec<SceneWin>) {
    let mut frames: Vec<(R, R)> = vec![];
    let mut raw: Vec<(isize, u32, R)> = vec![];
    unsafe {
        let _ = EnumDisplayMonitors(None, None, Some(collect_frames), LPARAM(&mut frames as *mut _ as isize));
        let _ = EnumWindows(Some(collect_scene), LPARAM(&mut raw as *mut _ as isize));
    }
    let me = unsafe { GetCurrentProcessId() };
    let mut names: std::collections::HashMap<u32, String> = Default::default();
    let displays = frames.into_iter().map(|(frame, work)| Display { frame, work }).collect();
    let wins = raw
        .into_iter()
        .filter(|(_, pid, r)| *pid == me || (r.w() > 80 && r.h() > 40))
        .map(|(h, pid, r)| {
            let app = names.entry(pid).or_insert_with(|| exe_of(pid).unwrap_or_default()).clone();
            SceneWin { id: h as i64, app, rect: r, mine: pid == me }
        })
        .collect();
    (displays, wins)
}
