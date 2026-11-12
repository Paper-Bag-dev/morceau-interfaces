//! The app and its windows on macOS: what tao/wry need set differently from Windows. Called from host/mod.rs and surface/mod.rs.

use tao::event_loop::EventLoop;
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS, WindowBuilderExtMacOS, WindowExtMacOS};
use tao::window::{Window, WindowBuilder};

/// The page every surface loads: wry serves a custom protocol as <name>://localhost/ with WKWebView (assets::handle only reads the path).
pub const START_URL: &str = "morc://localhost/index.html";
/// The hidden page that opens every card (surface/root.rs).
pub const ROOT_URL: &str = "morc://localhost/root.html";

/// An accessory app: no Dock icon, not in Cmd-Tab (like the tool windows on Windows). And no activation at launch: tao activates the app
/// by default, which takes focus from the window you are in, so the "active window" would be ours and placement has nothing to sit beside.
pub fn configure_event_loop<T: 'static>(event_loop: &mut EventLoop<T>) {
    event_loop.set_activation_policy(ActivationPolicy::Accessory);
    event_loop.set_activate_ignoring_other_apps(false);
}

/// No system shadow: the page draws its own edge.
pub fn window_builder(builder: WindowBuilder) -> WindowBuilder {
    builder.with_has_shadow(false)
}

/// The handle the platform layer works with: the NSWindow pointer.
pub fn native_handle(window: &Window) -> isize {
    window.ns_window() as isize
}

/// Nothing to add: WKWebView has no browser flags and uses the GPU (MORC_WV_ARGS is Windows-only).
pub fn webview_builder(wvb: wry::WebViewBuilder) -> wry::WebViewBuilder {
    wvb
}

/// Cards open as linked pages of one hidden root page (surface/root.rs): WebKit keeps a page and the windows it opens in ONE web
/// process (they can reach each other's JavaScript), so 20 cards cost one process instead of 20, and they can share state.
/// MORC_LINKED=0 turns it off (one process per card, as before): a switch for comparing and for falling back.
pub fn linked_cards() -> bool {
    std::env::var("MORC_LINKED").map(|v| v != "0").unwrap_or(true)
}

/// WebKit blocks window.open() from script unless the page's preferences allow it; the root page opens every card by script.
pub fn allow_script_windows(wv: &wry::WebView) {
    use wry::WebViewExtMacOS;
    unsafe { wv.webview().configuration().preferences().setJavaScriptCanOpenWindowsAutomatically(true) };
}

/// A card's web view built on its opener's configuration: same process, same morc:// handler, same ipc channel (wry skips adding them again).
pub fn linked_builder<'a>(wvb: wry::WebViewBuilder<'a>, req: &wry::NewWindowFeatures) -> wry::WebViewBuilder<'a> {
    use wry::WebViewBuilderExtMacos;
    wvb.with_webview_configuration(req.opener.target_configuration.clone())
}

/// Hand the built card web view back to WebKit as the window it asked for.
pub fn linked_response(wv: &wry::WebView) -> wry::NewWindowResponse {
    use wry::WebViewExtMacOS;
    wry::NewWindowResponse::Create { webview: objc2::rc::Retained::into_super(wv.webview()) }
}
