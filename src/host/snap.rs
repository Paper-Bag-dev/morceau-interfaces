//! Magnetic drag (docs/cards.md "Snapping"). While you carry a card, the host moves it itself (not the OS drag loop, which cannot be
//! steered mid-way) and pulls its visible content onto nearby edges:
//!   beside a target   dock to the active window's / another card's side, with the placement seam (gap) between
//!   stacked on one    dock above / below it
//!   lined up          once docked beside (or stacked), the other axis also lines up with the target's top / bottom (left / right)
//!   the screen        the work area's inner edges, with the placement margin
//! An axis that is snapped STAYS snapped while the pointer is within `breakaway` of it (so the card slides along the edge as you move
//! along it) and lets go past that; it re-snaps to whatever is nearest within `radius`. Each axis is independent: corner = both.
//! A fast throw (pointer faster than `snap-max-speed`) never snaps: moving a card to the other end of the screen is not slowed down by
//! every edge on the way; it snaps once you slow down near where you are going.
//! Settings: ui/limits.json `snap-px` (0 = off), `snap-break-px`, `snap-max-speed` (css px / s), `snap-frame-ms`, scaled to the display.

use crate::placement::R;

#[derive(Clone, Copy, Default, Debug)]
pub struct Sticky {
    pub x: Option<i32>,
    pub y: Option<i32>,
}

pub struct Grab {
    /// pointer minus window top-left when the carry began
    pub offset: (i32, i32),
    pub sticky: Sticky,
    /// last pointer spot and when, and the smoothed pointer speed (physical px / s): a fast throw across the screen never snaps
    pub last: Option<((i32, i32), std::time::Instant)>,
    pub speed: f64,
}

impl Grab {
    pub fn new(offset: (i32, i32)) -> Grab {
        Grab { offset, sticky: Sticky::default(), last: None, speed: 0.0 }
    }
    /// Feed the pointer; returns the smoothed speed. Smoothing (0.35 new / 0.65 old) keeps one jittery frame from deciding.
    pub fn track(&mut self, p: (i32, i32)) -> f64 {
        let now = std::time::Instant::now();
        if let Some((q, t)) = self.last {
            let dt = now.duration_since(t).as_secs_f64();
            if dt > 0.0 {
                let v = (((p.0 - q.0) as f64).powi(2) + ((p.1 - q.1) as f64).powi(2)).sqrt() / dt;
                self.speed = self.speed * 0.65 + v * 0.35;
            }
        }
        self.last = Some((p, now));
        self.speed
    }
}

pub struct Magnets<'a> {
    pub targets: &'a [R],
    pub work: R,
    pub radius: i32,
    pub breakaway: i32,
    pub gap: i32,
    pub margin: i32,
}

fn near(a: i32, b: i32, r: i32) -> bool {
    (a - b).abs() < r
}

/// Where the content's left (x) / top (y) may snap to, for content of size (w, h) wanted at `raw`.
fn candidates(raw: &R, m: &Magnets) -> (Vec<i32>, Vec<i32>) {
    let (w, h) = (raw.w(), raw.h());
    let (r, g) = (m.radius, m.gap);
    let mut xs = vec![m.work.l + m.margin, m.work.r - m.margin - w];
    let mut ys = vec![m.work.t + m.margin, m.work.b - m.margin - h];
    for t in m.targets {
        let beside_band = raw.t < t.b + r && raw.b > t.t - r;   // vertically level with it: can dock to its left / right side
        let stack_band = raw.l < t.r + r && raw.r > t.l - r;    // horizontally level with it: can dock above / below
        if beside_band {
            xs.push(t.r + g);
            xs.push(t.l - g - w);
        }
        if stack_band {
            ys.push(t.b + g);
            ys.push(t.t - g - h);
        }
        // lined up: when docked beside it, tops / bottoms line up; when stacked on it, left / right edges line up
        let docked_beside = beside_band && (near(raw.l, t.r + g, r) || near(raw.r, t.l - g, r));
        let docked_stacked = stack_band && (near(raw.t, t.b + g, r) || near(raw.b, t.t - g, r));
        if docked_beside {
            ys.push(t.t);
            ys.push(t.b - h);
        }
        if docked_stacked {
            xs.push(t.l);
            xs.push(t.r - w);
        }
    }
    (xs, ys)
}

fn axis(raw: i32, cands: &[i32], held: Option<i32>, m: &Magnets) -> Option<i32> {
    if let Some(h) = held {
        if cands.contains(&h) && near(raw, h, m.breakaway) {
            return Some(h);   // sticky: slide along it until pulled past the breakaway
        }
    }
    cands.iter().copied().filter(|c| near(raw, *c, m.radius)).min_by_key(|c| (raw - c).abs())
}

/// The content's snapped top-left for a wanted content rect, and the new sticky state. `fast` = the pointer is being thrown (a long
/// drag to the other side): nothing new snaps and a held edge lets go, so the card simply follows the hand.
pub fn apply_moving(raw: &R, m: &Magnets, st: Sticky, fast: bool) -> ((i32, i32), Sticky) {
    if fast {
        return ((raw.l, raw.t), Sticky::default());
    }
    apply(raw, m, st)
}

/// The content's snapped top-left for a wanted content rect, and the new sticky state.
pub fn apply(raw: &R, m: &Magnets, st: Sticky) -> ((i32, i32), Sticky) {
    let (xs, ys) = candidates(raw, m);
    let x = axis(raw.l, &xs, st.x, m);
    // the y candidates depend on where x ended up (lined up only once docked beside): recompute with the snapped x
    let moved = R { l: x.unwrap_or(raw.l), t: raw.t, r: x.unwrap_or(raw.l) + raw.w(), b: raw.b };
    let (_, ys2) = candidates(&moved, m);
    let y = axis(raw.t, if x.is_some() { &ys2 } else { &ys }, st.y, m);
    ((x.unwrap_or(raw.l), y.unwrap_or(raw.t)), Sticky { x, y })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(targets: &[R]) -> Magnets<'_> {
        Magnets { targets, work: R { l: 0, t: 0, r: 3000, b: 2000 }, radius: 40, breakaway: 80, gap: 6, margin: 14 }
    }
    fn rect(l: i32, t: i32, w: i32, h: i32) -> R {
        R { l, t, r: l + w, b: t + h }
    }

    #[test]
    fn docks_beside_slides_and_breaks_away() {
        let win = [rect(100, 100, 1000, 800)];
        let mm = m(&win);
        // 20 px off the window's right side: docks at r + gap
        let ((x, _), st) = apply(&rect(1126, 300, 400, 300), &mm, Sticky::default());
        assert_eq!(x, 1106);
        // pulled 60 px away (inside the breakaway): still docked, and it slides down freely
        let ((x, y), st) = apply(&rect(1166, 420, 400, 300), &mm, st);
        assert_eq!((x, y), (1106, 420));
        // pulled 100 px away: lets go
        let ((x, _), _) = apply(&rect(1206, 420, 400, 300), &mm, st);
        assert_eq!(x, 1206);
    }

    #[test]
    fn lines_up_with_the_top_once_docked() {
        let win = [rect(100, 100, 1000, 800)];
        let ((x, y), _) = apply(&rect(1120, 125, 400, 300), &m(&win), Sticky::default());
        assert_eq!((x, y), (1106, 100));
    }

    #[test]
    fn stacks_under_another_card_and_far_away_is_free() {
        let cards = [rect(2000, 200, 500, 300)];
        let ((x, y), _) = apply(&rect(2010, 520, 500, 200), &m(&cards), Sticky::default());
        assert_eq!((x, y), (2000, 506));   // below it (b + gap), left edges lined up
        let ((x, y), st) = apply(&rect(1500, 1100, 300, 200), &m(&cards), Sticky::default());
        assert_eq!((x, y), (1500, 1100));
        assert!(st.x.is_none() && st.y.is_none());
    }

    #[test]
    fn a_fast_throw_never_snaps() {
        let win = [rect(100, 100, 1000, 800)];
        let ((x, _), st) = apply_moving(&rect(1120, 300, 400, 300), &m(&win), Sticky { x: Some(1106), y: None }, true);
        assert_eq!(x, 1120);
        assert!(st.x.is_none());
    }

    #[test]
    fn screen_edges() {
        let ((x, y), _) = apply(&rect(2600, 30, 380, 200), &m(&[]), Sticky::default());
        assert_eq!((x, y), (3000 - 14 - 380, 14));
    }
}
