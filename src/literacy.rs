//! What the user has learned to read, per chart kind: {seen, got, confused}.
//! "Got it" stops the explainer. "Still confusing" is friction: the page swaps in a plain chart, and the agent is told
//! (inbox event kind "friction") so it explains differently or stops using that kind. Engagement stats stay separate.

use crate::{inbox, paths};
use serde_json::{json, Value};
use std::fs;

fn file() -> std::path::PathBuf {
    paths::runtime_dir().join("literacy.json")
}

pub fn snapshot() -> Value {
    fs::read_to_string(file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(json!({}))
}

pub fn record(surface: &str, topic: &str, kind: &str, v: &str) {
    if kind.is_empty() || !matches!(v, "seen" | "got" | "confused") {
        return;
    }
    let mut m = snapshot();
    let n = m[kind][v].as_u64().unwrap_or(0) + 1;
    m[kind][v] = json!(n);
    let _ = fs::write(file(), m.to_string());
    if v == "confused" {
        inbox::push(surface, topic, "friction", &json!({"chart_kind": kind, "note": "user tapped 'Still confusing'; a plain chart was shown instead. Re-explain differently or avoid this kind."}));
    }
}
