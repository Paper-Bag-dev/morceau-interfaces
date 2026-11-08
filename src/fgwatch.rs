//! Event-driven foreground tracking (no polling): Windows tells us when the active window changes,
//! and surfaces that would now cover it slide out of the way.

use crate::ev::{Ev, Proxy};
use std::sync::OnceLock;
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT};

static PROXY: OnceLock<Proxy> = OnceLock::new();

unsafe extern "system" fn on_event(_h: HWINEVENTHOOK, _e: u32, _w: HWND, _o: i32, _c: i32, _t: u32, _ms: u32) {
    if let Some(p) = PROXY.get() {
        let _ = p.send_event(Ev::Foreground);
    }
}

/// Must be called on the thread that runs the message loop (the main thread, before the event loop starts).
pub fn install(proxy: Proxy) {
    let _ = PROXY.set(proxy);
    unsafe {
        let _ = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, Some(HMODULE(std::ptr::null_mut())), Some(on_event), 0, 0, WINEVENT_OUTOFCONTEXT);
    }
}
