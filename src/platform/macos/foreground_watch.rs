//! Notice when the active window changes. Windows pushes an event; macOS has no public "frontmost window changed" notification that also
//! covers switching windows inside one app, so a small thread polls the window server (thread-safe, ~1 ms) and sends the same
//! `Ev::Foreground` the Windows hook sends: every 400 ms while a card is up, every 2 s otherwise.

use super::sys;
use crate::ev::{Ev, Proxy};
use std::time::Duration;

pub fn install(proxy: Proxy) {
    std::thread::spawn(move || {
        let me = std::process::id() as i32;
        let mut last: Option<(i32, i64)> = None;
        loop {
            // quick while a card is up (it may have to move out of the way), slow otherwise: the next card's placement looks again anyway
            std::thread::sleep(Duration::from_millis(if super::window_style::cards_up() { 400 } else { 2000 }));
            // front-most normal window (layer 0) that is not one of our cards
            let now = sys::window_list().into_iter().find(|w| w.layer == 0 && w.pid != me && w.alpha > 0.0).map(|w| (w.pid, w.number));
            if now != last {
                last = now;
                if proxy.send_event(Ev::Foreground).is_err() {
                    return;
                }
            }
        }
    });
}
