//! Placement policy as DATA, not code: weights, thresholds and the shape of every anchor live in `prefs/placement.json`.
//! The host re-reads the file whenever it changes (mtime), so tuning placement, or inventing a new anchor, needs no rebuild:
//! edit the JSON, push the next surface. A missing file is written out with the defaults below; a missing key uses its default.
//!
//! An anchor is a list of terms. Each term pulls one edge of the candidate toward a target, with a weight:
//!   {"edge": "right", "to": "work.right", "w": 3, "off": 0}      cost += |cand.right - (work.right + off)| * w
//!   edge:   left right top bottom cx cy   (or "gap" with to "fg": the distance to the active window)
//!   to:     work.<edge> | fg.<edge> | work.y:0.35 | work.x:0.5  (fraction of the work area), fg = the active window (term skipped if none)
//! An anchor may name a fallback: {"anchor": "right_bottom", "when": "strip_below_lt_h"}  (blanket | no_fg | strip_below_lt_h | room_right_lt_w).

use crate::paths;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Mutex;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Term {
    pub edge: String,
    pub to: String,
    pub w: f64,
    #[serde(default)]
    pub off: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Fallback {
    pub anchor: String,
    pub when: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Anchor {
    #[serde(default)]
    pub terms: Vec<Term>,
    #[serde(default)]
    pub fallback: Option<Fallback>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Weights {
    /// cost per px^2 of covering the active window (normal / important surfaces)
    pub cover_normal: f64,
    pub cover_important: f64,
    /// cost per px^2 inside the bottom-right toast zone (normal / when the surface asks for a bottom anchor)
    pub toast: f64,
    pub toast_explicit: f64,
    /// reward for sitting flush against a neighbour window / one of my own surfaces (max cost units)
    pub dock_flush: f64,
    pub harmony_flush: f64,
    /// harmony is scaled by this when the surface names an explicit bottom/corner anchor
    pub explicit_harmony: f64,
    /// cost per px^2 of overlapping my own other surfaces
    pub mine_overlap: f64,
    /// reward per px of separation from surfaces of a different context (capped at 900 px)
    pub foreign_sep: f64,
    /// how much a full dock setting softens the anchor pull (0..1)
    pub anchor_soften: f64,
}

impl Default for Weights {
    fn default() -> Self {
        Weights { cover_normal: 400.0, cover_important: 4.0, toast: 6.0, toast_explicit: 0.3, dock_flush: 2000.0, harmony_flush: 1600.0, explicit_harmony: 0.15, mine_overlap: 5000.0, foreign_sep: 1.5, anchor_soften: 0.55 }
    }
}

/// When the host may move a surface by itself (it never hops: see host.rs replace()).
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AutoMove {
    /// minimum fraction of the surface that must be covered
    pub min_cover: f64,
    /// move only if the shift is between min_dist and max_dist of the work area
    pub min_dist: f64,
    pub max_dist: f64,
    pub cooldown_s: f64,
}

impl Default for AutoMove {
    fn default() -> Self {
        AutoMove { min_cover: 0.25, min_dist: 0.04, max_dist: 0.25, cooldown_s: 10.0 }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Policy {
    pub weights: Weights,
    pub auto_move: AutoMove,
    /// distance kept from the work-area edge, the seam to a neighbour, the candidate grid step (physical px)
    pub margin: i32,
    pub gap: i32,
    pub step: i32,
    /// bottom-right zone where Windows toasts appear: [width, height] physical px
    pub toast_zone: [i32; 2],
    /// More than this many cards on screen counts as crowded: while the user is on one card, the OTHERS shrink to `crowd_yield` of their size
    /// (never below the page's zoom floor) and come back when the pointer leaves. 0 turns it off.
    pub crowd_max: usize,
    pub crowd_yield: f64,
    /// anchors that mean "I chose where": they override sibling pull and toast avoidance
    pub explicit_anchors: Vec<String>,
    pub anchors: BTreeMap<String, Anchor>,
}

const DEFAULT_ANCHORS: &str = r#"{
  "right":        {"terms": [{"edge":"right","to":"work.right","w":3}, {"edge":"cy","to":"work.y:0.35","w":0.6}]},
  "near_focus":   {"terms": [{"edge":"gap","to":"fg","w":4}], "fallback": {"anchor":"right","when":"blanket"}},
  "corner":       {"terms": [{"edge":"right","to":"work.right","w":2}, {"edge":"bottom","to":"work.bottom","w":2}]},
  "center":       {"terms": [{"edge":"cx","to":"work.cx","w":2}, {"edge":"cy","to":"work.cy","w":2}]},
  "bottom":       {"terms": [{"edge":"bottom","to":"work.bottom","w":3}, {"edge":"cx","to":"work.cx","w":0.5}]},
  "side":         {"terms": [{"edge":"right","to":"work.right","w":3}, {"edge":"cy","to":"work.cy","w":0.5}]},
  "left_bottom":  {"terms": [{"edge":"left","to":"work.left","w":3}, {"edge":"bottom","to":"work.bottom","w":1.5}]},
  "left_top":     {"terms": [{"edge":"left","to":"work.left","w":3}, {"edge":"top","to":"work.top","w":1.5}]},
  "right_top":    {"terms": [{"edge":"right","to":"work.right","w":3}, {"edge":"top","to":"work.top","w":1.5}]},
  "right_bottom": {"terms": [{"edge":"right","to":"work.right","w":3}, {"edge":"bottom","to":"work.bottom","w":1.5}]},
  "below":        {"terms": [{"edge":"top","to":"fg.bottom","w":3,"off":6}, {"edge":"right","to":"fg.right","w":1.2}], "fallback": {"anchor":"right_bottom","when":"strip_below_lt_h"}},
  "beside":       {"terms": [{"edge":"left","to":"fg.right","w":3,"off":6}, {"edge":"top","to":"fg.top","w":1.2}], "fallback": {"anchor":"right","when":"room_right_lt_w"}}
}"#;

impl Default for Policy {
    fn default() -> Self {
        Policy {
            weights: Weights::default(),
            auto_move: AutoMove::default(),
            margin: 14,
            gap: 6,
            step: 20,
            toast_zone: [420, 380],
            crowd_max: 5, // the overload test: past about five cards the screen reads as too much
            crowd_yield: 0.8,
            explicit_anchors: ["right_bottom", "left_bottom", "bottom", "corner", "below"].iter().map(|s| s.to_string()).collect(),
            anchors: serde_json::from_str(DEFAULT_ANCHORS).unwrap_or_default(),
        }
    }
}

static CACHE: Mutex<Option<(SystemTime, Policy)>> = Mutex::new(None);

fn file() -> std::path::PathBuf {
    let d = paths::prefs_dir();
    let _ = fs::create_dir_all(&d);
    d.join("placement.json")
}

/// The current policy (re-read when the file changes). A broken file falls back to the defaults and says so in host.log.
pub fn load() -> Policy {
    let path = file();
    let m = fs::metadata(&path).and_then(|m| m.modified()).ok();
    let mut g = CACHE.lock().unwrap();
    if let (Some(m), Some((t, p))) = (m, g.as_ref()) {
        if *t == m {
            return p.clone();
        }
    }
    let p = match fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str::<Policy>(s.trim_start_matches('\u{feff}')) {
            Ok(mut p) => {
                // keep the built-in anchors the file does not override, so an old file never loses a new default
                for (k, v) in Policy::default().anchors {
                    p.anchors.entry(k).or_insert(v);
                }
                p
            }
            Err(e) => {
                crate::util::log(&format!("placement.json is invalid ({e}); using defaults"));
                Policy::default()
            }
        },
        Err(_) => {
            let d = Policy::default();
            if let Ok(s) = serde_json::to_string_pretty(&d) {
                let _ = fs::write(&path, s);
            }
            d
        }
    };
    if let Some(m) = fs::metadata(&path).and_then(|m| m.modified()).ok() {
        *g = Some((m, p.clone()));
    }
    p
}
