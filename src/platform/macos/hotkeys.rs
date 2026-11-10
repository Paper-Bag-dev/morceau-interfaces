//! System-wide shortcuts on macOS, through the global-hotkey crate (Carbon hot keys underneath, no permission prompt):
//!   Ctrl+Option+X      dismiss every floating UI
//!   Ctrl+Option+Space  bring back anything that needed you (note: macOS uses this combination for "next input source" when more than
//!                      one keyboard layout is enabled; then registration fails and `morc summon` still works)
//! Must be called on the main thread before the event loop runs (host/mod.rs does); presses arrive on a channel read by a small thread.

use crate::{ev::{Ev, Proxy}, protocol::Request, util};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

pub fn spawn(proxy: Proxy) {
    let manager = match GlobalHotKeyManager::new() {
        Ok(m) => m,
        Err(e) => {
            util::log(&format!("hotkeys: unavailable ({e})"));
            return;
        }
    };
    let mods = Some(Modifiers::CONTROL | Modifiers::ALT);
    let dismiss = HotKey::new(mods, Code::KeyX);
    let summon = HotKey::new(mods, Code::Space);
    let a = manager.register(dismiss).is_ok();
    let b = manager.register(summon).is_ok();
    util::log(&format!("hotkeys: dismiss-all Ctrl+Option+X {}, summon Ctrl+Option+Space {}", if a { "on" } else { "unavailable" }, if b { "on" } else { "unavailable" }));
    // The registrations live as long as the manager: keep it for the host's whole life.
    std::mem::forget(manager);
    let (id_dismiss, id_summon) = (dismiss.id(), summon.id());
    std::thread::spawn(move || {
        while let Ok(e) = GlobalHotKeyEvent::receiver().recv() {
            if e.state != HotKeyState::Pressed {
                continue;
            }
            let req = if e.id == id_dismiss {
                Request::DismissAll
            } else if e.id == id_summon {
                Request::Summon
            } else {
                continue;
            };
            if proxy.send_event(Ev::Cmd(req, None)).is_err() {
                return;
            }
        }
    });
}
