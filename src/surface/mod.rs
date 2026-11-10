//! One floating surface: a borderless transparent window + its webview + its lifetime state.

use crate::{assets, ev::{Ev, Proxy}, placement::R, protocol::Opts, window_style};
use self::lifetime::Life;
use serde_json::Value;
use std::time::Instant;
use tao::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};
use wry::{WebView, WebViewBuilder};

use crate::platform::os::app::{native_handle, START_URL};

#[derive(PartialEq, Clone, Copy)]
pub enum Phase {
    /// Warm and hidden, waiting to be claimed.
    Spare,
    /// Claimed; waiting for the page to report its first layout.
    Waiting,
    Visible,
    /// Playing its exit animation (since).
    Leaving,
}

pub struct Move {
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub start: Instant,
}

pub struct Surface {
    pub key: u64,
    pub id: String,
    pub name: String,
    pub topic: String,
    pub opts: Opts,
    pub phase: Phase,
    pub phase_since: Instant,
    pub ready: bool,
    pub init: Option<Value>,
    pub last_init: Value,
    pub life: Life,
    pub user_moved: bool,
    /// transparent padding around the visible content (physical px: left, top, right, bottom); placement aligns the CONTENT, not the window
    pub inset: (i32, i32, i32, i32),
    /// when the host last moved this surface by itself (rate-limits auto re-placement so it never hops)
    pub auto_moved_at: Option<std::time::Instant>,
    /// While the user carries the window: where it started, and when. Cleared when the mouse button is released.
    pub drag_from: Option<R>,
    pub drag_started: Option<Instant>,
    pub mv: Option<Move>,
    // Field order matters: the webview must drop before the window that hosts it.
    pub webview: WebView,
    pub window: Window,
}

impl Surface {
    pub fn create(target: &EventLoopWindowTarget<Ev>, key: u64, proxy: Proxy) -> Result<Surface, String> {
        let builder = WindowBuilder::new()
            .with_title("")
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(true)
            .with_resizable(false)
            .with_visible(false)
            .with_focused(false)
            .with_inner_size(LogicalSize::new(10.0, 10.0));
        let builder = crate::platform::os::app::window_builder(builder);
        let window = builder.build(target).map_err(|e| e.to_string())?;
        window_style::overlay_styles(native_handle(&window));
        let wvb = crate::platform::os::app::webview_builder(WebViewBuilder::new());
        let webview = wvb
            .with_transparent(true)
            .with_background_color((0, 0, 0, 0))
            .with_accept_first_mouse(true)
            .with_custom_protocol("morc".into(), |_id, req| assets::handle(req))
            .with_ipc_handler(move |req| {
                let _ = proxy.send_event(Ev::Ipc(key, req.body().clone()));
            })
            .with_url(START_URL)
            .build(&window)
            .map_err(|e| e.to_string())?;
        let now = Instant::now();
        Ok(Surface {
            key,
            id: String::new(),
            name: String::new(),
            topic: String::new(),
            opts: Opts::default(),
            phase: Phase::Spare,
            phase_since: now,
            ready: false,
            init: None,
            last_init: Value::Null,
            life: Life::new(None, false, now),
            user_moved: false,
            inset: (0, 0, 0, 0),
            auto_moved_at: None,
            drag_from: None,
            drag_started: None,
            mv: None,
            window,
            webview,
        })
    }

    /// The native window handle for the platform layer (HWND on Windows, NSWindow pointer on macOS); the name is kept from Windows.
    pub fn hwnd(&self) -> isize {
        native_handle(&self.window)
    }

    /// Send a message to the page.
    pub fn send(&self, msg: &Value) {
        let js = msg.to_string().replace('\u{2028}', "\\u2028").replace('\u{2029}', "\\u2029");
        let _ = self.webview.evaluate_script(&format!("window.morc&&window.morc.recv({js})"));
    }

    /// Outer rect in physical pixels.
    pub fn rect(&self) -> Option<R> {
        let p = self.window.outer_position().ok()?;
        let s = self.window.outer_size();
        Some(R { l: p.x, t: p.y, r: p.x + s.width as i32, b: p.y + s.height as i32 })
    }
}

// Submodules: a card's lifetime, the actions its buttons run, and the spec linter.
pub mod actions;
pub mod lifetime;
pub mod lint;
