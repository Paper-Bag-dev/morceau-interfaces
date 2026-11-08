//! Scored placement: try candidate spots on the work area and pick the cheapest (avoid the active window, other windows and our own cards, pull toward the anchor).

use super::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// Where each context scope last landed (so the next surface of that scope assimilates to it, even after the first one closed).
static LAST: Mutex<Option<HashMap<String, (R, Instant)>>> = Mutex::new(None);

fn last_of(scope: &str) -> Option<R> {
    let g = LAST.lock().ok()?;
    g.as_ref()?.get(scope).filter(|(_, t)| t.elapsed().as_secs() < 900).map(|(r, _)| *r)
}

fn remember(scope: &str, r: R) {
    if let Ok(mut g) = LAST.lock() {
        g.get_or_insert_with(HashMap::new).insert(scope.to_string(), (r, Instant::now()));
    }
}

/// How flush a candidate sits against one rect (0..1.35): touching along an edge with a shared run, with extra credit for
/// lining up a CORNER (top edges level beside it, or right/left edges level below it). Stacking ABOVE it is discouraged:
/// a surface should sit beside the window, aligned to its top-right corner, not crowd the strip above it.
fn flush_to(c: &R, o: &R) -> f64 {
    let tol = 14;
    let vshare = ((c.b.min(o.b) - c.t.max(o.t)).max(0) as f64 / c.h().max(1) as f64).min(1.0);
    let hshare = ((c.r.min(o.r) - c.l.max(o.l)).max(0) as f64 / c.w().max(1) as f64).min(1.0);
    let mut best: f64 = 0.0;
    if (c.l - o.r).abs() <= tol || (c.r - o.l).abs() <= tol {
        let mut v = 0.4 + 0.6 * vshare;   // side by side
        if (c.t - o.t).abs() <= tol { v += 0.35; }   // top corners level: the "docked to its top-right corner" look
        else if (c.b - o.b).abs() <= tol { v += 0.2; }
        best = best.max(v);
    }
    if (c.t - o.b).abs() <= tol {   // stacked below it
        let mut v = 0.4 + 0.6 * hshare;
        if (c.r - o.r).abs() <= tol || (c.l - o.l).abs() <= tol { v += 0.35; }
        best = best.max(v);
    }
    if (c.b - o.t).abs() <= tol {   // stacked above it: crowding, so only a weak pull
        best = best.max(0.35 * (0.4 + 0.6 * hshare));
    }
    best
}

/// Knobs from the annoyance vector that shape placement.
#[derive(Clone, Copy)]
pub struct Cohesion {
    /// snap flush to neighbouring windows and the screen edge
    pub dock: f64,
    /// line up with my own other surfaces (shared edges and gaps)
    pub harmony: f64,
    /// cost per px^2 of covering the active window. High (default) = never; low only for important/urgent surfaces.
    pub cover: f64,
    /// cost per px^2 of sitting in the bottom-right toast zone (low when the surface explicitly asks for a bottom anchor)
    pub toast: f64,
}

fn edge_of(c: &R, e: &str) -> Option<f64> {
    Some(match e {
        "left" => c.l,
        "right" => c.r,
        "top" => c.t,
        "bottom" => c.b,
        "cx" => (c.l + c.r) / 2,
        "cy" => (c.t + c.b) / 2,
        _ => return None,
    } as f64)
}

/// Resolve a policy target such as "work.right", "fg.bottom", "work.y:0.35". None when it needs a window that is not there.
fn target_of(to: &str, work: &R, fg: &Option<R>) -> Option<f64> {
    let (who, what) = to.split_once('.')?;
    let r = match who {
        "work" => *work,
        "fg" => (*fg)?,
        _ => return None,
    };
    if let Some(f) = what.strip_prefix("y:") {
        return Some(r.t as f64 + r.h() as f64 * f.parse::<f64>().ok()?);
    }
    if let Some(f) = what.strip_prefix("x:") {
        return Some(r.l as f64 + r.w() as f64 * f.parse::<f64>().ok()?);
    }
    edge_of(&r, what)
}

/// Total pull of an anchor's terms on a candidate (data-driven: see policy.rs).
fn anchor_cost(terms: &[policy::Term], cand: &R, work: &R, fg: &Option<R>) -> f64 {
    let mut cost = 0.0;
    for t in terms {
        if t.edge == "gap" {
            if let Some(f) = fg {
                cost += cand.gap(f) * t.w;
            }
            continue;
        }
        if let (Some(e), Some(tg)) = (edge_of(cand, &t.edge), target_of(&t.to, work, fg)) {
            cost += (e - (tg + t.off)).abs() * t.w;
        }
    }
    cost
}

/// Top-left (physical px) for a surface of `size`, avoiding `mine` (our other surfaces).
/// kin = my other surfaces of the SAME context scope (line up with them); foreign = surfaces of a different scope
/// (keep away: each context keeps its own estate, and a dissimilar newcomer goes to the opposite side of the screen).
/// Weights, thresholds and anchor shapes come from prefs/placement.json (policy.rs): tune them without rebuilding.
pub fn pick(size: (i32, i32), anchor: &str, scope: &str, kin: &[R], foreign: &[R], others_w: f64, co: Cohesion) -> (i32, i32) {
    let pol = policy::load();
    let wt = &pol.weights;
    let mine: Vec<R> = kin.iter().chain(foreign.iter()).copied().collect();
    let mine = &mine[..];
    // Previous context: if no sibling is on screen, the spot this scope last used still counts as a (ghost) sibling to line up with.
    let ghost = if kin.is_empty() { last_of(scope) } else { None };
    let kin_eff: Vec<R> = kin.iter().copied().chain(ghost).collect();
    let kin_eff = &kin_eff[..];
    let fg = foreground();
    let wins: Vec<R> = windows();   // desktop.rs: every visible window except ours and the shell
    let work = fg.work;
    let (w, h) = size;
    let (margin, gapx) = (pol.margin, pol.gap);
    // If the active window blankets the work area there is no free side.
    let blanket = fg.rect.map(|r| r.overlap(&work) as f64 > work.area() as f64 * 0.85).unwrap_or(false);

    // Resolve the anchor through its fallbacks (data: policy anchors[...].fallback).
    let mut anchor = anchor.to_string();
    for _ in 0..3 {
        let Some(fb) = pol.anchors.get(&anchor).and_then(|a| a.fallback.clone()) else { break };
        let hit = match fb.when.as_str() {
            "blanket" => blanket,
            "no_fg" => fg.rect.is_none(),
            "strip_below_lt_h" => fg.rect.map(|f| work.b - f.b - margin < h).unwrap_or(true),
            "room_right_lt_w" => fg.rect.map(|f| work.r - f.r - margin < w).unwrap_or(true),
            _ => false,
        };
        if hit { anchor = fb.anchor } else { break }
    }

    // Windows toasts and the notification flyouts stack up from the bottom-right: keep out of that zone so they never
    // land on top of (or push against) a surface.
    let toast = R { l: work.r - pol.toast_zone[0], t: work.b - pol.toast_zone[1], r: work.r, b: work.b };

    let (x0, x1) = (work.l + margin, (work.r - w - margin).max(work.l + margin));
    let (y0, y1) = (work.t + margin, (work.b - h - margin).max(work.t + margin));
    // Candidates: a coarse grid, plus positions that sit exactly flush to every neighbour (and to my own surfaces).
    let mut cands: Vec<(i32, i32)> = vec![];
    let step = pol.step.max(5);
    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            cands.push((x, y));
            x += step;
        }
        y += step;
    }
    let mut snap = |o: &R| {
        for &sx in &[o.r + gapx, o.l - w - gapx] {
            for &sy in &[o.t, o.b - h, y0, y1, (o.t + o.b - h) / 2] {
                cands.push((sx.clamp(x0, x1), sy.clamp(y0, y1)));
            }
        }
        for &sy in &[o.b + gapx, o.t - h - gapx] {
            for &sx in &[o.l, o.r - w, x1, x0] {
                cands.push((sx.clamp(x0, x1), sy.clamp(y0, y1)));
            }
        }
    };
    if co.dock > 0.05 {
        for o in fg.rect.iter().chain(wins.iter()) { snap(o); }
    }
    if co.harmony > 0.05 {
        for o in kin_eff { snap(o); }
    }
    // A different context already holds the screen: start from the opposite side (its horizontal and vertical centre of mass).
    if !foreign.is_empty() {
        let cx = foreign.iter().map(|r| (r.l + r.r) / 2).sum::<i32>() / foreign.len() as i32;
        let cy = foreign.iter().map(|r| (r.t + r.b) / 2).sum::<i32>() / foreign.len() as i32;
        anchor = match (cx > (work.l + work.r) / 2, cy > (work.t + work.b) / 2) {
            (true, true) => "left_top",
            (true, false) => "left_bottom",
            (false, true) => "right_top",
            (false, false) => "right_bottom",
        }
        .to_string();
    }
    let terms = pol.anchors.get(&anchor).or_else(|| pol.anchors.get("near_focus")).map(|a| a.terms.clone()).unwrap_or_default();

    let mut best = (f64::MAX, (work.r - w - margin, work.b - h - margin));
    for (x, y) in cands {
        let cand = R { l: x, t: y, r: x + w, b: y + h };
        let mut cost = 0.0;
        if let Some(f) = fg.rect {
            cost += cand.overlap(&f) as f64 * co.cover;   // if NOTHING fits, the least-covering spot still wins
        }
        for o in &wins {
            if fg.rect.map(|f| (f.l, f.t, f.r, f.b) == (o.l, o.t, o.r, o.b)).unwrap_or(false) {
                continue;
            }
            cost += cand.overlap(o) as f64 * others_w; // from the annoyance vector ("overlap")
        }
        for m in mine {
            cost += cand.overlap(m) as f64 * wt.mine_overlap;
        }
        // Estates: stay clear of other contexts (up to ~900 px of separation is rewarded), so they never eat each other's room.
        if !foreign.is_empty() {
            let d = foreign.iter().map(|f| cand.gap(f)).fold(f64::MAX, f64::min).min(900.0);
            cost -= d * wt.foreign_sep;
        }
        cost += cand.overlap(&toast) as f64 * co.toast; // discouraged, not forbidden (small screens)
        // Pull toward the anchor (softened as dock grows: the neighbours matter more than the screen edge).
        cost += (1.0 - wt.anchor_soften * co.dock) * anchor_cost(&terms, &cand, &work, &fg.rect);
        // Dock: a flush fit is worth up to dock_flush cost units at full strength.
        let mut fl: f64 = 0.0;
        if let Some(f) = fg.rect { fl = fl.max(flush_to(&cand, &f)); }
        for o in &wins { fl = fl.max(0.7 * flush_to(&cand, o)); }
        if (work.r - cand.r).abs() <= 14 || (cand.l - work.l).abs() <= 14 { fl = fl.max(0.45); }
        cost -= co.dock * wt.dock_flush * fl;
        // Harmony: line up with my other surfaces (stack or sit side by side with a tidy seam).
        let mut hm: f64 = 0.0;
        for m in kin_eff { hm = hm.max(flush_to(&cand, m)); }
        cost -= co.harmony * wt.harmony_flush * hm;
        if cost < best.0 {
            best = (cost, (x, y));
        }
    }
    remember(scope, R { l: best.1 .0, t: best.1 .1, r: best.1 .0 + w, b: best.1 .1 + h });
    best.1
}
