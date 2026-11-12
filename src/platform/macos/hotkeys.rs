//! System-wide shortcuts on macOS, through the global-hotkey crate (Carbon hot keys underneath, no permission prompt):
//!   Ctrl+Option+X      dismiss every floating UI
//!   Ctrl+Option+Space  bring back anything that needed you (note: macOS uses this combination for "next input source" when more than
//!                      one keyboard layout is enabled; then registration fails and `morc summon` still works)
//!   Ctrl+Option+Up     every card to the top of the window stack; Ctrl+Option+Down: every card to the bottom
//!   Up / Down          ONLY while the pointer is on a card (`arrows(true)` on hover, `arrows(false)` when it leaves): step the card up /
//!                      down the window stack (host/cards.rs). Registered and released with the hover, so arrows reach your apps otherwise.
//! Must be called on the main thread before the event loop runs (host/mod.rs does); presses arrive on a channel read by a small thread.

use crate::{ev::{Ev, Proxy}, protocol::Request, util};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::cell::RefCell;

thread_local! {
    // the manager lives on the main thread (Carbon): kept here so the arrows can be registered and released later
    static MANAGER: RefCell<Option<GlobalHotKeyManager>> = const { RefCell::new(None) };
}
fn arrow(code: Code) -> HotKey {
    HotKey::new(None, code)
}

/// Up / Down as global keys while the pointer is on a card (main thread). Idempotent.
pub fn arrows(on: bool) {
    MANAGER.with(|m| {
        if let Some(m) = m.borrow().as_ref() {
            let keys = [arrow(Code::ArrowUp), arrow(Code::ArrowDown)];
            let _ = if on { m.register_all(&keys) } else { m.unregister_all(&keys) };
        }
    });
}

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
    let (all_up, all_down) = (HotKey::new(mods, Code::ArrowUp), HotKey::new(mods, Code::ArrowDown));
    let c = manager.register_all(&[all_up, all_down]).is_ok();
    util::log(&format!("hotkeys: stack all Ctrl+Option+Up/Down {}", if c { "on" } else { "unavailable" }));
    let (id_all_up, id_all_down) = (all_up.id(), all_down.id());
    util::log(&format!("hotkeys: dismiss-all Ctrl+Option+X {}, summon Ctrl+Option+Space {}", if a { "on" } else { "unavailable" }, if b { "on" } else { "unavailable" }));
    // The registrations live as long as the manager: keep it for the host's whole life (and for the arrows).
    MANAGER.with(|m| *m.borrow_mut() = Some(manager));
    let (id_dismiss, id_summon) = (dismiss.id(), summon.id());
    let (id_up, id_down) = (arrow(Code::ArrowUp).id(), arrow(Code::ArrowDown).id());
    std::thread::spawn(move || {
        while let Ok(e) = GlobalHotKeyEvent::receiver().recv() {
            if e.state != HotKeyState::Pressed {
                continue;
            }
            if e.id == id_all_up || e.id == id_all_down {
                if proxy.send_event(Ev::StackAll(if e.id == id_all_up { -1 } else { 1 })).is_err() {
                    return;
                }
                continue;
            }
            if e.id == id_up || e.id == id_down {
                if proxy.send_event(Ev::Step(if e.id == id_up { -1 } else { 1 })).is_err() {
                    return;
                }
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
