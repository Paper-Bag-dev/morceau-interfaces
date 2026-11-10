//! Crowded-screen behaviour: while the pointer is on one card, the others yield room (shrink) and come back after. Split out of host/mod.rs.

use super::*;

impl Host {
    /// The page told us its content size and the shapes to keep clickable.
    /// Crowded screen (more than `crowd_max` visible cards): while the pointer is on one card, every OTHER card shrinks to `crowd_yield` of its size
    /// so the one in use has room (it grows itself on hover); when the pointer leaves, all go back. Thresholds are in prefs/placement.json.
    pub(super) fn crowd(&mut self, key: u64, on: bool) {
        static YIELDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        use std::sync::atomic::Ordering::Relaxed;
        let pol = policy::load();
        // off (the default since 11-10: the user found the others shrinking while one is in use too much): touch nothing at all, so no
        // card re-lays itself out because another one is hovered
        if pol.crowd_yield >= 0.999 {
            return;
        }
        let prefs = prefs::load();
        let visible: Vec<u64> = self.surfaces.iter().filter(|(_, s)| s.phase == Phase::Visible).map(|(k, _)| *k).collect();
        // "crowded" is not a count: it is how much of the free room (the work area minus the active window) our cards already cover, compared with the
        // learned `crowd` limit in the annoyance vector (`morc annoy nudge crowd 0.8 --why "too crowded"`)
        let fg = placement::foreground();
        let usable = ((fg.work.w() as i64 * fg.work.h() as i64) - fg.rect.map(|r| r.w() as i64 * r.h() as i64).unwrap_or(0)).max(1);
        let used: i64 = visible.iter().filter_map(|k| self.surfaces.get(k)).filter_map(|s| s.rect()).map(|r| r.w() as i64 * r.h() as i64).sum();
        let topic = self.surfaces.get(&key).map(|s| s.topic.clone()).unwrap_or_default();
        let crowded = (used as f64 / usable as f64) > prefs::get(&prefs, &topic, "crowd");
        if on && !crowded {
            return;
        }
        if !on && !YIELDED.load(Relaxed) {
            return; // nothing was shrunk, nothing to restore
        }
        for k in visible {
            if let Some(s) = self.surfaces.get(&k) {
                let base = view_for(s, &s.opts, &prefs, &s.topic, &[])["scale"].as_f64().unwrap_or(1.0);
                let f = if on && k != key { pol.crowd_yield } else { 1.0 };
                s.send(&json!({"t": "view", "view": {"scale": base * f}}));
            }
        }
        YIELDED.store(on, Relaxed);
    }
}

