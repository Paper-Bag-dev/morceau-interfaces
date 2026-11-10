//! Regions: the free rectangles on screen, by math, and placement INTO a region with an alignment and an offset.
//! The host owns all geometry; the agent only names what it wants (`--region right --align top --dx 12`), so placement needs no
//! per-widget math from the agent and no rebuild per tweak.
//!
//! Model: the active window is the centre cell of a 3x3 grid cut by its own edges. The other eight cells are the regions:
//!     top-left   above   top-right
//!     left       [ACTIVE] right
//!     bottom-left below  bottom-right
//! plus four composites that span the whole band: right-col, left-col, below-row, above-row.
//! Free space in each = the cell minus what is occupied:
//!   tier 0: every visible window + my widgets   (gaps between windows, between a window and the monitor edge)
//!   tier 1: only the active window + my widgets (non-active windows are sacrificed: a widget may sit on top of them)
//! With no usable active window (fullscreen, desktop, or one maximised so no side is free) the screen is split into thirds: right | mid | left (tier 0 only).
//! Default fill order: right, then down (below), then left, then the corners. If nothing fits, the caller falls back to the
//! scoring placement in placement.rs (least overlap).
//!
//! Default alignment hugs the active window (the dock side), with a small pad: right -> left+top, left -> right+top,
//! below -> top+right, above -> bottom+right, corners -> toward the window's corner. `--align` overrides it per push.

use crate::placement::{self, R};
use crate::policy;
use serde_json::{json, Value};

const MIN_W: i32 = 160;
const MIN_H: i32 = 90;
const PER_NAME: usize = 2;

#[derive(Clone, Debug)]
pub struct Region {
    pub name: String,
    pub rect: R,
    pub tier: u8,
}

pub struct Spec {
    pub region: Option<String>,
    pub align: String,
    pub dx: i32,
    pub dy: i32,
    /// may the widget sit on top of NON-active windows (tier 1)? From the annoyance vector: the `overlap` knob.
    pub sacrifice: bool,
}

fn clip(r: &R, w: &R) -> Option<R> {
    let c = R { l: r.l.max(w.l), t: r.t.max(w.t), r: r.r.min(w.r), b: r.b.min(w.b) };
    (c.w() > 0 && c.h() > 0).then_some(c)
}

/// Every maximal empty rectangle inside `region` that touches no obstacle (grid compression + prefix sums), largest first.
pub fn free_rects(region: &R, obs: &[R]) -> Vec<R> {
    let o: Vec<R> = obs.iter().filter_map(|r| clip(r, region)).collect();
    let (mut xs, mut ys) = (vec![region.l, region.r], vec![region.t, region.b]);
    for r in &o {
        xs.extend([r.l, r.r]);
        ys.extend([r.t, r.b]);
    }
    xs.sort_unstable();
    xs.dedup();
    ys.sort_unstable();
    ys.dedup();
    if xs.len() < 2 || ys.len() < 2 {
        return vec![];
    }
    let (nx, ny) = (xs.len() - 1, ys.len() - 1);
    let mut ps = vec![vec![0i32; nx + 1]; ny + 1];
    for j in 0..ny {
        for i in 0..nx {
            let occ = o.iter().any(|r| r.l <= xs[i] && xs[i + 1] <= r.r && r.t <= ys[j] && ys[j + 1] <= r.b) as i32;
            ps[j + 1][i + 1] = occ + ps[j][i + 1] + ps[j + 1][i] - ps[j][i];
        }
    }
    let empty = |i0: usize, i1: usize, j0: usize, j1: usize| ps[j1][i1] - ps[j0][i1] - ps[j1][i0] + ps[j0][i0] == 0;
    let mut out: Vec<R> = vec![];
    for i0 in 0..nx {
        for i1 in i0 + 1..=nx {
            for j0 in 0..ny {
                for j1 in j0 + 1..=ny {
                    if !empty(i0, i1, j0, j1) {
                        continue;
                    }
                    // maximal = cannot grow by one cell in any direction
                    let grow = (i0 > 0 && empty(i0 - 1, i1, j0, j1)) || (i1 < nx && empty(i0, i1 + 1, j0, j1)) || (j0 > 0 && empty(i0, i1, j0 - 1, j1)) || (j1 < ny && empty(i0, i1, j0, j1 + 1));
                    if !grow {
                        out.push(R { l: xs[i0], t: ys[j0], r: xs[i1], b: ys[j1] });
                    }
                }
            }
        }
    }
    out.sort_by_key(|r| -r.area());
    out
}

/// The eight cells around the active window `a` inside `w`, plus the four full-width/height bands.
fn cells(a: &R, w: &R) -> Vec<(&'static str, R)> {
    let xs = [w.l, a.l, a.r, w.r];
    let ys = [w.t, a.t, a.b, w.b];
    let c = |i: usize, j: usize| R { l: xs[i], t: ys[j], r: xs[i + 1], b: ys[j + 1] };
    vec![
        ("top-left", c(0, 0)),
        ("above", c(1, 0)),
        ("top-right", c(2, 0)),
        ("left", c(0, 1)),
        ("right", c(2, 1)),
        ("bottom-left", c(0, 2)),
        ("below", c(1, 2)),
        ("bottom-right", c(2, 2)),
        ("right-col", R { l: a.r, t: w.t, r: w.r, b: w.b }),
        ("left-col", R { l: w.l, t: w.t, r: a.l, b: w.b }),
        ("below-row", R { l: w.l, t: a.b, r: w.r, b: w.b }),
        ("above-row", R { l: w.l, t: w.t, r: w.r, b: a.t }),
    ]
}

/// All regions (both tiers), the mode, and the active rect. `widgets` = my own visible surfaces (always occupied).
pub fn compute(widgets: &[R]) -> (Vec<Region>, &'static str, Option<R>) {
    let fg = placement::foreground();
    let work = fg.work;
    let active = fg.rect.filter(|_| !fg.fullscreen).and_then(|a| clip(&a, &work));
    let mut out: Vec<Region> = vec![];
    let mut add = |name: &str, cell: &R, obs: &[R], tier: u8| {
        for r in free_rects(cell, obs).into_iter().filter(|r| r.w() >= MIN_W && r.h() >= MIN_H).take(PER_NAME) {
            out.push(Region { name: name.to_string(), rect: r, tier });
        }
    };
    match active {
        // A maximised active window leaves no free side in either tier: fall through to the screen thirds (the same rung as fullscreen)
        // instead of the scored corner fallback.
        Some(a) if cells(&a, &work).iter().any(|(_, cell)| free_rects(cell, &[a]).iter().any(|r| r.w() >= MIN_W && r.h() >= MIN_H)) => {
            let wins = placement::windows();
            let tier0: Vec<R> = wins.iter().chain(std::iter::once(&a)).chain(widgets.iter()).copied().collect();
            let tier1: Vec<R> = std::iter::once(a).chain(widgets.iter().copied()).collect();
            for (name, cell) in cells(&a, &work) {
                add(name, &cell, &tier0, 0);
            }
            for (name, cell) in cells(&a, &work) {
                add(name, &cell, &tier1, 1);
            }
            (out, "around-active", Some(a))
        }
        _ => {
            let third = work.w() / 3;
            let parts = [("right", R { l: work.r - third, t: work.t, r: work.r, b: work.b }), ("mid", R { l: work.l + third, t: work.t, r: work.r - third, b: work.b }), ("left", R { l: work.l, t: work.t, r: work.l + third, b: work.b })];
            for (name, reg) in parts {
                add(name, &reg, widgets, 0);
            }
            (out, "screen-thirds", None)
        }
    }
}

/// The default alignment of a region: hug the active window (the dock side).
fn default_align(name: &str) -> &'static str {
    match name {
        "right" | "right-col" | "bottom-right" => "left-top",
        "left" | "left-col" | "bottom-left" => "right-top", // hug the active window: a card in the left side sits at its right edge, next to the window
        "below" | "below-row" => "top-right",
        "above" | "above-row" => "bottom-right",
        "top-right" => "left-bottom",
        "top-left" => "right-bottom",
        _ => "right-top", // thirds: start from the right, go down
    }
}

fn aligned(r: &R, (w, h): (i32, i32), align: &str, name: &str, dx: i32, dy: i32, m: i32, gap: i32) -> (i32, i32) {
    // A partial --align (e.g. just "top") overrides only its own axis; the other axis keeps the region's default (hug the active window).
    let split = |s: &'static str| -> Vec<&str> { s.split(|c: char| c == '-' || c == ',' || c == ' ').filter(|t| !t.is_empty()).collect() };
    let given: Vec<&str> = align.split(|c: char| c == '-' || c == ',' || c == ' ').filter(|t| !t.is_empty()).collect();
    let mut toks: Vec<&str> = given.clone();
    let horiz = |t: &&str| matches!(*t, "left" | "right");
    let vert = |t: &&str| matches!(*t, "top" | "bottom");
    let mid_given = given.iter().any(|t| matches!(*t, "middle" | "center"));
    for d in split(default_align(name)) {
        let covered = (horiz(&d) && (given.iter().any(horiz) || mid_given)) || (vert(&d) && (given.iter().any(vert) || mid_given));
        if !covered {
            toks.push(d);
        }
    }
    let has = |t: &str| toks.contains(&t);
    let mid = has("middle") || has("center");
    // the edge that touches the active window gets the small seam, the others the normal margin
    let (pl, pr, pt, pb) = match name {
        "right" | "right-col" | "bottom-right" => (gap, m, m, m),
        "left" | "left-col" | "bottom-left" => (m, gap, m, m),
        "below" | "below-row" => (m, m, gap, m),
        "above" | "above-row" => (m, m, m, gap),
        "top-right" => (gap, m, m, gap),
        "top-left" => (m, gap, m, gap),
        _ => (m, m, m, m),
    };
    let x = if has("left") { r.l + pl } else if has("right") { r.r - w - pr } else if mid { (r.l + r.r - w) / 2 } else { r.r - w - pr };
    let y = if has("top") { r.t + pt } else if has("bottom") { r.b - h - pb } else if mid { (r.t + r.b - h) / 2 } else { r.t + pt };
    (x + dx, y + dy)
}

fn candidates<'a>(regs: &'a [Region], wanted: Option<&str>, tiers: u8) -> Vec<&'a Region> {
    let mut order: Vec<&Region> = vec![];
    if let Some(name) = wanted {
        order.extend(regs.iter().filter(|r| r.name == name && r.tier < tiers));
    }
    for tier in 0..tiers {
        // "auto" = go where the room is: the biggest free region first (the active window is on one side, the free side is the other),
        // not a fixed right-first order that puts cards over a busy side while the other side is empty
        let mut here: Vec<&Region> = regs.iter().filter(|r| r.tier == tier).collect();
        // the four main sides (left, right, above, below) always come first, biggest first; corners and the merged column/row variants only after
        here.sort_by_key(|r| (!matches!(r.name.as_str(), "left" | "right" | "above" | "below"), std::cmp::Reverse(r.rect.w() as i64 * r.rect.h() as i64)));
        order.extend(here);
    }
    order
}

/// Where a content box of `size` goes for this spec (the wanted region if it fits, tier 0 then 1, else the first that fits in fill order),
/// and WHICH region it chose ("left", "right", "above", "below", a corner ...): the card's contents then start at the edge
/// nearest the active window (a card left of the window right-aligns, a card right of it left-aligns).
pub fn place_named(size: (i32, i32), spec: &Spec, widgets: &[R]) -> Option<((i32, i32), String)> {
    let (regs, _, _) = compute(widgets);
    let pol = policy::load();
    let (m, gap) = (pol.margin, pol.gap);
    let wanted = spec.region.as_deref().filter(|r| *r != "auto");
    // Tier 0 (free of every window) first; when a big background window covers the whole screen tier 0 is empty, and then the free sides of the
    // ACTIVE window (tier 1) are the free space: background windows never block a card. (`sacrifice` no longer gates this.)
    let _ = spec.sacrifice;
    let r = candidates(&regs, wanted, 2).into_iter().find(|r| r.rect.w() >= size.0 + m + gap && r.rect.h() >= size.1 + m + gap)?;
    Some((aligned(&r.rect, size, &spec.align, &r.name, spec.dx, spec.dy, m, gap), r.name.clone()))
}

/// The size (physical px) of the region a spec will aim at, BEFORE the content is measured, so the page can wrap and flow to fit it.
/// `min` = the smallest the content can be (declared by the component): regions too small for it are skipped. `widgets` = my cards already on screen:
/// the region is what is REALLY left after them (a third or a side), so the card is sized to that and shrinks to fit it. None = no region qualifies,
/// the page keeps its default sizing.
pub fn size_for(region: Option<&str>, sacrifice: bool, min: Option<(i32, i32)>, widgets: &[R]) -> Option<(i32, i32)> {
    let (regs, _, _) = compute(widgets);
    let pol = policy::load();
    let (mw, mh) = min.unwrap_or((0, 0));
    let _ = sacrifice;
    candidates(&regs, region.filter(|r| *r != "auto"), 2)
        .into_iter()
        .find(|r| r.rect.w() >= mw + pol.margin + pol.gap && r.rect.h() >= mh + pol.margin + pol.gap)
        .map(|r| (r.rect.w(), r.rect.h()))
}

/// Everything the agent may want to know, in one query (`morc space`). Rects are [left, top, right, bottom] in physical px.
pub fn snapshot(widgets: &[R]) -> Value {
    let fg = placement::foreground();
    let (regs, mode, active) = compute(widgets);
    let a = |r: &R| vec![r.l, r.t, r.r, r.b];
    let tier = |t: u8| regs.iter().filter(|r| r.tier == t).map(|r| json!({"name": r.name, "rect": a(&r.rect)})).collect::<Vec<_>>();
    let tz = policy::load().toast_zone;
    json!({
        "ok": true,
        "mode": mode,
        "work": a(&fg.work),
        "active": active.map(|x| a(&x)),
        "fullscreen": fg.fullscreen,
        "A": tier(0),
        "B": tier(1),
        "windows": placement::windows().iter().map(|w| a(w)).collect::<Vec<_>>(),
        "toast_zone": a(&R { l: fg.work.r - tz[0], t: fg.work.b - tz[1], r: fg.work.r, b: fg.work.b }),
    })
}
