//! Window styling on macOS: float above normal windows on every Space, never show in the Dock or Cmd-Tab (the Accessory activation policy,
//! set in host/mod.rs), and click-through outside the card's content.
//!
//! macOS has no window regions (Windows' SetWindowRgn). Instead `set_region` remembers each window's content rects, and a small timer on the
//! main thread turns `ignoresMouseEvents` on while the pointer is over the transparent padding and off while it is over the content, so
//! clicks on the empty corners still reach the window behind. `win` is the NSWindow pointer (Surface::hwnd() on macOS).
//!
//! Hidden = on screen at alpha 0, ignoring the mouse, never ordered out. WKWebView pauses requestAnimationFrame and CSS animations in a window
//! that is not on screen, and the page measures itself from requestAnimationFrame (ui/core/bridge.js), so an ordered-out card never reports its
//! size and never appears. WebView2 keeps hidden pages running, which is why Windows can really hide.

use super::sys::{self, CGPoint, CGRect, Id};
use objc2::{class, msg_send};
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, Once, OnceLock};

type Rects = Vec<(i32, i32, i32, i32, i32)>;

/// Where hover changes go (set once at startup by watch_hover) and which cards the pointer is on now.
fn hover_proxy() -> &'static Mutex<Option<crate::ev::Proxy>> {
    static P: OnceLock<Mutex<Option<crate::ev::Proxy>>> = OnceLock::new();
    P.get_or_init(Default::default)
}
fn hovered() -> &'static Mutex<HashMap<isize, bool>> {
    static H: OnceLock<Mutex<HashMap<isize, bool>>> = OnceLock::new();
    H.get_or_init(Default::default)
}

/// WKWebView only tracks the pointer in the active app's key window, so a floating card never gets pointerenter until it is clicked.
/// The hit test below already knows when the pointer is over a card's content; it sends `Ev::Hover` on every change.
pub fn watch_hover(proxy: crate::ev::Proxy) {
    if let Ok(mut p) = hover_proxy().lock() {
        *p = Some(proxy);
    }
}

/// The last pointer spot sent per card (points), so a still pointer sends nothing.
fn last_pointer() -> &'static Mutex<HashMap<isize, (i32, i32)>> {
    static L: OnceLock<Mutex<HashMap<isize, (i32, i32)>>> = OnceLock::new();
    L.get_or_init(Default::default)
}

/// Cards sent down the window stack (set_depth > 0): the hit test checks whether something covers them at the pointer.
fn lowered() -> &'static Mutex<std::collections::HashSet<isize>> {
    static L: OnceLock<Mutex<std::collections::HashSet<isize>>> = OnceLock::new();
    L.get_or_init(Default::default)
}

/// Is another app's window in front of this card at this point (points, bottom-left origin, as NSEvent gives it)?
fn covered_at(w: Id, mouse: CGPoint) -> bool {
    let mine: isize = unsafe { msg_send![w, windowNumber] };
    let primary_h = super::sys::screens().first().map(|s| s.frame.size.height).unwrap_or(0.0);
    let (x, y) = (mouse.x, primary_h - mouse.y);   // window list bounds: points, top-left origin
    let me = std::process::id() as i32;
    for win in super::sys::window_list() {
        if win.number as isize == mine {
            return false;   // reached the card first: nothing above it here
        }
        let b = win.bounds;
        if win.pid != me && win.layer == 0 && win.alpha > 0.0 && x >= b.origin.x && x < b.origin.x + b.size.width && y >= b.origin.y && y < b.origin.y + b.size.height {
            return true;
        }
    }
    false
}

fn shapes() -> &'static Mutex<HashMap<isize, Rects>> {
    static S: OnceLock<Mutex<HashMap<isize, Rects>>> = OnceLock::new();
    S.get_or_init(Default::default)
}

/// Float level, all Spaces, out of the window cycle, allowed beside full-screen apps, no system shadow (the page draws its own edge).
pub fn overlay_styles(win: isize) {
    let w = win as Id;
    if w.is_null() {
        return;
    }
    const CAN_JOIN_ALL_SPACES: usize = 1 << 0;
    const STATIONARY: usize = 1 << 4;
    const IGNORES_CYCLE: usize = 1 << 6;
    const FULL_SCREEN_AUXILIARY: usize = 1 << 8;
    const FLOATING_LEVEL: isize = 3;
    unsafe {
        let _: () = msg_send![w, setCollectionBehavior: CAN_JOIN_ALL_SPACES | STATIONARY | IGNORES_CYCLE | FULL_SCREEN_AUXILIARY];
        let _: () = msg_send![w, setLevel: FLOATING_LEVEL];
        let _: () = msg_send![w, setHasShadow: false];
        let _: () = msg_send![w, setHidesOnDeactivate: false];
    }
    hide(win);
    start_hit_test();
}

/// Stack depth: 0 = on the floating level above every window; n = on the normal level, just under the n-th window that overlaps the card
/// (front to back; `rect` = the card's visible plate, physical px). Only overlapping windows count, so every step is a visible change;
/// with none overlapping, the front-most app window is level 1. Returns the depth applied (clamped to how low it can go).
pub fn set_depth(win: isize, depth: usize, rect: crate::placement::R) -> usize {
    let w = win as Id;
    if w.is_null() {
        return 0;
    }
    const NORMAL_LEVEL: isize = 0;
    const FLOATING_LEVEL: isize = 3;
    const BELOW: isize = -1;
    let me = std::process::id() as i32;
    let k = super::sys::scale();
    let apps: Vec<(isize, crate::placement::R)> = super::sys::window_list()
        .into_iter()
        .filter(|x| x.layer == 0 && x.pid != me && x.alpha > 0.0)
        .map(|x| {
            let b = x.bounds;
            (x.number as isize, crate::placement::R { l: (b.origin.x * k) as i32, t: (b.origin.y * k) as i32, r: ((b.origin.x + b.size.width) * k) as i32, b: ((b.origin.y + b.size.height) * k) as i32 })
        })
        .collect();
    let mut levels: Vec<isize> = apps.iter().filter(|(_, r)| r.overlap(&rect) > 0).map(|(n, _)| *n).collect();
    if levels.is_empty() {
        levels = apps.first().map(|(n, _)| vec![*n]).unwrap_or_default();
    }
    let d = depth.min(levels.len());
    if let Ok(mut l) = lowered().lock() {
        if d == 0 { l.remove(&win); } else { l.insert(win); }
    }
    unsafe {
        if d == 0 {
            let _: () = msg_send![w, setLevel: FLOATING_LEVEL];
            let _: () = msg_send![w, orderFrontRegardless];
        } else {
            let _: () = msg_send![w, setLevel: NORMAL_LEVEL];
            let _: () = msg_send![w, orderWindow: BELOW, relativeTo: levels[d - 1]];
        }
    }
    d
}

/// Windows swaps the system cursor while a card is carried. macOS has no public system-wide cursor swap, and the native drag loop keeps the
/// arrow, so these are no-ops.
pub fn drag_cursor_on() {}
pub fn drag_cursor_off() {}

/// Text inputs need real keyboard focus: activate the app and make the card key. Otherwise do nothing (cards are shown with
/// orderFrontRegardless, which never activates).
pub fn set_noactivate(win: isize, on: bool) {
    let w = win as Id;
    if w.is_null() || on {
        return;
    }
    unsafe {
        let app: Id = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, activateIgnoringOtherApps: true];
        let _: () = msg_send![w, makeKeyWindow];
    }
}

/// Show without activating the app, above normal windows.
pub fn show_topmost_noactivate(win: isize) {
    let w = win as Id;
    if w.is_null() {
        return;
    }
    let shaped = shapes().lock().map(|m| m.contains_key(&win)).unwrap_or(false);
    unsafe {
        let _: () = msg_send![w, setAlphaValue: 1.0f64];
        if !shaped {
            let _: () = msg_send![w, setIgnoresMouseEvents: false];
        }
        let _: () = msg_send![w, orderFrontRegardless];
    }
}

pub fn hide(win: isize) {
    let w = win as Id;
    if w.is_null() {
        return;
    }
    if let Ok(mut m) = shapes().lock() {
        m.remove(&win);
    }
    unsafe {
        let _: () = msg_send![w, setAlphaValue: 0.0f64];
        let _: () = msg_send![w, setIgnoresMouseEvents: true];
        let _: () = msg_send![w, orderFrontRegardless];
    }
}

/// Remember the content rects (physical px relative to the window's top-left: left, top, right, bottom, radius). An empty list means the
/// whole window takes the mouse, as on Windows.
pub fn set_region(win: isize, rects: &[(i32, i32, i32, i32, i32)]) {
    if let Ok(mut m) = shapes().lock() {
        if rects.is_empty() {
            m.remove(&win);
        } else {
            m.insert(win, rects.to_vec());
        }
    }
    if rects.is_empty() {
        let w = win as Id;
        let alpha: f64 = if w.is_null() { 0.0 } else { unsafe { msg_send![w, alphaValue] } };
        if !w.is_null() && alpha > 0.0 {
            unsafe {
                let _: () = msg_send![w, setIgnoresMouseEvents: false];
            }
        }
    }
}

/// Native glass under each plate (rects like set_region's, one per plate): see glass.rs.
pub fn set_glass(win: isize, rects: &[(i32, i32, i32, i32, i32)]) {
    super::glass::set_glass(win, rects);
}

/// Should the window take the mouse at this point (physical px relative to its top-left), given its content rects?
fn wants_mouse(x: f64, y: f64, rects: &[(i32, i32, i32, i32, i32)]) -> bool {
    // Inside any piece, with rounded corners: a point in a corner square must also be within `radius` of that corner's circle centre.
    // One pixel of slack keeps the pointer from flickering between the card and the app behind on an edge pixel.
    const SLACK: f64 = 1.0;
    rects.iter().any(|&(l, t, r, b, rad)| {
        let (l, t, r, b, rad) = (l as f64 - SLACK, t as f64 - SLACK, r as f64 + SLACK, b as f64 + SLACK, rad.max(0) as f64);
        if x < l || x >= r || y < t || y >= b {
            return false;
        }
        let cx = x.clamp(l + rad, r - rad);
        let cy = y.clamp(t + rad, b - rad);
        (x - cx).powi(2) + (y - cy).powi(2) <= rad * rad
    })
}

/// Is any card on screen with a content shape? (cheap; the timers below idle when not)
pub fn cards_up() -> bool {
    shapes().lock().map(|m| !m.is_empty()).unwrap_or(false)
}

fn start_hit_test() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::thread::spawn(|| loop {
            std::thread::sleep(std::time::Duration::from_millis(40));
            // nothing on screen: nothing to flip, so do not wake the main thread 25 times a second
            if cards_up() {
                sys::on_main(hit_test_tick);
            }
        });
    });
}

/// Main thread, ~25 times a second: flip each card's click-through to match what is under the pointer.
extern "C" fn hit_test_tick(_: *mut c_void) {
    let Ok(mut map) = shapes().lock() else { return };
    if map.is_empty() {
        return;
    }
    unsafe {
        // Only touch windows the app still owns: a card that was closed without `hide` must not be messaged after it is freed.
        let app: Id = msg_send![class!(NSApplication), sharedApplication];
        let list: Id = msg_send![app, windows];
        let n: usize = if list.is_null() { 0 } else { msg_send![list, count] };
        let mut live: Vec<isize> = Vec::with_capacity(n);
        for i in 0..n {
            let w: Id = msg_send![list, objectAtIndex: i];
            live.push(w as isize);
        }
        map.retain(|k, _| live.contains(k));
        let mouse: CGPoint = msg_send![class!(NSEvent), mouseLocation];
        for (win, rects) in map.iter() {
            let w = *win as Id;
            let visible: bool = msg_send![w, isVisible];
            let alpha: f64 = msg_send![w, alphaValue];
            if !visible || alpha <= 0.0 {
                continue;
            }
            let f: CGRect = msg_send![w, frame];
            let s: f64 = msg_send![w, backingScaleFactor];
            let x = (mouse.x - f.origin.x) * s;
            let y = (f.origin.y + f.size.height - mouse.y) * s;
            // on the card's shape AND, for a card sent down the stack, no other app's window in front of it there (it must not react
            // through the window covering it). Cards on top skip the check: nothing can cover them. (Not windowNumberAtPoint: a card that
            // ignores the mouse is skipped by it, so the card would never take the mouse back.)
            let take = wants_mouse(x, y, rects) && !(lowered().lock().map(|l| l.contains(win)).unwrap_or(false) && covered_at(w, mouse));
            // tell the page when the pointer goes onto or off the card (it cannot see that itself in an inactive app)
            let was = hovered().lock().ok().and_then(|h| h.get(win).copied()).unwrap_or(false);
            if was != take {
                if let Ok(mut h) = hovered().lock() {
                    h.insert(*win, take);
                }
                if let Some(p) = hover_proxy().lock().ok().and_then(|p| p.clone()) {
                    let _ = p.send_event(crate::ev::Ev::Hover(*win, take));
                }
            }
            // while on the card, where (points from the window's top-left): the page marks what is under it as hovered
            if take {
                let (px, py) = ((mouse.x - f.origin.x).round() as i32, (f.origin.y + f.size.height - mouse.y).round() as i32);
                let moved = last_pointer().lock().ok().map(|mut l| l.insert(*win, (px, py)) != Some((px, py))).unwrap_or(false);
                if moved {
                    if let Some(p) = hover_proxy().lock().ok().and_then(|p| p.clone()) {
                        let _ = p.send_event(crate::ev::Ev::Pointer(*win, px as f64, py as f64));
                    }
                }
            } else if let Ok(mut l) = last_pointer().lock() {
                l.remove(win);
            }
            let ignoring: bool = msg_send![w, ignoresMouseEvents];
            if ignoring == take {
                let _: () = msg_send![w, setIgnoresMouseEvents: !take];
            }
        }
    }
}
