//! System-wide shortcuts, registered on their own thread with its own message loop (no extra dependency):
//!   Ctrl+Alt+X      dismiss every floating UI (the quick "get rid of all of it")
//!   Ctrl+Alt+Space  bring back anything that needed you (parked urgent asks), next to the active window
//!   Ctrl+Alt+Up     every card to the top of the window stack; Ctrl+Alt+Down: every card to the bottom
//!   Up / Down       ONLY while the pointer is on a card (`arrows(true)` on hover): step the card up / down the window stack. The host
//!                 asks this thread (its message loop owns the registrations) with a thread message; released when the pointer leaves.
//! If another program owns a combination, registration quietly fails and the CLI equivalents still work
//! (`morc dismiss-all`, `morc summon`).

use crate::{ev::{Ev, Proxy}, protocol::Request, util};
use std::sync::atomic::{AtomicU32, Ordering};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, PostThreadMessageW, MSG, WM_APP, WM_HOTKEY};

const ID_DISMISS: i32 = 1;
const ID_SUMMON: i32 = 2;
const ID_UP: i32 = 3;
const ID_DOWN: i32 = 4;
const ID_ALL_UP: i32 = 5;
const ID_ALL_DOWN: i32 = 6;
const WM_ARROWS: u32 = WM_APP + 1;
static THREAD: AtomicU32 = AtomicU32::new(0);

/// Up / Down as global keys while the pointer is on a card: asks the hotkey thread (registrations belong to its message loop).
pub fn arrows(on: bool) {
    let t = THREAD.load(Ordering::Relaxed);
    if t != 0 {
        unsafe {
            let _ = PostThreadMessageW(t, WM_ARROWS, WPARAM(on as usize), LPARAM(0));
        }
    }
}

pub fn spawn(proxy: Proxy) {
    std::thread::spawn(move || unsafe {
        THREAD.store(GetCurrentThreadId(), Ordering::Relaxed);
        let mods = HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0 | MOD_NOREPEAT.0);
        let a = RegisterHotKey(None, ID_DISMISS, mods, 'X' as u32).is_ok();
        let b = RegisterHotKey(None, ID_SUMMON, mods, 0x20).is_ok(); // VK_SPACE
        let c = RegisterHotKey(None, ID_ALL_UP, mods, 0x26).is_ok() && RegisterHotKey(None, ID_ALL_DOWN, mods, 0x28).is_ok(); // VK_UP / VK_DOWN
        util::log(&format!("hotkeys: stack all Ctrl+Alt+Up/Down {}", if c { "on" } else { "unavailable" }));
        util::log(&format!("hotkeys: dismiss-all Ctrl+Alt+X {}, summon Ctrl+Alt+Space {}", if a { "on" } else { "unavailable" }, if b { "on" } else { "unavailable" }));
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_ARROWS {
                // no MOD_NOREPEAT: holding an arrow keeps stepping
                if msg.wParam.0 != 0 {
                    let _ = RegisterHotKey(None, ID_UP, HOT_KEY_MODIFIERS(0), 0x26);   // VK_UP
                    let _ = RegisterHotKey(None, ID_DOWN, HOT_KEY_MODIFIERS(0), 0x28); // VK_DOWN
                } else {
                    let _ = UnregisterHotKey(None, ID_UP);
                    let _ = UnregisterHotKey(None, ID_DOWN);
                }
                continue;
            }
            if msg.message == WM_HOTKEY {
                let id = msg.wParam.0 as i32;
                if id == ID_ALL_UP || id == ID_ALL_DOWN {
                    if proxy.send_event(Ev::StackAll(if id == ID_ALL_UP { -1 } else { 1 })).is_err() {
                        return;
                    }
                    continue;
                }
                if id == ID_UP || id == ID_DOWN {
                    if proxy.send_event(Ev::Step(if id == ID_UP { -1 } else { 1 })).is_err() {
                        return;
                    }
                    continue;
                }
                let req = match id {
                    ID_DISMISS => Request::DismissAll,
                    ID_SUMMON => Request::Summon,
                    _ => continue,
                };
                if proxy.send_event(Ev::Cmd(req, None)).is_err() {
                    return;
                }
            }
        }
    });
}
