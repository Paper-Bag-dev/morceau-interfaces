//! How a card is drawn and placed: its scale and size limits (view_for), the cohesion weights, what counts as important, the topic scope.
//! (Split out of host/mod.rs: the pure decisions about one card; the event loop and the windows stay in mod.rs.)

use super::*;

/// Important or urgent surfaces may cover the active window; everything else keeps off it (unless nothing fits anywhere).
pub(super) fn is_important(opts: &Opts) -> bool {
    opts.urgent || match opts.importance.as_deref() {
        Some("high") | Some("critical") => true,
        Some(x) => x.parse::<f64>().map(|v| v >= 1.2).unwrap_or(false),
        None => false,
    }
}

pub(super) fn cohesion_for(s: &Surface) -> placement::Cohesion {
    let p = prefs::load();
    let pol = policy::load();
    // An explicit bottom/corner anchor is an intent: it overrides the pull toward sibling surfaces and the toast-zone avoidance,
    // so a long list can really use the space below instead of tucking under its siblings. Which anchors count is policy data.
    let explicit = s.opts.anchor.as_deref().map(|a| pol.explicit_anchors.iter().any(|e| e == a)).unwrap_or(false);
    placement::Cohesion {
        dock: prefs::get(&p, &s.topic, "dock"),
        harmony: prefs::get(&p, &s.topic, "harmony") * if explicit { pol.weights.explicit_harmony } else { 1.0 },
        cover: if is_important(&s.opts) { pol.weights.cover_important } else { pol.weights.cover_normal },
        toast: if explicit { pol.weights.toast_explicit } else { pol.weights.toast },
    }
}

/// A context scope is the first segment of the topic ("stats.glance" -> "stats"): related surfaces share a scope.
pub(super) fn scope_of(topic: &str) -> &str {
    topic.split('.').next().unwrap_or(topic)
}

/// How this surface should be drawn: scale (importance x size_bias), the 1/4-screen cap, motion, boldness, simplicity.
pub(super) fn view_for(s: &Surface, opts: &Opts, prefs: &prefs::Prefs, topic: &str, widgets: &[placement::R]) -> Value {
    let imp = match opts.importance.as_deref() {
        Some("low") => 0.9,
        Some("high") => 1.2,
        Some("critical") => 1.45,
        Some(x) => x.parse::<f64>().unwrap_or(1.0),
        None => 1.0,
    };
    let sf = s.window.scale_factor();
    let work = placement::foreground().work;
    // Width the page should WRAP to (css px, unzoomed) so the window fits the gap beside the neighbour instead of forcing a slab.
    // Skipped for important/urgent surfaces and for working windows (graphs/3D ask for max_frac > 0.5).
    // Size to the region the tier ladder will pick (every card but explicit bottom/corner anchors, see layout.rs), not to the gap beside the active window.
    let explicit_anchor = opts.anchor.as_deref().map(|a| policy::load().explicit_anchors.iter().any(|e| e == a)).unwrap_or(false);
    let reg = (opts.region.is_some() || !explicit_anchor)
        .then(|| regions::size_for(opts.region.as_deref(), prefs::get(prefs, topic, "overlap") >= 0.2 || is_important(opts), opts.min.map(|[w, h]| ((w as f64 * sf) as i32, (h as f64 * sf) as i32)), widgets))
        .flatten();
    let room = reg.map(|(w, _)| w as f64).unwrap_or(placement::dock_room() as f64) / sf;
    let strip_h = reg.map(|(_, h)| h as f64).unwrap_or(placement::dock_below() as f64);
    let big = opts.max_frac.unwrap_or(0.5) > 0.5;
    let wrap = if prefs::get(prefs, topic, "morph") >= 0.3 && !is_important(opts) && !big {
        // leave the policy margin + gap too, or content that wraps to exactly the region is a hair too wide for it and falls back to scored placement
        let pol = policy::load();
        let slack = 14.0 + (pol.margin + pol.gap) as f64 / sf;
        if room >= 230.0 { (room - slack).clamp(230.0, 420.0) } else { 360.0 }   // no gap beside a neighbour: still wrap long text instead of forcing a slab
    } else {
        0.0
    };
    // never wider than the free region the card is aimed at (less the margin and gap): the page zooms DOWN to fit it, but not below its zoom floor
    // (MIN_ZOOM), so a card compromises on size before it overlaps the active window or leaves its region
    let maxw = {
        let cap = work.w() as f64 / sf * opts.max_frac.unwrap_or(0.5).clamp(0.3, 0.92);
        match reg {
            Some((rw, _)) => {
                let pol = policy::load(); // margin and gap come from prefs/placement.json, not a number in the code
                cap.min((rw as f64 - (pol.margin + pol.gap) as f64) / sf).max(200.0)
            }
            None => cap,
        }
    };
    json!({
        "scale": (imp * prefs::get(prefs, topic, "size_bias")).clamp(0.5, 2.2),
        "fill": opts.fill,
        // default cap: 1/4 of the usable area = half the width x half the height; working windows (graphs) ask for more via max_frac
        // never wider than the free region the card is aimed at (less the margin and gap): the page zooms DOWN to fit it, but not below its zoom floor
        // (MIN_ZOOM), so a card compromises on size before it ever overlaps the active window or leaves its region
        "maxw": maxw,
        "ui": ui_bundle(),
        "maxh": work.h() as f64 / sf * opts.max_frac.unwrap_or(0.5).clamp(0.3, 0.92),
        "theme_default": prefs.theme.clone().unwrap_or_else(|| "coral".into()),
        "debug": std::env::var("MORC_DEBUG").is_ok(),
        "motion": prefs::get(prefs, topic, "motion"),
        "boldness": prefs::get(prefs, topic, "boldness"),
        "simplicity": prefs::get(prefs, topic, "simplicity"),
        // Width the page should WRAP to (css px, unzoomed) so the window fits the gap beside the neighbour instead of forcing a slab.
        // Skipped for important/urgent surfaces and for working windows (graphs/3D ask for max_frac > 0.5).
        "wrap": wrap,
        "dock_h": ((strip_h - 22.0) / sf).max(0.0),   // css px of height available (the named region, else the strip under the active window): a long list flows into it
        "dock": prefs::get(prefs, topic, "dock"),
        "morph": prefs::get(prefs, topic, "morph"),
        "harmony": prefs::get(prefs, topic, "harmony"),
        "dock_w": placement::dock_room() as f64 / sf,   // css px free beside the active window: the width a docked surface may use
    })
}

