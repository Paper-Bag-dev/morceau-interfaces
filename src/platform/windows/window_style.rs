//! Window styling that tao/wry don't expose: never steal focus, hide from Alt-Tab/taskbar,
//! and shape the window to its content (a union of rounded rects) so everything else is click-through.

use std::ffi::c_void;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{CombineRgn, CreateRectRgn, CreateRoundRectRgn, DeleteObject, SetWindowRgn, HGDIOBJ, RGN_OR};
use windows::Win32::UI::WindowsAndMessaging::*;

fn h(hwnd: isize) -> HWND {
    HWND(hwnd as *mut c_void)
}

/// WS_EX_NOACTIVATE (clicks don't take focus) + WS_EX_TOOLWINDOW (no Alt-Tab / taskbar entry),
/// and no DWM chrome: Windows 11 otherwise paints a thin frame line along the top (and a shadow/rounding) when the
/// window is clicked. Our shape comes from the window region, so all of that is switched off.
pub fn overlay_styles(hwnd: isize) {
    use windows::Win32::Graphics::Dwm::*;
    unsafe {
        let none: u32 = 0xFFFF_FFFE; // DWMWA_COLOR_NONE
        let _ = DwmSetWindowAttribute(h(hwnd), DWMWA_BORDER_COLOR, &none as *const _ as *const c_void, 4);
        let corners: i32 = DWMWCP_DONOTROUND.0;
        let _ = DwmSetWindowAttribute(h(hwnd), DWMWA_WINDOW_CORNER_PREFERENCE, &corners as *const _ as *const c_void, 4);
        // No non-client area at all: no caption, frame, system menu or min/max/close box, ever (not even when clicked).
        let st = GetWindowLongPtrW(h(hwnd), GWL_STYLE);
        let chrome = (WS_CAPTION.0 | WS_THICKFRAME.0 | WS_SYSMENU.0 | WS_MINIMIZEBOX.0 | WS_MAXIMIZEBOX.0 | WS_BORDER.0 | WS_DLGFRAME.0) as isize;
        SetWindowLongPtrW(h(hwnd), GWL_STYLE, (st & !chrome) | WS_POPUP.0 as isize);
        let _ = SetWindowPos(h(hwnd), None, 0, 0, 0, 0, SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
        let ex = GetWindowLongPtrW(h(hwnd), GWL_EXSTYLE);
        // APPWINDOW would force a taskbar button / frame behaviour; these are tool windows, never app windows.
        let ex = (ex | WS_EX_NOACTIVATE.0 as isize | WS_EX_TOOLWINDOW.0 as isize) & !(WS_EX_APPWINDOW.0 as isize);
        SetWindowLongPtrW(h(hwnd), GWL_EXSTYLE, ex);
    }
}

/// While a window is being carried, the ordinary arrow becomes the four-arrow "move" cursor, system-wide (the native move
/// loop owns the pointer, so a page-level CSS cursor can't show). Tied to onDragStart; `drag_cursor_off` restores the user's
/// own cursor scheme from the registry, so it is safe to call any time (also on host start and exit in case of a crash).
pub fn drag_cursor_on() {
    unsafe {
        if let Ok(c) = LoadCursorW(None, IDC_SIZEALL) {
            if let Ok(copy) = CopyIcon(HICON(c.0)) {
                let _ = SetSystemCursor(HCURSOR(copy.0), OCR_NORMAL);
            }
        }
    }
}

pub fn drag_cursor_off() {
    unsafe {
        let _ = SystemParametersInfoW(SPI_SETCURSORS, 0, None, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));
    }
}

/// Text inputs need real keyboard focus; everything else must never take it.
pub fn set_noactivate(hwnd: isize, on: bool) {
    unsafe {
        let ex = GetWindowLongPtrW(h(hwnd), GWL_EXSTYLE);
        let bit = WS_EX_NOACTIVATE.0 as isize;
        SetWindowLongPtrW(h(hwnd), GWL_EXSTYLE, if on { ex | bit } else { ex & !bit });
    }
}

/// Show without activating and keep above normal windows.
pub fn show_topmost_noactivate(hwnd: isize) {
    unsafe {
        let _ = ShowWindow(h(hwnd), SW_SHOWNOACTIVATE);
        let _ = SetWindowPos(h(hwnd), Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

pub fn hide(hwnd: isize) {
    unsafe {
        let _ = ShowWindow(h(hwnd), SW_HIDE);
    }
}

/// Hover reaches the page by itself on Windows (WebView2 sees the pointer without the app being active): nothing to watch.
pub fn watch_hover(_proxy: crate::ev::Proxy) {}

/// Native glass under each plate: macOS only (there it is an NSGlassEffectView behind the web view); the Windows plate keeps its CSS blur.
pub fn set_glass(_hwnd: isize, _rects: &[(i32, i32, i32, i32, i32)]) {}

/// Clip the window to the union of rounded rects (physical px: left, top, right, bottom, radius).
/// An empty list removes the region.
pub fn set_region(hwnd: isize, rects: &[(i32, i32, i32, i32, i32)]) {
    unsafe {
        if rects.is_empty() {
            SetWindowRgn(h(hwnd), None, true);
            return;
        }
        let all = CreateRectRgn(0, 0, 0, 0);
        for &(l, t, r, b, rad) in rects {
            let piece = CreateRoundRectRgn(l, t, r + 1, b + 1, rad * 2, rad * 2);
            CombineRgn(Some(all), Some(all), Some(piece), RGN_OR);
            let _ = DeleteObject(HGDIOBJ(piece.0));
        }
        // The system owns the region after this call; do not delete it.
        SetWindowRgn(h(hwnd), Some(all), true);
    }
}
