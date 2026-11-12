//! Messages from the card pages (hover, drag, act, layout, pin, close ...) turned into host actions. Split out of host/mod.rs.

use super::*;

impl Host {
    pub(super) fn ipc(&mut self, key: u64, text: &str) {
        let Ok(m) = serde_json::from_str::<Value>(text) else { return };
        let now = Instant::now();
        match m["t"].as_str().unwrap_or("") {
            "ready" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.ready = true;
                    if let Some(init) = s.init.take() {
                        s.send(&init);
                    }
                }
            }
            "layout" => self.layout(key, &m),
            // the page streams the plate's rect during a grow/shrink, so the native glass follows the spring frame by frame
            "glass" => {
                if let Some(s) = self.surfaces.get(&key) {
                    winstyle::set_glass(s.hwnd(), &super::layout::px_rects(&m["glass"], s.window.scale_factor()));
                }
            }
            "hover" => {
                let on = m["v"].as_bool().unwrap_or(false);
                // Up / Down belong to the card only while the pointer is on it (registered here, released when it leaves)
                let was = self.hovered.is_some();
                if on {
                    self.hovered = Some(key);
                } else if self.hovered == Some(key) {
                    self.hovered = None;
                }
                if was != self.hovered.is_some() {
                    crate::hotkeys::arrows(self.hovered.is_some());
                }
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.on_hover(on, now);
                }
                self.crowd(key, on);
            }
            "pin" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.pinned = m["v"].as_bool().unwrap_or(false);
                    s.life.on_interact(now);
                }
            }
            "drag" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    if s.drag_started.is_some() {
                        return;
                    }
                    s.life.dragging = true;
                    s.mv = None;
                    let from = s.rect();
                    s.drag_from = from;
                    s.drag_started = Some(Instant::now());
                    // onStart: tell the agent a carry began ...
                    inbox::push(&s.id, &s.topic, "drag_start", &json!({"from": from.as_ref().map(placed::rect_json), "via": m["src"].as_str().unwrap_or("hold")}));
                    winstyle::drag_cursor_on(); // pointer becomes the move cursor for the whole carry
                    if super::snap_setting("snap-px", 0.0) > 0.0 {
                        // magnetic carry (snap.rs): the host follows the pointer itself, so it can pull the card onto edges mid-way
                        if let Some(r) = s.rect() {
                            let (cx, cy) = crate::platform::os::sys::cursor();
                            s.grab = Some(super::snap::Grab::new((cx - r.l, cy - r.t)));
                        }
                    } else {
                        // The native move loop runs inside the message pump AFTER this returns, so the end is detected in tick()
                        // (mouse button released) rather than here.
                        let _ = s.window.drag_window();
                    }
                }
            }
            "kbd" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    let on = m["v"].as_bool().unwrap_or(false);
                    winstyle::set_noactivate(s.hwnd(), !on);
                    if on {
                        s.window.set_focus();
                    }
                    s.life.on_interact(now);
                }
            }
            "act" => {
                let (mut patch, mut wake_payload) = (None, None);
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.on_interact(now);
                    match handlers::run(&s.id, &s.topic, &m["action"], &self.proxy) {
                        handlers::Effect::Patch(path, v) => patch = Some((path, v)),
                        handlers::Effect::Wake(p) => wake_payload = Some(p),
                        handlers::Effect::None => {}
                    }
                    if let Some((path, v)) = patch {
                        s.send(&json!({"t": "patch", "data": {path: v}}));
                    }
                }
                if let Some(p) = wake_payload {
                    let r = wake::maybe_wake(&p);
                    util::log(&format!("wake: {r}"));
                    // a click that asked for background work but got nobody (agent off, rate limited): put the button back instead of leaving skeletons forever
                    if p["bg"].as_bool().unwrap_or(false) && !matches!(r, "headless-spawned" | "queued" | "dry-run") {
                        if let Some(s) = self.surfaces.get(&key) {
                            // the spec says what "back to the button" means: payload.reset = {"/mode": "cold"}; default {"/state": "idle"}
                            let reset = p["reset"].clone();
                            let data = if reset.is_object() { reset } else { json!({"/state": "idle"}) };
                            s.send(&json!({"t": "patch", "data": data}));
                        }
                    }
                }
            }
            "close" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.engaged = true;
                }
                self.begin_leave(key, Some(Outcome::Engaged));
            }
            "literacy" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.on_interact(now);
                    literacy::record(&s.id, &s.topic, m["kind"].as_str().unwrap_or(""), m["v"].as_str().unwrap_or(""));
                }
            }
            "dismiss" => self.begin_leave(key, Some(Outcome::Dismissed)),
            "left" => self.destroy(key),
            "reply" => self.reply_part(key, &m),
            // the layer button: behind the active window / back on top (cards.rs keeps the state)
            "layer" => {
                let d = self.surfaces.get(&key).map(|s| s.depth).unwrap_or(0);
                self.set_depth(key, if d == 0 { 1 } else { 0 });   // one level down, and back up
            }
            // an unlinked card wrote to the shared store (@/path): keep it and relay it (cards.rs)
            "shared" => {
                if let Some(p) = m["path"].as_str() {
                    self.shared_set(Some(key), p, m["value"].clone());
                }
            }
            // the page's data as it leaves: what `morc card <id> show` brings back
            "final" => self.note_final(key, &m["data"]),
            "did" => self.did(key, &m),
            "snap" => self.snap(key, &m),
            "log" => util::log(&format!("page[{key}]: {}", m["msg"])),
            _ => {}
        }
    }
}

