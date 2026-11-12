//! The host's clock: lifetimes, parked cards, the carry (drag) bookkeeping and the placement nudges each tick. Split out of host/mod.rs.

use super::*;

impl Host {
    /// onEnd: the carry finished (mouse button released). Reports where it landed and remembers the spot for the topic.
    pub(super) fn finish_drags(&mut self) {
        let ended: Vec<u64> = self
            .surfaces
            .values()
            .filter(|s| s.drag_started.map(|t| t.elapsed() > Duration::from_millis(150)).unwrap_or(false))
            .filter(|_| !left_button_down())
            .map(|s| s.key)
            .collect();
        if !ended.is_empty() {
            winstyle::drag_cursor_off(); // onDragEnd: the user's own pointer comes back
        }
        for k in ended.iter().copied() {
            let Some(s) = self.surfaces.get_mut(&k) else { continue };
            s.drag_started = None;
            s.grab = None;
            s.life.dragging = false;
            s.user_moved = true;
            s.life.on_drag_end(Instant::now());
            s.send(&json!({"t": "dropped"}));
            if let Some(to) = s.rect() {
                let work = placement::work_at((to.l + to.r) / 2, (to.t + to.b) / 2);
                let moved = s.drag_from.map(|f| (f.l, f.t) != (to.l, to.t)).unwrap_or(true);
                inbox::push(&s.id, &s.topic, "drag_end", &json!({"from": s.drag_from.as_ref().map(placed::rect_json), "to": placed::rect_json(&to), "moved": moved, "screen_zone": placed::describe(&to, &work), "remembered_for_topic": moved && !s.topic.is_empty()}));
                if moved {
                    placed::set(&s.topic, to.l, to.t);
                }
            }
            s.drag_from = None;
        }
        // the record learns the new spot at its next write (falloff in cards.rs), once per drop, never per frame
        for k in ended {
            if let Some(id) = self.surfaces.get(&k).map(|s| s.id.clone()) {
                self.note_card(&id);
            }
        }
    }

    /// Magnetic carry (snap.rs): put each carried card where the pointer wants it, pulled onto the nearest edges.
    fn follow_grabs(&mut self) {
        let keys: Vec<u64> = self.surfaces.values().filter(|s| s.grab.is_some()).map(|s| s.key).collect();
        if keys.is_empty() {
            return;
        }
        let pol = policy::load();
        let (cx, cy) = crate::platform::os::sys::cursor();
        let fg = placement::foreground();
        for k in keys {
            // targets: the active window and my other visible cards (their visible content)
            let mut targets: Vec<placement::R> = fg.rect.into_iter().collect();
            targets.extend(self.surfaces.values().filter(|o| o.key != k && o.phase == Phase::Visible).filter_map(|o| o.content_rect()));
            let Some(s) = self.surfaces.get_mut(&k) else { continue };
            let (Some(r), Some(c)) = (s.rect(), s.content_rect()) else { continue };
            // css px settings -> physical px on THIS card's display (mixed-DPI setups differ per display)
            let scale = s.window.scale_factor();
            let px = |name: &str, d: f64| (super::snap_setting(name, d) * scale).round() as i32;
            let (radius, breakaway) = (px("snap-px", 0.0), px("snap-break-px", 44.0));
            let Some(g) = s.grab.as_mut() else { continue };
            let (il, it) = (c.l - r.l, c.t - r.t);
            let (wx, wy) = (cx - g.offset.0, cy - g.offset.1);   // where the window would go without magnets
            let raw = placement::R { l: wx + il, t: wy + it, r: wx + il + c.w(), b: wy + it + c.h() };
            let work = placement::work_at((raw.l + raw.r) / 2, (raw.t + raw.b) / 2);
            // the seam a snapped card keeps from what it snapped to: its own setting (snap-gap), not the placement gap
            let seam = px("snap-gap", 12.0);
            let m = super::snap::Magnets { targets: &targets, work, radius, breakaway, gap: seam, margin: pol.margin };
            let fast = g.track((cx, cy)) > super::snap_setting("snap-max-speed", 900.0) * scale;
            let ((x, y), st) = super::snap::apply_moving(&raw, &m, g.sticky, fast);
            g.sticky = st;
            if (x - il, y - it) != (r.l, r.t) {
                s.window.set_outer_position(PhysicalPosition::new(x - il, y - it));
            }
        }
    }

    pub(super) fn tick(&mut self, target: &EventLoopWindowTarget<Ev>) {
        self.tree_timeouts();
        self.flush_cards(false);
        state::flush_history(false);
        self.finish_drags();
        let now = Instant::now();
        self.follow_grabs();
        // glide
        let mut moved: Vec<String> = vec![];
        for s in self.surfaces.values_mut() {
            if let Some(m) = &s.mv {
                let t = (m.start.elapsed().as_secs_f64() / MOVE_SECS).min(1.0);
                let e = 1.0 - (1.0 - t).powi(3); // ease-out cubic
                let x = m.from.0 as f64 + (m.to.0 - m.from.0) as f64 * e;
                let y = m.from.1 as f64 + (m.to.1 - m.from.1) as f64 * e;
                s.window.set_outer_position(PhysicalPosition::new(x as i32, y as i32));
                if t >= 1.0 {
                    s.mv = None;
                    moved.push(s.id.clone());
                }
            }
        }
        for id in moved {
            self.note_card(&id);
        }
        // away: no input for a while stops every card's clock, so nothing expires unseen (and nothing is logged as ignored)
        let dt = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        if crate::worker::idle_seconds() >= crate::lifetime::AWAY_AFTER_S {
            for s in self.surfaces.values_mut().filter(|s| s.phase == Phase::Visible) {
                s.life.hold(dt);
            }
        }
        // expiry (quiet, silent)
        let due: Vec<(u64, Outcome)> = self.surfaces.values().filter(|s| s.phase == Phase::Visible).filter_map(|s| s.life.due(now).map(|o| (s.key, o))).collect();
        for (k, o) in due {
            self.begin_leave(k, Some(o));
        }
        // stuck exits / broken pages
        let dead: Vec<u64> = self
            .surfaces
            .values()
            .filter(|s| (s.phase == Phase::Leaving && s.phase_since.elapsed() > Duration::from_millis(700)) || (s.phase == Phase::Waiting && s.phase_since.elapsed() > Duration::from_secs(25)))
            .map(|s| s.key)
            .collect();
        for k in dead {
            self.destroy(k);
        }
        // warm spare: keep one while the host is in use, drop it when idle
        let active = self.surfaces.values().any(|s| s.phase != Phase::Spare);
        let has_spare = self.surfaces.values().any(|s| s.phase == Phase::Spare);
        let recent = self.last_use.elapsed() < SPARE_KEEP;
        if !has_spare && (active || recent || self.surfaces.is_empty() && self.next_key == 1) {
            let _ = self.spawn(target);
        } else if has_spare && !active && !recent {
            let k = self.surfaces.values().find(|s| s.phase == Phase::Spare).map(|s| s.key);
            if let Some(k) = k {
                self.destroy(k);
            }
        }
    }
}

