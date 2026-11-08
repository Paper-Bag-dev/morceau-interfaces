//! Where the user likes things. When they carry a window somewhere (long-press the pin, or hold anywhere), that spot is
//! remembered per topic and used the next time that topic appears, unless it would land on top of the active window.
//! Also tells the agent where it ended up (inbox events drag_start / drag_end) so it can adapt.

use crate::{paths, placement::R};
use serde_json::{json, Value};
use std::fs;

fn file() -> std::path::PathBuf {
    paths::runtime_dir().join("placed.json")
}

fn load() -> Value {
    fs::read_to_string(file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(json!({}))
}

pub fn set(topic: &str, x: i32, y: i32) {
    if topic.is_empty() {
        return;
    }
    let mut m = load();
    m[topic] = json!([x, y]);
    let _ = fs::write(file(), m.to_string());
}

pub fn get(topic: &str) -> Option<(i32, i32)> {
    let m = load();
    let a = m.get(topic)?.as_array()?;
    Some((a.first()?.as_i64()? as i32, a.get(1)?.as_i64()? as i32))
}

/// "bottom-right", "top-left", "middle-center"... for where a rect sits inside its monitor's work area.
pub fn describe(r: &R, work: &R) -> String {
    let cx = (r.l + r.r) / 2 - work.l;
    let cy = (r.t + r.b) / 2 - work.t;
    let h = if cx < work.w() / 3 { "left" } else if cx > work.w() * 2 / 3 { "right" } else { "center" };
    let v = if cy < work.h() / 3 { "top" } else if cy > work.h() * 2 / 3 { "bottom" } else { "middle" };
    format!("{v}-{h}")
}

pub fn rect_json(r: &R) -> Value {
    json!({"x": r.l, "y": r.t, "w": r.w(), "h": r.h()})
}
