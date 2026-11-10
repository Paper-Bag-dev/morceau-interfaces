//! The app and its windows on macOS: what tao/wry need set differently from Windows. Called from host/mod.rs and surface/mod.rs.

use tao::event_loop::EventLoop;
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS, WindowBuilderExtMacOS, WindowExtMacOS};
use tao::window::{Window, WindowBuilder};

/// The page every surface loads: wry serves a custom protocol as <name>://localhost/ with WKWebView (assets::handle only reads the path).
pub const START_URL: &str = "morc://localhost/index.html";

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
