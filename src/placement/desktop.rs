//! What the desktop looks like right now: the visible windows, the monitors, the active window and whether it is fullscreen, its app and
//! title. The OS calls are in platform/<os>/desktop.rs; this file adds the snapshot and the placement helpers on top.

use super::R;
pub use crate::platform::os::desktop::{fg_exe, fg_title, foreground_now, scene_now, windows_now, work_at};

/// A display for the scene tree: its full frame and its work area (physical px).
pub struct Display {
    pub frame: R,
    pub work: R,
}

/// An on-screen window for the scene tree, front to back. `mine` = one of our cards.
pub struct SceneWin {
    pub id: i64,
    pub app: String,
    pub rect: R,
    pub mine: bool,
}

#[derive(Clone)]
pub struct Foreground {
    pub rect: Option<R>,
    pub work: R,
    pub fullscreen: bool,
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

/// Nudge a rect so it sits fully inside the work area of the monitor it mostly lives on. Returns the new top-left.
pub fn clamp_inside(r: &R) -> (i32, i32) {
    let work = work_at((r.l + r.r) / 2, (r.t + r.b) / 2);
    let m = 8;
    let x = r.l.min(work.r - r.w() - m).max(work.l + m);
    let y = r.t.min(work.b - r.h() - m).max(work.t + m);
    (x, y)
}

/// Does this rect sit on top of the active window right now?
pub fn covers_foreground(r: &R) -> bool {
    foreground().rect.map(|f| r.overlap(&f) > 0).unwrap_or(false)
}
