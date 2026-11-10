//! System-wide shortcuts, registered on their own thread with its own message loop (no extra dependency):
//!   Ctrl+Alt+X      dismiss every floating UI (the quick "get rid of all of it")
//!   Ctrl+Alt+Space  bring back anything that needed you (parked urgent asks), next to the active window
//! If another program owns a combination, registration quietly fails and the CLI equivalents still work
//! (`morc dismiss-all`, `morc summon`).

use crate::{ev::{Ev, Proxy}, protocol::Request, util};
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

const ID_DISMISS: i32 = 1;
const ID_SUMMON: i32 = 2;

pub fn spawn(proxy: Proxy) {
    std::thread::spawn(move || unsafe {
        let mods = HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0 | MOD_NOREPEAT.0);
        let a = RegisterHotKey(None, ID_DISMISS, mods, 'X' as u32).is_ok();
        let b = RegisterHotKey(None, ID_SUMMON, mods, 0x20).is_ok(); // VK_SPACE
        util::log(&format!("hotkeys: dismiss-all Ctrl+Alt+X {}, summon Ctrl+Alt+Space {}", if a { "on" } else { "unavailable" }, if b { "on" } else { "unavailable" }));
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY {
                let req = match msg.wParam.0 as i32 {
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
