//! What is behind a window? Samples the screen rectangle where a surface is about to appear (it is still invisible then, so it never
//! samples itself) and returns its average color, so the page picks its scheme exactly as on Windows (core/theme.js adaptToBackdrop:
//! dark page -> light surfaces, light page -> heavier outline, else the default dark glass). Local only: a 16x16 downsample averaged in
//! memory and discarded. Needs the Screen Recording permission; without it there is no sample and the card keeps the default dark glass.
//! Same signature as the Windows file so host/layout.rs is unchanged. The raw CoreGraphics calls live in sys.rs.

/// l, t, w, h: physical px from the top-left of the primary screen (the host's coordinates).
pub fn average_rgb(l: i32, t: i32, w: i32, h: i32) -> Option<[u8; 3]> {
    let s = super::sys::scale();
    let s = if s > 0.0 { s } else { 1.0 };
    super::sys::screen_average_rgb(l as f64 / s, t as f64 / s, w as f64 / s, h as f64 / s)
}
