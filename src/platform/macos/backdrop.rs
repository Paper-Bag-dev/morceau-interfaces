//! What is behind a window? On macOS, reading other apps' pixels needs the Screen Recording permission (and ScreenCaptureKit, since
//! CGWindowListCreateImage is gone in macOS 15). Until that is worth asking for, there is no sample: the page keeps its own scheme, exactly
//! as with `contrast: "off"`. Same signature as the Windows file so host/layout.rs is unchanged.

pub fn average_rgb(_l: i32, _t: i32, _w: i32, _h: i32) -> Option<[u8; 3]> {
    None
}
