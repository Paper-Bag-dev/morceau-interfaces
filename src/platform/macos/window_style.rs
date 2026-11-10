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

fn start_hit_test() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::thread::spawn(|| loop {
            std::thread::sleep(std::time::Duration::from_millis(40));
            sys::on_main(hit_test_tick);
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
            let take = wants_mouse(x, y, rects);
            let ignoring: bool = msg_send![w, ignoresMouseEvents];
            if ignoring == take {
                let _: () = msg_send![w, setIgnoresMouseEvents: !take];
            }
        }
    }
}
