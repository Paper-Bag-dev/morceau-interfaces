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
        for k in ended {
            let Some(s) = self.surfaces.get_mut(&k) else { continue };
            s.drag_started = None;
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
    }

    pub(super) fn tick(&mut self, target: &EventLoopWindowTarget<Ev>) {
        self.finish_drags();
        let now = Instant::now();
        // glide
        for s in self.surfaces.values_mut() {
            if let Some(m) = &s.mv {
                let t = (m.start.elapsed().as_secs_f64() / MOVE_SECS).min(1.0);
                let e = 1.0 - (1.0 - t).powi(3); // ease-out cubic
                let x = m.from.0 as f64 + (m.to.0 - m.from.0) as f64 * e;
                let y = m.from.1 as f64 + (m.to.1 - m.from.1) as f64 * e;
                s.window.set_outer_position(PhysicalPosition::new(x as i32, y as i32));
                if t >= 1.0 {
                    s.mv = None;
                }
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

