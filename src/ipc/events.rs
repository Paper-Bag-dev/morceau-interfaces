//! The one event type that flows into the main (UI) thread from every other thread.

use crate::protocol::Request;
use std::sync::mpsc::Sender;
use tao::event_loop::EventLoopProxy;

pub enum Ev {
    /// A request from the CLI / a trigger. The Sender, if present, gets one JSON reply string.
    Cmd(Request, Option<Sender<String>>),
    /// A message from a webview page: (surface key, raw JSON text).
    Ipc(u64, String),
    /// The active window changed.
    Foreground,
    /// The pointer went onto (true) or off (false) a card's content: (native window handle, inside). macOS only: its web views do not
    /// see the pointer in a window of an inactive app, so the platform's hit test reports it (Windows' WebView2 sees it by itself).
    #[allow(dead_code)] // only the platforms whose web view cannot see the pointer send it (platform/<os>/window_style)
    Hover(isize, bool),
    /// Where the pointer is on a card (native window handle, x, y in points from the window's top-left), while it is on it. macOS only,
    /// same reason: the page marks what is under it so every hover style works without a click.
    #[allow(dead_code)] // only the platforms whose web view cannot see the pointer send it (platform/<os>/window_style)
    Pointer(isize, f64, f64),
}

pub type Proxy = EventLoopProxy<Ev>;
