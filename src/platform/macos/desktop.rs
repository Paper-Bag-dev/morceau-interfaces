//! What the desktop looks like right now on macOS: the visible windows, the displays, the active window and whether it is fullscreen, its
//! app and title. From the window server (CGWindowList) and AppKit (NSScreen, NSWorkspace). No permission is needed except for window
//! titles (`fg_title`), which macOS only shows to apps with Screen Recording; without it the title is None.
//! placement/desktop.rs re-exports these; its snapshot wrappers (`foreground()`, `windows()`) call the `_now` ones.

use crate::placement::{Foreground, R};
use super::sys::{self, CGRect};

/// Points (top-left origin) to physical pixels.
fn px(r: &CGRect, s: f64) -> R {
    R {
        l: (r.origin.x * s).round() as i32,
        t: (r.origin.y * s).round() as i32,
        r: ((r.origin.x + r.size.width) * s).round() as i32,
        b: ((r.origin.y + r.size.height) * s).round() as i32,
    }
}

fn contains(r: &R, x: i32, y: i32) -> bool {
    x >= r.l && x < r.r && y >= r.t && y < r.b
}

const FALLBACK: R = R { l: 0, t: 0, r: 1920, b: 1080 };

/// The active window (the front-most normal window of the front-most app) and the work area of its display. Uncached.
pub fn foreground_now() -> Foreground {
    let s = sys::scale();
    let screens = sys::screens();
    let me = std::process::id() as i32;
    // Our own card can be "active" while you type in a field; Finder with no window open is the desktop. Neither is something to avoid.
    let fg_pid = sys::frontmost_pid().filter(|p| *p != me);
    let wins = sys::window_list();
    // When WE are frontmost (you clicked or hovered a card, or typed in one) the active window is still the one you were working in: the
    // front-most normal window of another app (the list is front to back), as foreground_watch does. Without this, placement saw no active
    // window at all and every card fell back to scored placement, ignoring the tiers around the active window (user, 11-11).
    let fg_rect = fg_pid
        .and_then(|p| wins.iter().find(|w| w.pid == p && w.layer == 0 && w.alpha > 0.0))
        .or_else(|| wins.iter().find(|w| w.pid != me && w.layer == 0 && w.alpha > 0.0))
        .map(|w| px(&w.bounds, s));
    let mon = match fg_rect {
        Some(r) => screens.iter().max_by_key(|m| px(&m.frame, s).overlap(&r)),
        None => screens.first(),
    };
    let work = mon.map(|m| px(&m.visible, s)).unwrap_or(FALLBACK);
    // Fullscreen = the active window covers its entire display (menu bar included): a full-screen app, a game or a video.
    let fullscreen = match (fg_rect, mon) {
        (Some(r), Some(m)) => {
            let m = px(&m.frame, s);
            r.l <= m.l && r.t <= m.t && r.r >= m.r && r.b >= m.b && r.area() > 0
        }
        _ => false,
    };
    Foreground { rect: fg_rect, work, fullscreen }
}

/// Every visible normal window except ours (physical px). Menu bar, Dock and other system layers are left out by `layer == 0`. Uncached.
pub fn windows_now() -> Vec<R> {
    let s = sys::scale();
    let me = std::process::id() as i32;
    sys::window_list()
        .iter()
        .filter(|w| w.layer == 0 && w.pid != me && w.alpha > 0.0)
        .map(|w| px(&w.bounds, s))
        .filter(|r| r.w() > 80 && r.h() > 40)
        .collect()
}

/// Work area of the display that contains this point (primary if none).
pub fn work_at(x: i32, y: i32) -> R {
    let s = sys::scale();
    let screens = sys::screens();
    screens
        .iter()
        .find(|m| contains(&px(&m.frame, s), x, y))
        .or(screens.first())
        .map(|m| px(&m.visible, s))
        .unwrap_or(FALLBACK)
}

/// Executable name (lowercase) of the active app, e.g. "google chrome".
pub fn fg_exe() -> Option<String> {
    sys::frontmost_exe()
}

/// Title of the active window, if macOS lets us see it (Screen Recording permission).
pub fn fg_title() -> Option<String> {
    let pid = sys::frontmost_pid()?;
    sys::window_list().into_iter().find(|w| w.pid == pid && w.layer == 0).and_then(|w| w.name)
}
