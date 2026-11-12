//! The app and its windows on Windows: what tao/wry need set differently from macOS. Called from host/mod.rs and surface/mod.rs.

use tao::event_loop::EventLoop;
use tao::platform::windows::{WindowBuilderExtWindows, WindowExtWindows};
use tao::window::{Window, WindowBuilder};
use wry::{WebViewBuilder, WebViewBuilderExtWindows, WebViewExtWindows};

/// The page every surface loads: wry serves a custom protocol as http://<name>.localhost/ with WebView2.
pub const START_URL: &str = "http://morc.localhost/index.html";
/// The hidden page that opens every card (surface/root.rs).
pub const ROOT_URL: &str = "http://morc.localhost/root.html";

/// Software rendering (Google SwiftShader), opt-in with MORC_WV_ARGS=soft.
const SOFT_WV_ARGS: &str = "--use-angle=swiftshader --enable-unsafe-swiftshader";

/// Nothing to set on Windows: tool windows (window_style.rs) already keep cards out of the taskbar and Alt-Tab.
pub fn configure_event_loop<T: 'static>(_event_loop: &mut EventLoop<T>) {}

/// No taskbar entry, no redirection bitmap, no undecorated shadow.
pub fn window_builder(builder: WindowBuilder) -> WindowBuilder {
    builder
        .with_skip_taskbar(true)
        .with_no_redirection_bitmap(true) // no default GDI surface behind the transparent page (suspected source of the pale top band)
        .with_undecorated_shadow(false)
}

/// Graphics backend: the GPU by default, so card motion (springs, clip shapes, shadows) runs on the GPU as it does on macOS. Software
/// rendering was the default because hardware WebGL lost its context within 20-70 s on one machine (Intel Iris Xe + a transparent
/// window); it made every animation run on the CPU at a low frame rate, and 3D cards now rebuild themselves after a lost context
/// (ui/plugins/model3d.js). WebView2 allows ONE configuration per browser process, so this is host-wide. MORC_WV_ARGS=soft = SwiftShader;
/// any other value is passed through as extra browser flags.
pub fn webview_builder(wvb: WebViewBuilder) -> WebViewBuilder {
    let extra = match std::env::var("MORC_WV_ARGS").ok().as_deref() {
        None | Some("gpu") | Some("") => None,
        Some("soft") => Some(SOFT_WV_ARGS.to_string()),
        Some(a) => Some(a.to_string()),
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

/// Linked cards (one hidden root page window.open()s every card; docs/process-model.md), on by default like macOS: the cards share the
/// root's WebView2 environment, so `window.opener.morcShared` is one object for all. MORC_LINKED=0 turns it off (the host relays the
/// shared store instead).
pub fn linked_cards() -> bool {
    std::env::var("MORC_LINKED").map(|v| v != "0").unwrap_or(true)
}

/// WebView2 lets a page open windows when the new-window handler answers; nothing to switch on.
pub fn allow_script_windows(_wv: &wry::WebView) {}

/// A card opened by the root: built on the opener's environment (WebView2 requires the same one for a popup).
pub fn linked_builder<'a>(wvb: WebViewBuilder<'a>, req: &wry::NewWindowFeatures) -> WebViewBuilder<'a> {
    wvb.with_environment(req.opener.environment.clone())
}

/// Hand the new web view to WebView2 as the popup's target.
pub fn linked_response(wv: &wry::WebView) -> wry::NewWindowResponse {
    wry::NewWindowResponse::Create { webview: wv.webview() }
}
