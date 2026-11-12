//! Small input and feedback calls on Windows: idle time, the mouse button, the beep. (macOS has the same names in platform/macos/sys.rs.)

/// Seconds since the last keyboard or mouse input anywhere in Windows.
pub fn idle_seconds() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    unsafe {
        let mut li = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        if GetLastInputInfo(&mut li).as_bool() {
            (GetTickCount().wrapping_sub(li.dwTime) / 1000) as u64
        } else {
            0
        }
    }
}

/// Is the left mouse button held right now?
pub fn left_button_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}

pub fn beep() {
    unsafe {
        let _ = windows::Win32::System::Diagnostics::Debug::MessageBeep(windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE(0x40));
    }
}

/// The pointer, physical px (the process is per-monitor DPI aware).
pub fn cursor() -> (i32, i32) {
    let mut p = windows::Win32::Foundation::POINT::default();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut p);
    }
    (p.x, p.y)
}

/// The user's Windows accent as the dark-mode fill Windows 11 itself uses for accent buttons ("AccentLight2" in the accent palette).
/// None when the palette is missing (older Windows) or unreadable; the theme then keeps its own accent.
pub fn accent_rgb() -> Option<[u8; 3]> {
    use windows::core::w;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_BINARY};
    let mut buf = [0u8; 32];
    let mut len = buf.len() as u32;
    let ok = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, w!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent"), w!("AccentPalette"), RRF_RT_REG_BINARY, None, Some(buf.as_mut_ptr() as *mut _), Some(&mut len))
    };
    // 8 colors of 4 bytes (R, G, B, A): Light3, Light2, Light1, Base, Dark1, Dark2, Dark3, ...
    (ok.is_ok() && len >= 8).then(|| [buf[4], buf[5], buf[6]])
}
