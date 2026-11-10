//! The app and its windows on Windows: what tao/wry need set differently from macOS. Called from host/mod.rs and surface/mod.rs.

use tao::event_loop::EventLoop;
use tao::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
use tao::window::{Window, WindowBuilder};
use wry::{WebViewBuilder, WebViewBuilderExtWindows};

/// The page every surface loads: wry serves a custom protocol as http://<name>.localhost/ with WebView2.
pub const START_URL: &str = "http://morc.localhost/index.html";

const DEFAULT_WV_ARGS: &str = "--use-angle=swiftshader --enable-unsafe-swiftshader";

/// Nothing to set on Windows: tool windows (window_style.rs) already keep cards out of the taskbar and Alt-Tab.
pub fn configure_event_loop<T: 'static>(_event_loop: &mut EventLoop<T>) {}

/// No taskbar entry, no redirection bitmap, no undecorated shadow.
pub fn window_builder(builder: WindowBuilder) -> WindowBuilder {
    builder
        .with_skip_taskbar(true)
        .with_no_redirection_bitmap(true) // no default GDI surface behind the transparent page (suspected source of the pale top band)
        .with_undecorated_shadow(false)
}

/// Graphics backend. Hardware WebGL gets its context reset within 20-70 s on this machine (Intel Iris Xe + a transparent window),
/// leaving a dead canvas; Google's SwiftShader (software) is stable and plenty for these scenes. WebView2 allows ONE configuration per
/// browser process, so this is host-wide. MORC_WV_ARGS=gpu keeps hardware; any other value is passed through as extra browser flags.
pub fn webview_builder(wvb: WebViewBuilder) -> WebViewBuilder {
    let extra = match std::env::var("MORC_WV_ARGS").ok().as_deref() {
        Some("gpu") => None,
        Some(a) => Some(a.to_string()),
        None => Some(DEFAULT_WV_ARGS.to_string()),
    };
    match extra {
        Some(extra) => wvb.with_additional_browser_args(format!("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection {extra}")), // wry's own default flag is kept
        None => wvb,
    }
}

/// The handle the platform layer works with: the HWND.
pub fn native_handle(window: &Window) -> isize {
    window.hwnd()
}
