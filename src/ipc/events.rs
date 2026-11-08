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
}

pub type Proxy = EventLoopProxy<Ev>;
