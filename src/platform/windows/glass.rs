//! Native glass behind each card's plate (Windows). A transparent WebView2 cannot blur the apps behind it (CSS `backdrop-filter` only blurs
//! the page), and blurring the card window itself would also blur its hit margin and corner hot zone. So each card gets a second window
//! UNDER it: borderless, never activated, exactly the plates' bounds that the page reports (layout `glass`), with DWM's acrylic (or plain
//! blur) behind it and Windows 11's own rounded corners. (DWM draws the accent across the whole window and ignores a window region, so
//! the glass window IS the plate's rect; the fluent skin gives plates the same radius as Windows 11's rounding.) The page then draws its tint, grain and sheen on top, as on macOS.
//!
//! The glass follows its card with no lag: the card window is subclassed, and every position, size, z-order or visibility change of the
//! card (drag, glide, stack depth, hide) moves the glass right under it in the same message. `MORC_WIN_GLASS` = acrylic (default) | blur | off.
//! Main thread only (tao's event loop), like every other window call here.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use windows::core::{s, w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};
use windows::Win32::Graphics::Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::*;

const SUBCLASS_ID: usize = 0x6d6f_7263; // "morc"

thread_local! {
    /// card window -> (its glass window, the plates' bounds relative to the card window: left, top, right, bottom)
    static GLASS: RefCell<HashMap<isize, (isize, (i32, i32, i32, i32))>> = RefCell::new(HashMap::new());
}

fn h(x: isize) -> HWND {
    HWND(x as *mut c_void)
}

/// What sits under the plates: DWM accent state (4 acrylic, 3 blur) or none.
fn mode() -> Option<u32> {
    match std::env::var("MORC_WIN_GLASS").unwrap_or_default().to_ascii_lowercase().as_str() {
        "off" | "0" | "none" => None,
        "blur" => Some(3),
        _ => Some(4),
    }
}

#[repr(C)]
struct AccentPolicy {
    state: u32,
    flags: u32,
    /// tint under the page's own tint, AABBGGRR; the page draws the real color, this only keeps acrylic from going fully clear
    color: u32,
    anim: u32,
}

#[repr(C)]
struct CompositionData {
    attrib: u32,
    data: *mut c_void,
    size: usize,
}

/// SetWindowCompositionAttribute (user32, undocumented but stable since Windows 10): the only way to put acrylic under a borderless,
/// non-framed popup. Looked up by name so a system without it just gets no glass.
unsafe fn accent(hwnd: HWND, state: u32) {
    type Swca = unsafe extern "system" fn(HWND, *mut CompositionData) -> i32;
    let Ok(user32) = GetModuleHandleW(w!("user32.dll")) else { return };
    let Some(f) = GetProcAddress(user32, s!("SetWindowCompositionAttribute")) else { return };
    let f: Swca = std::mem::transmute(f);
    let mut p = AccentPolicy { state, flags: 0, color: 0x1012_1212, anim: 0 };
    let mut d = CompositionData { attrib: 19, data: &mut p as *mut _ as *mut c_void, size: std::mem::size_of::<AccentPolicy>() }; // WCA_ACCENT_POLICY
    f(hwnd, &mut d);
}

unsafe extern "system" fn glass_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_NCHITTEST {
        return LRESULT(-1); // HTTRANSPARENT: the glass never takes the pointer
    }
    DefWindowProcW(hwnd, msg, wp, lp)
}

fn class() -> PCWSTR {
    static ONCE: std::sync::Once = std::sync::Once::new();
    let name = w!("MorcGlass");
    ONCE.call_once(|| unsafe {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(glass_proc),
            hInstance: GetModuleHandleW(None).map(|m| HINSTANCE(m.0)).unwrap_or_default(),
            lpszClassName: name,
            // black = fully clear for DWM: the acrylic shows through everywhere the region allows
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            ..Default::default()
        };
        RegisterClassW(&wc);
    });
    name
}

unsafe fn create(state: u32) -> Option<HWND> {
    let ex = WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT;
    let inst = GetModuleHandleW(None).ok().map(|m| HINSTANCE(m.0));
    let g = CreateWindowExW(ex, class(), w!(""), WS_POPUP, 0, 0, 0, 0, None, None, inst, None).ok()?;
    let none: u32 = 0xFFFF_FFFE; // DWMWA_COLOR_NONE: no Windows 11 frame line
    let _ = DwmSetWindowAttribute(g, DWMWA_BORDER_COLOR, &none as *const _ as *const c_void, 4);
    let corners: i32 = DWMWCP_ROUND.0; // Windows 11's own rounding (ignored on Windows 10)
    let _ = DwmSetWindowAttribute(g, DWMWA_WINDOW_CORNER_PREFERENCE, &corners as *const _ as *const c_void, 4);
    accent(g, state);
    Some(g)
}

/// Put the glass right under its card, over the plates, shown only while the card is.
unsafe fn follow(card: HWND, g: HWND, b: (i32, i32, i32, i32)) {
    if !IsWindowVisible(card).as_bool() {
        let _ = ShowWindow(g, SW_HIDE);
        return;
    }
    let mut r = RECT::default();
    if GetWindowRect(card, &mut r).is_err() {
        return;
    }
    let _ = SetWindowPos(g, Some(card), r.left + b.0, r.top + b.1, b.2 - b.0, b.3 - b.1, SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_NOOWNERZORDER);
}

unsafe extern "system" fn card_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM, _id: usize, _data: usize) -> LRESULT {
    let r = DefSubclassProc(hwnd, msg, wp, lp);
    let g = GLASS.with(|m| m.borrow().get(&(hwnd.0 as isize)).copied());
    if let Some((g, b)) = g {
        match msg {
            WM_WINDOWPOSCHANGED | WM_SHOWWINDOW => follow(hwnd, h(g), b),
            WM_NCDESTROY => {
                GLASS.with(|m| m.borrow_mut().remove(&(hwnd.0 as isize)));
                let _ = DestroyWindow(h(g));
                let _ = RemoveWindowSubclass(hwnd, Some(card_proc), SUBCLASS_ID);
            }
            _ => {}
        }
    }
    r
}

/// rects: physical px relative to the card window's top-left (left, top, right, bottom, radius), one per plate. Empty = no glass.
pub fn set_glass(hwnd: isize, rects: &[(i32, i32, i32, i32, i32)]) {
    let Some(state) = mode() else { return };
    let card = h(hwnd);
    unsafe {
        let have = GLASS.with(|m| m.borrow().get(&hwnd).map(|x| x.0));
        if rects.is_empty() {
            if let Some(g) = have {
                let _ = ShowWindow(h(g), SW_HIDE);
            }
            return;
        }
        let g = match have {
            Some(g) => h(g),
            None => {
                let Some(g) = create(state) else { return };
                let _ = SetWindowSubclass(card, Some(card_proc), SUBCLASS_ID, 0);
                g
            }
        };
        // the plates' bounds, 1 px inside so the blur never peeks past the page's anti-aliased rim
        let b = (
            rects.iter().map(|r| r.0).min().unwrap_or(0) + 1,
            rects.iter().map(|r| r.1).min().unwrap_or(0) + 1,
            rects.iter().map(|r| r.2).max().unwrap_or(0) - 1,
            rects.iter().map(|r| r.3).max().unwrap_or(0) - 1,
        );
        GLASS.with(|m| m.borrow_mut().insert(hwnd, (g.0 as isize, b)));
        follow(card, g, b);
    }
}
