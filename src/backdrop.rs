//! What is behind a window? Samples the screen rectangle where a surface is about to appear and returns its
//! average color, so the page can pick an "anti" scheme (light surfaces over dark pages, and the reverse) and a
//! complementary accent. Local only: a 16x16 downsample is averaged in memory and discarded; nothing is saved or sent.

use std::ffi::c_void;
use windows::Win32::Graphics::Gdi::*;

const N: i32 = 16;

pub fn average_rgb(l: i32, t: i32, w: i32, h: i32) -> Option<[u8; 3]> {
    if w < 4 || h < 4 {
        return None;
    }
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return None;
        }
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, N, N);
        let old = SelectObject(mem, HGDIOBJ(bmp.0));
        SetStretchBltMode(mem, HALFTONE);
        let ok = StretchBlt(mem, 0, 0, N, N, Some(screen), l, t, w, h, SRCCOPY).as_bool();

        let mut out = None;
        if ok {
            let mut bi = BITMAPINFO::default();
            bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bi.bmiHeader.biWidth = N;
            bi.bmiHeader.biHeight = -N; // top-down
            bi.bmiHeader.biPlanes = 1;
            bi.bmiHeader.biBitCount = 32;
            let mut px = vec![0u8; (N * N * 4) as usize];
            let got = GetDIBits(mem, bmp, 0, N as u32, Some(px.as_mut_ptr() as *mut c_void), &mut bi, DIB_RGB_COLORS);
            if got > 0 {
                let (mut r, mut g, mut b) = (0u32, 0u32, 0u32);
                for p in px.chunks_exact(4) {
                    b += p[0] as u32;
                    g += p[1] as u32;
                    r += p[2] as u32;
                }
                let n = (N * N) as u32;
                out = Some([(r / n) as u8, (g / n) as u8, (b / n) as u8]);
            }
        }
        SelectObject(mem, old);
        let _ = DeleteObject(HGDIOBJ(bmp.0));
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        out
    }
}
