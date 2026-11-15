//! When does a surface go away? Pure logic (no windows), so it is unit-testable.
//! Default: vanish quietly after ~15s of no interaction. Hover or interaction buys time.
//! Pinned surfaces never go: the agent pins with `opts.pinned` (or `morc card <id> set {"pinned": true}`), the user with the pin button,
//! and either can unpin; an unpinned card then runs its ttl from that moment.
//! Urgent surfaces skip the timer but are parked (not lost) once untouched for 10 min.

use std::time::{Duration, Instant};

pub const DEFAULT_TTL: f64 = 15.0;
pub const AFTER_HOVER: f64 = 5.0;
pub const AFTER_INTERACT: f64 = 14.0;
pub const AFTER_DRAG: f64 = 12.0;
pub const STALE_AFTER: Duration = Duration::from_secs(600);
/// No keyboard or mouse input for this long = the user is away: every card's clock stops (see `hold`).
pub const AWAY_AFTER_S: u64 = 45;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Outcome {
    Engaged,
    Glanced,
    Ignored,
    Dismissed,
    Stale,
}

impl Outcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Outcome::Engaged => "engaged",
            Outcome::Glanced => "glanced",
            Outcome::Ignored => "ignored",
            Outcome::Dismissed => "dismissed",
            Outcome::Stale => "stale",
        }
    }
}

pub struct Life {
    pub shown: Instant,
    pub deadline: Instant,
    pub last_touch: Instant,
    pub ttl: f64,
    /// Urgent: no timer, parked (Outcome::Stale) once untouched for STALE_AFTER.
    pub urgent: bool,
    pub pinned: bool,
    pub hover: bool,
    pub dragging: bool,
    pub engaged: bool,
    pub glanced: bool,
}

impl Life {
    pub fn new(ttl: Option<f64>, urgent: bool, pinned: bool, now: Instant) -> Life {
        let ttl = ttl.unwrap_or(DEFAULT_TTL).max(2.0);
        Life { shown: now, deadline: now + Duration::from_secs_f64(ttl), last_touch: now, ttl, urgent, pinned, hover: false, dragging: false, engaged: false, glanced: false }
    }

    /// Pin or unpin (the user's button or the agent). Unpinning starts the ttl again from now, so the card does not vanish at once.
    pub fn set_pinned(&mut self, v: bool, now: Instant) {
        if self.pinned && !v {
            self.deadline = now + Duration::from_secs_f64(self.ttl.max(AFTER_INTERACT));
            self.last_touch = now;
        }
        self.pinned = v;
    }

    fn extend(&mut self, now: Instant, secs: f64) {
        let d = now + Duration::from_secs_f64(secs);
        if d > self.deadline {
            self.deadline = d;
        }
    }

    pub fn on_hover(&mut self, v: bool, now: Instant) {
        self.hover = v;
        if v {
            self.glanced = true;
        } else {
            self.extend(now, AFTER_HOVER);
        }
    }

    pub fn on_interact(&mut self, now: Instant) {
        self.engaged = true;
        self.last_touch = now;
        self.extend(now, AFTER_INTERACT.max(self.ttl));
    }

    pub fn on_drag_end(&mut self, now: Instant) {
        self.engaged = true;
        self.last_touch = now;
        self.extend(now, AFTER_DRAG);
    }

    /// A fresh show/replace on the same id restarts the clock.
    /// A pin survives a re-push (only an unpin takes it away); `pinned` here can only add one.
    pub fn restart(&mut self, ttl: Option<f64>, urgent: bool, pinned: bool, now: Instant) {
        self.ttl = ttl.unwrap_or(DEFAULT_TTL).max(2.0);
        self.urgent = urgent;
        self.pinned |= pinned;
        self.deadline = now + Duration::from_secs_f64(self.ttl);
        self.last_touch = now;
    }

    pub fn outcome(&self) -> Outcome {
        if self.engaged {
            Outcome::Engaged
        } else if self.glanced {
            Outcome::Glanced
        } else {
            Outcome::Ignored
        }
    }

    /// The user is away: stop this card's clock for `dt`. A card must never expire unseen, and an unseen expiry would be logged as
    /// "ignored" and teach the annoyance vector that the user wants fewer cards when they were simply not there.
    pub fn hold(&mut self, dt: Duration) {
        self.deadline += dt;
        self.last_touch += dt;
    }

    /// Should this surface leave now? Returns why.
    pub fn due(&self, now: Instant) -> Option<Outcome> {
        if self.pinned || self.hover || self.dragging {
            return None;
        }
        if self.urgent {
            return if now.duration_since(self.last_touch) >= STALE_AFTER { Some(Outcome::Stale) } else { None };
        }
        if now >= self.deadline {
            return Some(self.outcome());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expires_ignored() {
        let t0 = Instant::now();
        let l = Life::new(Some(10.0), false, false, t0);
        assert_eq!(l.due(t0 + Duration::from_secs(5)), None);
        assert_eq!(l.due(t0 + Duration::from_secs(11)), Some(Outcome::Ignored));
    }

    #[test]
    fn hover_pauses_then_grants_grace() {
        let t0 = Instant::now();
        let mut l = Life::new(Some(10.0), false, false, t0);
        l.on_hover(true, t0 + Duration::from_secs(8));
        assert_eq!(l.due(t0 + Duration::from_secs(30)), None);
        l.on_hover(false, t0 + Duration::from_secs(30));
        assert_eq!(l.due(t0 + Duration::from_secs(34)), None);
        assert_eq!(l.due(t0 + Duration::from_secs(36)), Some(Outcome::Glanced));
    }

    #[test]
    fn interaction_means_engaged_and_pin_holds() {
        let t0 = Instant::now();
        let mut l = Life::new(Some(10.0), false, false, t0);
        l.on_interact(t0 + Duration::from_secs(3));
        assert_eq!(l.due(t0 + Duration::from_secs(20)), Some(Outcome::Engaged));
        l.set_pinned(true, t0 + Duration::from_secs(20));
        assert_eq!(l.due(t0 + Duration::from_secs(9999)), None);
    }

    #[test]
    fn pinned_from_the_start_stays_until_unpinned() {
        let t0 = Instant::now();
        let mut l = Life::new(Some(15.0), false, true, t0);
        assert_eq!(l.due(t0 + Duration::from_secs(86_400)), None);   // a day later, still up
        l.restart(Some(15.0), false, false, t0 + Duration::from_secs(100));
        assert!(l.pinned, "a re-push without the pin keeps it");
        l.set_pinned(false, t0 + Duration::from_secs(200));
        assert_eq!(l.due(t0 + Duration::from_secs(205)), None);   // the ttl starts at the unpin
        assert!(l.due(t0 + Duration::from_secs(216)).is_some());
    }

    #[test]
    fn urgent_goes_stale_quietly() {
        let t0 = Instant::now();
        let l = Life::new(None, true, false, t0);
        assert_eq!(l.due(t0 + Duration::from_secs(300)), None);
        assert_eq!(l.due(t0 + Duration::from_secs(601)), Some(Outcome::Stale));
    }

    #[test]
    fn away_time_does_not_count() {
        let t0 = Instant::now();
        let mut l = Life::new(Some(15.0), false, false, t0);
        l.hold(Duration::from_secs(300)); // the user was away for 5 minutes
        assert_eq!(l.due(t0 + Duration::from_secs(20)), None);
        assert_eq!(l.due(t0 + Duration::from_secs(316)), Some(Outcome::Ignored));
        let mut st = Life::new(None, true, false, t0);
        st.hold(Duration::from_secs(900)); // an urgent card survives a long absence
        assert_eq!(st.due(t0 + Duration::from_secs(700)), None);
    }
}
