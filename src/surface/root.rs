//! The hidden root page that opens every card (linked cards, macOS). WebKit keeps a page and the windows it opens with window.open() in
//! ONE web process, because they can reach each other's JavaScript: 20 cards cost one process (measured: +1 process / ~44 MB against
//! +20 / ~590 MB for unlinked cards), and every card can reach the root's shared state as `window.opener.morcShared`.
//!
//! Flow: the host makes the card's window, parks it here under its key and asks the root page to `window.open('...index.html?k=<key>')`;
//! wry's new-window handler finds the parked window by that key and builds the card's web view on the root's configuration.
//! A card's messages carry its key (`k<key>\n`, ui/core/bridge.js) and go on as Ev::Ipc. On macOS a card shares the root's ipc channel
//! and file handler (one configuration); on Windows each WebView2 needs its own, so every card is given the same ones (`handlers`).
//! Both platforms: WebView2 popups share the root's environment, WKWebView popups its configuration (platform/<os>/app.rs).

use crate::{assets, ev::{Ev, Proxy}, util};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use tao::{event_loop::EventLoopWindowTarget, window::{Window, WindowBuilder}};
use wry::{NewWindowResponse, WebView, WebViewBuilder};

use crate::platform::os::app;

/// Where a linked card's web view lands once WebKit asks for it; empty until then.
pub type Slot = Rc<RefCell<Option<WebView>>>;


/// Cards waiting for their web view: key -> (the card's window, its slot). The window is boxed in the Surface, so the pointer stays valid
/// until the surface is destroyed, which removes the entry first (Root::forget).
type Parked = Rc<RefCell<HashMap<u64, (*const Window, Slot)>>>;

pub struct Root {
    parked: Parked,
    ready: bool,
    queued: Vec<u64>,
    // Field order matters: the web view must drop before the window that hosts it.
    webview: WebView,
    _window: Window,
}

impl Root {
    pub fn create(target: &EventLoopWindowTarget<Ev>, proxy: Proxy) -> Result<Root, String> {
        let window = app::window_builder(WindowBuilder::new().with_title("").with_visible(false)).build(target).map_err(|e| e.to_string())?;
        let parked: Parked = Rc::new(RefCell::new(HashMap::new()));
        let p = parked.clone();
        let card_proxy = proxy.clone();
        // the root uses the cards' builder too: WebView2 allows ONE browser configuration per process (app::webview_builder)
        let webview = handlers(super::card_builder(WebViewBuilder::new()), proxy)
            .with_new_window_req_handler(move |url, req| {
                let Some(key) = key_of(&url) else { return NewWindowResponse::Deny };
                let Some((win, slot)) = p.borrow_mut().remove(&key) else { return NewWindowResponse::Deny };
                // SAFETY: parked windows are boxed by their Surface and unparked before it drops (see Parked).
                match handlers(super::card_builder(app::linked_builder(WebViewBuilder::new(), &req)), card_proxy.clone()).build(unsafe { &*win }) {
                    Ok(wv) => {
                        let r = app::linked_response(&wv);
                        *slot.borrow_mut() = Some(wv);
                        r
                    }
                    Err(e) => {
                        util::log(&format!("linked card {key}: {e}"));
                        NewWindowResponse::Deny
                    }
                }
            })
            .with_url(app::ROOT_URL)
            .build(&window)
            .map_err(|e| e.to_string())?;
        app::allow_script_windows(&webview);
        Ok(Root { parked, ready: false, queued: vec![], webview, _window: window })
    }

    /// Park a card's window and ask the root page to open it (now, or once the page has loaded).
    pub fn open(&mut self, key: u64, window: &Window, slot: Slot) {
        self.parked.borrow_mut().insert(key, (window as *const Window, slot));
        if self.ready {
            self.request(key);
        } else {
            self.queued.push(key);
        }
    }

    pub fn set_ready(&mut self) {
        self.ready = true;
        for k in std::mem::take(&mut self.queued) {
            self.request(k);
        }
    }

    /// The card is going away: never build into its window.
    pub fn forget(&mut self, key: u64) {
        self.parked.borrow_mut().remove(&key);
        self.queued.retain(|k| *k != key);
    }

    /// Run a script in the root page (the shared store lives there).
    pub fn eval(&self, js: &str) {
        let _ = self.webview.evaluate_script(js);
    }

    fn request(&self, key: u64) {
        let _ = self.webview.evaluate_script(&format!("window.morcOpen && window.morcOpen({key})"));
    }
}

/// The ui/ files and the keyed ipc, for the root and for every card it opens (macOS shares the root's, so wry skips them there).
fn handlers(wvb: WebViewBuilder<'_>, proxy: Proxy) -> WebViewBuilder<'_> {
    wvb.with_custom_protocol("morc".into(), |_id, req| assets::handle(req)).with_ipc_handler(move |req| {
        let body = req.body();
        if body == "root:ready" {
            let _ = proxy.send_event(Ev::RootReady);
        } else if let Some((k, msg)) = body.strip_prefix('k').and_then(|r| r.split_once('\n')) {
            if let Ok(k) = k.parse() {
                let _ = proxy.send_event(Ev::Ipc(k, msg.to_string()));
            }
        }
    })
}

fn key_of(url: &str) -> Option<u64> {
    url.split("k=").nth(1)?.split(['&', '#']).next()?.parse().ok()
}
