//! The raw macOS calls the port needs, in one place: AppKit through objc2 messages, CoreGraphics / CoreFoundation / libdispatch through
//! small extern blocks. Everything else in the port goes through these helpers.
//!
//! Coordinates: AppKit uses points with the origin at the BOTTOM-left of the primary screen; CoreGraphics window bounds use points with the
//! origin at the TOP-left. The host works in physical pixels, top-left (like tao's outer_position), so `screens()` flips AppKit rects to
//! top-left here and callers multiply by `scale()`.

use objc2::encode::{Encode, Encoding};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};
use std::ffi::{c_char, c_void, CStr};

pub type Id = *mut AnyObject;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CGSize {
    pub width: f64,
    pub height: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}
unsafe impl Encode for CGPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}
unsafe impl Encode for CGSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}
unsafe impl Encode for CGRect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[CGPoint::ENCODING, CGSize::ENCODING]);
}

#[repr(C)]
pub struct DispatchQueue {
    _private: [u8; 0],
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> *const c_void;
    fn CGRectMakeWithDictionaryRepresentation(dict: *const c_void, rect: *mut CGRect) -> bool;
    fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
    fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    static kCGWindowBounds: *const c_void;
    static kCGWindowLayer: *const c_void;
    static kCGWindowOwnerPID: *const c_void;
    static kCGWindowNumber: *const c_void;
    static kCGWindowName: *const c_void;
    static kCGWindowAlpha: *const c_void;
    static kCGWindowOwnerName: *const c_void;
}
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn NSBeep();
}
// libdispatch is part of libSystem, linked by default.
extern "C" {
    static _dispatch_main_q: DispatchQueue;
    fn dispatch_async_f(queue: *const DispatchQueue, context: *mut c_void, work: extern "C" fn(*mut c_void));
}

/// Run `work` on the main thread (AppKit window calls must happen there). Fire and forget.
pub fn on_main(work: extern "C" fn(*mut c_void)) {
    unsafe { dispatch_async_f(&_dispatch_main_q, std::ptr::null_mut(), work) }
}

/// One on-screen window as the window server sees it (bounds in points, top-left origin).
pub struct WinInfo {
    pub pid: i32,
    pub number: i64,
    pub layer: i64,
    pub alpha: f64,
    pub bounds: CGRect,
    /// Only filled when this process has the Screen Recording permission; None otherwise.
    pub name: Option<String>,
    /// The owning app's name (always available, no permission needed).
    pub owner: Option<String>,
}

unsafe fn dict_get(d: Id, key: *const c_void) -> Id {
    msg_send![d, objectForKey: key as Id]
}

unsafe fn num(d: Id, key: *const c_void) -> Option<f64> {
    let v = dict_get(d, key);
    if v.is_null() {
        None
    } else {
        Some(msg_send![v, doubleValue])
    }
}

/// An NSString as a Rust String.
pub unsafe fn string(s: Id) -> Option<String> {
    if s.is_null() {
        return None;
    }
    let p: *const c_char = msg_send![s, UTF8String];
    (!p.is_null()).then(|| CStr::from_ptr(p).to_string_lossy().into_owned())
}

/// On-screen windows, front to back, without the desktop elements. Safe to call from any thread.
pub fn window_list() -> Vec<WinInfo> {
    const ON_SCREEN_ONLY: u32 = 1 << 0;
    const EXCLUDE_DESKTOP: u32 = 1 << 4;
    objc2::rc::autoreleasepool(|_| unsafe {
        let arr = CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP, 0);
        if arr.is_null() {
            return vec![];
        }
        let a = arr as Id;
        let n: usize = msg_send![a, count];
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let d: Id = msg_send![a, objectAtIndex: i];
            let mut bounds = CGRect::default();
            let b = dict_get(d, kCGWindowBounds);
            if b.is_null() || !CGRectMakeWithDictionaryRepresentation(b as *const c_void, &mut bounds) {
                continue;
            }
            out.push(WinInfo {
                pid: num(d, kCGWindowOwnerPID).unwrap_or(0.0) as i32,
                number: num(d, kCGWindowNumber).unwrap_or(0.0) as i64,
                layer: num(d, kCGWindowLayer).unwrap_or(0.0) as i64,
                alpha: num(d, kCGWindowAlpha).unwrap_or(1.0),
                bounds,
                name: string(dict_get(d, kCGWindowName)).filter(|s| !s.is_empty()),
                owner: string(dict_get(d, kCGWindowOwnerName)).filter(|s| !s.is_empty()),
            });
        }
        CFRelease(arr);
        out
    })
}

/// A display: its full frame and its visible frame (without the menu bar and the Dock), both in points with a TOP-left origin.
pub struct Screen {
    pub frame: CGRect,
    pub visible: CGRect,
}

/// Every display, the primary (menu bar) one first. Main thread.
pub fn screens() -> Vec<Screen> {
    unsafe {
        let arr: Id = msg_send![class!(NSScreen), screens];
        if arr.is_null() {
            return vec![];
        }
        let n: usize = msg_send![arr, count];
        let mut raw: Vec<(CGRect, CGRect)> = vec![];
        for i in 0..n {
            let s: Id = msg_send![arr, objectAtIndex: i];
            let f: CGRect = msg_send![s, frame];
            let v: CGRect = msg_send![s, visibleFrame];
            raw.push((f, v));
        }
        let primary_h = raw.first().map(|(f, _)| f.size.height).unwrap_or(0.0);
        let flip = |r: CGRect| CGRect { origin: CGPoint { x: r.origin.x, y: primary_h - (r.origin.y + r.size.height) }, size: r.size };
        raw.into_iter().map(|(f, v)| Screen { frame: flip(f), visible: flip(v) }).collect()
    }
}

/// The pointer, physical px from the top-left of the primary display (NSEvent's is points from the bottom-left). Main thread.
pub fn cursor() -> (i32, i32) {
    let p: CGPoint = unsafe { msg_send![class!(NSEvent), mouseLocation] };
    let primary_h = screens().first().map(|s| s.frame.size.height).unwrap_or(0.0);
    let k = scale();
    ((p.x * k).round() as i32, ((primary_h - p.y) * k).round() as i32)
}

/// Physical pixels per point on the primary display (2.0 on Retina). Mixed-DPI setups use this one factor for every display.
pub fn scale() -> f64 {
    unsafe {
        let arr: Id = msg_send![class!(NSScreen), screens];
        if arr.is_null() {
            return 1.0;
        }
        let n: usize = msg_send![arr, count];
        if n == 0 {
            return 1.0;
        }
        let s: Id = msg_send![arr, objectAtIndex: 0usize];
        msg_send![s, backingScaleFactor]
    }
}

fn frontmost_app() -> Id {
    unsafe {
        let ws: Id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if ws.is_null() {
            return std::ptr::null_mut();
        }
        msg_send![ws, frontmostApplication]
    }
}

/// Process id of the active app.
pub fn frontmost_pid() -> Option<i32> {
    let app = frontmost_app();
    if app.is_null() {
        return None;
    }
    Some(unsafe { msg_send![app, processIdentifier] })
}

/// Executable name of the active app, lowercase (e.g. "google chrome", "code").
pub fn frontmost_exe() -> Option<String> {
    let app = frontmost_app();
    if app.is_null() {
        return None;
    }
    objc2::rc::autoreleasepool(|_| unsafe {
        let url: Id = msg_send![app, executableURL];
        if url.is_null() {
            return None;
        }
        let name: Id = msg_send![url, lastPathComponent];
        string(name).map(|s| s.to_lowercase())
    })
}

/// Seconds since the last keyboard or mouse input anywhere.
pub fn idle_seconds() -> u64 {
    const HID_SYSTEM_STATE: i32 = 1;
    const ANY_INPUT: u32 = !0;
    unsafe { CGEventSourceSecondsSinceLastEventType(HID_SYSTEM_STATE, ANY_INPUT).max(0.0) as u64 }
}

/// Is the left mouse button held right now?
pub fn left_button_down() -> bool {
    const COMBINED_SESSION_STATE: i32 = 0;
    const LEFT: u32 = 0;
    unsafe { CGEventSourceButtonState(COMBINED_SESSION_STATE, LEFT) }
}

pub fn beep() {
    unsafe { NSBeep() }
}

// ---------- screen pixels (backdrop sampling): needs the Screen Recording permission, checked first, never prompted here ----------
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
    fn CGGetDisplaysWithPoint(point: CGPoint, max: u32, displays: *mut u32, count: *mut u32) -> i32;
    fn CGDisplayBounds(display: u32) -> CGRect;
    // Marked obsolete in the macOS 15 headers but still exported and working on macOS 26 (checked 10 Nov 2026). If it ever returns
    // null, sampling just stops and cards keep their default scheme; ScreenCaptureKit is the replacement.
    fn CGDisplayCreateImageForRect(display: u32, rect: CGRect) -> *mut c_void;
    fn CGColorSpaceCreateDeviceRGB() -> *mut c_void;
    fn CGColorSpaceRelease(space: *mut c_void);
    fn CGBitmapContextCreate(data: *mut c_void, w: usize, h: usize, bpc: usize, bpr: usize, space: *mut c_void, info: u32) -> *mut c_void;
    fn CGContextDrawImage(ctx: *mut c_void, rect: CGRect, image: *mut c_void);
    fn CGContextRelease(ctx: *mut c_void);
    fn CGImageRelease(image: *mut c_void);
}

/// Average color of a screen rectangle (global points, top-left origin), from a 16x16 downsample kept in memory only.
/// None without the Screen Recording permission: without it macOS returns just the wallpaper, which would pick the wrong scheme.
pub fn screen_average_rgb(x: f64, y: f64, w: f64, h: f64) -> Option<[u8; 3]> {
    const N: usize = 16;
    unsafe {
        if w < 2.0 || h < 2.0 || !CGPreflightScreenCaptureAccess() {
            return None;
        }
        let (mut id, mut n) = (0u32, 0u32);
        if CGGetDisplaysWithPoint(CGPoint { x: x + w / 2.0, y: y + h / 2.0 }, 1, &mut id, &mut n) != 0 || n == 0 {
            return None;
        }
        let db = CGDisplayBounds(id);
        let local = CGRect { origin: CGPoint { x: x - db.origin.x, y: y - db.origin.y }, size: CGSize { width: w, height: h } };
        let img = CGDisplayCreateImageForRect(id, local);
        if img.is_null() {
            return None;
        }
        let mut px = vec![0u8; N * N * 4];
        let space = CGColorSpaceCreateDeviceRGB();
        const PREMULTIPLIED_LAST: u32 = 1; // RGBA, alpha last
        let ctx = CGBitmapContextCreate(px.as_mut_ptr() as *mut c_void, N, N, 8, N * 4, space, PREMULTIPLIED_LAST);
        let mut out = None;
        if !ctx.is_null() {
            CGContextDrawImage(ctx, CGRect { origin: CGPoint { x: 0.0, y: 0.0 }, size: CGSize { width: N as f64, height: N as f64 } }, img);
            let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
            for p in px.chunks_exact(4) {
                r += p[0] as u32;
                g += p[1] as u32;
                b += p[2] as u32;
            }
            let k = (N * N) as u32;
            out = Some([(r / k) as u8, (g / k) as u8, (b / k) as u8]);
            CGContextRelease(ctx);
        }
        CGColorSpaceRelease(space);
        CGImageRelease(img);
        out
    }
}

/// The system accent for the theme: none on macOS (the theme keeps its own accent, platforms.mac in ui/themes.json).
pub fn accent_rgb() -> Option<[u8; 3]> {
    None
}
