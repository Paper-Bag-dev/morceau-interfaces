//! One floating surface: a borderless transparent window + its webview + its lifetime state.

use crate::{assets, ev::{Ev, Proxy}, lifetime::Life, placement::R, protocol::Opts, winstyle};
use serde_json::Value;
use std::time::Instant;
use tao::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    platform::windows::{WindowBuilderExtWindows, WindowExtWindows},
    window::{Window, WindowBuilder},
};
use wry::{WebView, WebViewBuilder, WebViewBuilderExtWindows};

const DEFAULT_WV_ARGS: &str = "--use-angle=swiftshader --enable-unsafe-swiftshader";

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
        let window = WindowBuilder::new()
            .with_title("")
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top(true)
            .with_resizable(false)
            .with_visible(false)
            .with_focused(false)
            .with_skip_taskbar(true)
            .with_no_redirection_bitmap(true) // no default GDI surface behind the transparent page (suspected source of the pale top band)
            .with_undecorated_shadow(false)
            .with_inner_size(LogicalSize::new(10.0, 10.0))
            .build(target)
            .map_err(|e| e.to_string())?;
        winstyle::overlay_styles(window.hwnd());
        let mut wvb = WebViewBuilder::new();
        // Graphics backend. Hardware WebGL gets its context reset within 20-70 s on this machine (Intel Iris Xe + a transparent window),
        // leaving a dead canvas; Google's SwiftShader (software) is stable and plenty for these scenes. WebView2 allows ONE configuration per
        // browser process, so this is host-wide. MORC_WV_ARGS=gpu keeps hardware; any other value is passed through as extra browser flags.
        let extra = match std::env::var("MORC_WV_ARGS").ok().as_deref() {
            Some("gpu") => None,
            Some(a) => Some(a.to_string()),
            None => Some(DEFAULT_WV_ARGS.to_string()),
        };
        if let Some(extra) = extra {
            wvb = wvb.with_additional_browser_args(format!("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection {extra}"));   // wry's own default flag is kept
        }
        let webview = wvb
            .with_transparent(true)
            .with_background_color((0, 0, 0, 0))
            .with_accept_first_mouse(true)
            .with_custom_protocol("morc".into(), |_id, req| assets::handle(req))
            .with_ipc_handler(move |req| {
                let _ = proxy.send_event(Ev::Ipc(key, req.body().clone()));
            })
            .with_url("http://morc.localhost/index.html")
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

    pub fn hwnd(&self) -> isize {
        self.window.hwnd()
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
