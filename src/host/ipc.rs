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
                    window_style::set_glass(s.hwnd(), &super::layout::px_rects(&m["glass"], s.window.scale_factor()));
                }
            }
            "hover" => {
                let on = m["v"].as_bool().unwrap_or(false);
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.on_hover(on, now);
                }
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
                    window_style::drag_cursor_on(); // pointer becomes the move cursor for the whole carry
                    // The native move loop runs inside the message pump AFTER this returns, so the end is detected in tick()
                    // (mouse button released) rather than here.
                    let _ = s.window.drag_window();
                }
            }
            "kbd" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    let on = m["v"].as_bool().unwrap_or(false);
                    window_style::set_noactivate(s.hwnd(), !on);
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
            "log" => util::log(&format!("page[{key}]: {}", m["msg"])),
            _ => {}
        }
    }
}

