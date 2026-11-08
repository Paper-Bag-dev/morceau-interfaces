//! Behaviour log: what happened to each surface (engaged / glanced / ignored / dismissed / stale).
//! "Where I don't click it means I wasn't interested": ignored topics back off, engaged topics keep their rhythm.
//! Other tools may read this file (for example for a daily summary). It never changes anything else.

use crate::{paths, util};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;

pub fn record(topic: &str, name: &str, auto: bool, outcome: &str, dwell_s: f64) {
    let line = json!({"ts": util::now_ms() as u64, "topic": topic, "name": name, "auto": auto, "outcome": outcome, "dwell_s": (dwell_s * 10.0).round() / 10.0});
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(paths::engagement_file()) {
        let _ = writeln!(f, "{}", line);
    }
}

fn weight(outcome: &str) -> Option<f64> {
    match outcome {
        "engaged" => Some(1.0),
        "glanced" => Some(0.3),
        "ignored" | "stale" => Some(0.0),
        "dismissed" => Some(0.0),
        _ => None, // closed_by_agent etc. say nothing about interest
    }
}

/// Interest in a topic, 0..1, from its last records (exponential weighting, prior 0.5).
pub fn score(topic: &str) -> f64 {
    let Ok(text) = fs::read_to_string(paths::engagement_file()) else { return 0.5 };
    let recent: Vec<Value> = text
        .lines()
        .rev()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["topic"] == topic)
        .take(12)
        .collect();
    let (mut num, mut den) = (0.5 * 1.5, 1.5); // prior: 1.5 pseudo-observations at 0.5
    for (i, v) in recent.iter().enumerate() {
        if let Some(w) = v["outcome"].as_str().and_then(weight) {
            let decay = 0.85_f64.powi(i as i32);
            num += w * decay;
            den += decay;
        }
    }
    num / den
}

/// The current run of identical outcomes for a topic after `since_ms`: (outcome, length). "glanced"/"engaged" end a run.
pub fn streak(topic: &str, since_ms: u64) -> (String, usize) {
    let Ok(text) = fs::read_to_string(paths::engagement_file()) else { return (String::new(), 0) };
    let mut kind = String::new();
    let mut n = 0;
    for v in text.lines().rev().filter_map(|l| serde_json::from_str::<Value>(l).ok()).filter(|v| v["topic"] == topic) {
        if v["ts"].as_u64().unwrap_or(0) <= since_ms { break; }
        let o = v["outcome"].as_str().unwrap_or("");
        if weight(o).is_none() { continue; }
        if kind.is_empty() { kind = o.to_string(); }
        if o == kind { n += 1 } else { break }
    }
    (kind, n)
}

/// How much to stretch a trigger's cooldown for this topic: 1x when loved, up to 6x when ignored.
pub fn cooldown_factor(topic: &str) -> f64 {
    1.0 + 5.0 * (1.0 - score(topic)).powi(2)
}
