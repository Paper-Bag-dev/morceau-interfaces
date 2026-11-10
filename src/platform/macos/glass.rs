//! Native glass behind each card's plate. CSS `backdrop-filter` in a transparent WKWebView only blurs the page itself, never the apps
//! behind the window, so on macOS the blur comes from AppKit: the page reports each plate's rounded rect (layout `glass`), and each one
//! gets an NSGlassEffectView (macOS 26 Liquid Glass) or, on older systems, an NSVisualEffectView, placed in the card's own window UNDER
//! the web view (wry adds the WKWebView as a subview of the content view). The page then draws a translucent tint, grain and sheen on top.
//! Called on the main thread from host/layout.rs (tao's event loop); anything else is ignored.

use super::sys::{CGPoint, CGRect, CGSize, Id};
use objc2::runtime::AnyClass;
use objc2::{class, msg_send};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn views() -> &'static Mutex<HashMap<isize, Vec<isize>>> {
    static V: OnceLock<Mutex<HashMap<isize, Vec<isize>>>> = OnceLock::new();
    V.get_or_init(Default::default)
}

/// rects: physical px relative to the window's top-left (left, top, right, bottom, radius), one per plate. Empty = no glass.
pub fn set_glass(win: isize, rects: &[(i32, i32, i32, i32, i32)]) {
    let w = win as Id;
    if w.is_null() {
        return;
    }
    unsafe {
        let main: bool = msg_send![class!(NSThread), isMainThread];
        if !main {
            return;
        }
        let content: Id = msg_send![w, contentView];
        if content.is_null() {
            return;
        }
        let s: f64 = msg_send![w, backingScaleFactor];
        let s = if s > 0.0 { s } else { 1.0 };
        let bounds: CGRect = msg_send![content, bounds];
        let flipped: bool = msg_send![content, isFlipped];
        prune_closed();
        let Ok(mut map) = views().lock() else { return };
        let list = map.entry(win).or_default();
        while list.len() > rects.len() {
            let v = list.pop().unwrap_or(0) as Id;
            if !v.is_null() {
                let _: () = msg_send![v, removeFromSuperview];
                let _: () = msg_send![v, release];
            }
        }
        for (i, &(l, t, r, b, rad)) in rects.iter().enumerate() {
            let (x, wd, ht) = (l as f64 / s, (r - l).max(0) as f64 / s, (b - t).max(0) as f64 / s);
            let y = if flipped { t as f64 / s } else { bounds.size.height - b as f64 / s };
            let frame = CGRect { origin: CGPoint { x, y }, size: CGSize { width: wd, height: ht } };
            let radius = rad.max(0) as f64 / s;
            if i >= list.len() {
                let v = make(frame);
                if v.is_null() {
                    return;
                }
                // NSWindowBelow (-1) relative to nil: under every other subview, so under the web view
                let _: () = msg_send![content, addSubview: v, positioned: -1isize, relativeTo: std::ptr::null_mut::<objc2::runtime::AnyObject>()];
                list.push(v as isize);
            }
            let v = list[i] as Id;
            let _: () = msg_send![v, setFrame: frame];
            round(v, radius);
        }
    }
}

/// Drop our handles for windows the app no longer owns (a closed card), so their glass views are freed with them.
unsafe fn prune_closed() {
    let app: Id = msg_send![class!(NSApplication), sharedApplication];
    let list: Id = msg_send![app, windows];
    let n: usize = if list.is_null() { 0 } else { msg_send![list, count] };
    let live: Vec<isize> = (0..n).map(|i| { let w: Id = msg_send![list, objectAtIndex: i]; w as isize }).collect();
    let dead: Vec<isize> = views().lock().map(|m| m.keys().filter(|k| !live.contains(k)).copied().collect()).unwrap_or_default();
    for w in dead {
        forget(w);
    }
}

/// Forget a closed window's views (they go with the window; this only drops our handles).
pub fn forget(win: isize) {
    if let Ok(mut m) = views().lock() {
        if let Some(list) = m.remove(&win) {
            for v in list {
                let v = v as Id;
                if !v.is_null() {
                    unsafe {
                        let _: () = msg_send![v, release];
                    }
                }
            }
        }
    }
}

unsafe fn make(frame: CGRect) -> Id {
    let v = make_view(frame);
    if !v.is_null() {
        // always the dark material: the cards are dark glass whatever the system appearance is (user, 10 Nov: "mostly dark")
        let name: Id = msg_send![class!(NSString), stringWithUTF8String: c"NSAppearanceNameDarkAqua".as_ptr()];
        let dark: Id = msg_send![class!(NSAppearance), appearanceNamed: name];
        if !dark.is_null() {
            let _: () = msg_send![v, setAppearance: dark];
        }
    }
    v
}

unsafe fn make_view(frame: CGRect) -> Id {
    if let Some(cls) = AnyClass::get(c"NSGlassEffectView") {
        let v: Id = msg_send![cls, alloc];
        let v: Id = msg_send![v, initWithFrame: frame];
        if !v.is_null() {
            const CLEAR: isize = 1; // NSGlassEffectViewStyleClear: lets more of what is behind through than Regular (user, 10 Nov)
            let _: () = msg_send![v, setStyle: CLEAR];
        }
        return v;
    }
    let v: Id = msg_send![class!(NSVisualEffectView), alloc];
    let v: Id = msg_send![v, initWithFrame: frame];
    if v.is_null() {
        return v;
    }
    const HUD_WINDOW: isize = 13; // NSVisualEffectMaterialHUDWindow
    const BEHIND_WINDOW: isize = 0;
    const ACTIVE: isize = 1;
    let _: () = msg_send![v, setMaterial: HUD_WINDOW];
    let _: () = msg_send![v, setBlendingMode: BEHIND_WINDOW];
    let _: () = msg_send![v, setState: ACTIVE];
    let _: () = msg_send![v, setWantsLayer: true];
    v
}

unsafe fn round(v: Id, radius: f64) {
    let glass: bool = match AnyClass::get(c"NSGlassEffectView") {
        Some(cls) => msg_send![v, isKindOfClass: cls],
        None => false,
    };
    if glass {
        let _: () = msg_send![v, setCornerRadius: radius];
    } else {
        let layer: Id = msg_send![v, layer];
        if !layer.is_null() {
            let _: () = msg_send![layer, setCornerRadius: radius];
            let _: () = msg_send![layer, setMasksToBounds: true];
        }
    }
}
