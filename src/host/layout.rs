//! A card reported its measured size: resize the window, set its hit shape, and place it (regions first, scored fallback). Split out of host/mod.rs.

use super::*;

impl Host {
    pub(super) fn layout(&mut self, key: u64, m: &Value) {
        let (kin, foreign) = self.estates(key);
        let Some(s) = self.surfaces.get_mut(&key) else { return };
        let (w, h) = (m["w"].as_f64().unwrap_or(0.0).ceil(), m["h"].as_f64().unwrap_or(0.0).ceil());
        if w < 1.0 || h < 1.0 {
            return;
        }
        let scale = s.window.scale_factor();
        s.window.set_inner_size(LogicalSize::new(w, h));
        let rects: Vec<(i32, i32, i32, i32, i32)> = m["rects"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|r| {
                        let v: Vec<f64> = r.as_array()?.iter().filter_map(|x| x.as_f64()).collect();
                        (v.len() == 5).then(|| (((v[0]) * scale) as i32, (v[1] * scale) as i32, ((v[0] + v[2]) * scale) as i32, ((v[1] + v[3]) * scale) as i32, (v[4] * scale) as i32))
                    })
                    .collect()
            })
            .unwrap_or_default();
        winstyle::set_region(s.hwnd(), &rects);
        // the plates' exact rounded rects (not the padded hit shapes): where the platform may put native glass
        if m["glass"].is_array() {
            // absent mid-morph: the page is streaming the glass frame by frame then
            winstyle::set_glass(s.hwnd(), &px_rects(&m["glass"], scale));
        }
        s.inset = if rects.is_empty() {
            (0, 0, 0, 0)
        } else {
            let (l, t) = (rects.iter().map(|r| r.0).min().unwrap_or(0), rects.iter().map(|r| r.1).min().unwrap_or(0));
            let (r, b) = (rects.iter().map(|r| r.2).max().unwrap_or(0), rects.iter().map(|r| r.3).max().unwrap_or(0));
            (l.max(0), t.max(0), ((w * scale) as i32 - r).max(0), ((h * scale) as i32 - b).max(0))
        };
        if std::env::var("MORC_DEBUG").is_ok() { util::log(&format!("layout {} css={}x{} scale={} win_outer={:?} inner={:?} rects={:?}", s.id, w, h, scale, s.window.outer_size(), s.window.inner_size(), rects.iter().take(3).collect::<Vec<_>>())); }

        // The page may still be fitting itself (scale / 1/4-screen cap). Place and show only once it says it settled,
        // otherwise a later resize leaves the window hanging off the screen.
        let settled = m["settled"].as_bool().unwrap_or(true);
        if s.phase == Phase::Waiting && !settled {
            return;
        }
        if s.phase == Phase::Visible && s.mv.is_none() && !s.life.dragging {
            if let Some(r) = s.rect() {
                let (il, it, ir, ib) = s.inset;
                let (x, y) = placement::clamp_inside(&placement::R { l: r.l + il, t: r.t + it, r: r.r - ir, b: r.b - ib });
                if (x - il, y - it) != (r.l, r.t) {
                    s.window.set_outer_position(PhysicalPosition::new(x - il, y - it));
                }
            }
        }

        if s.phase == Phase::Waiting {
            let (il, it, ir, ib) = s.inset;
            let size = ((w * scale) as i32 - il - ir, (h * scale) as i32 - it - ib);   // the visible content
            let ow = 1.2 - prefs::get(&prefs::load(), &s.topic, "overlap");
            // A spot the user carried this topic to before wins, unless it would sit on top of the active window right now.
            // ...and never on top of one of my own cards: cards of one topic would otherwise all pile onto the same remembered spot.
            let mine: Vec<placement::R> = kin.iter().chain(foreign.iter()).copied().collect();
            let remembered = placed::get(&s.topic).filter(|&(rx, ry)| {
                let r = placement::R { l: rx, t: ry, r: rx + size.0, b: ry + size.1 };
                !placement::covers_foreground(&r) && !mine.iter().any(|w| r.l < w.r && w.l < r.r && r.t < w.b && w.t < r.b)
            });
            let explicit = s.opts.at.map(|[ax, ay]| placement::clamp_inside(&placement::R { l: ax, t: ay, r: ax + size.0, b: ay + size.1 }));
            // Region placement: the host divides the free space and puts the content in a region, through the tier ladder (tier 0, tier 1,
            // then the screen thirds, right third first). Every card goes this way unless it names an explicit bottom/corner anchor
            // (prefs/placement.json "explicit_anchors"); only those, and cards nothing fits for, use the scored placement below.
            let mut region_name: Option<String> = None;
            let explicit_anchor = s.opts.anchor.as_deref().map(|a| crate::policy::load().explicit_anchors.iter().any(|e| e == a)).unwrap_or(false);
            let by_region = (s.opts.region.is_some() || s.opts.align.is_some() || !explicit_anchor)
                .then(|| regions::Spec { region: s.opts.region.clone(), align: s.opts.align.clone().unwrap_or_default(), dx: s.opts.dx.unwrap_or(0), dy: s.opts.dy.unwrap_or(0), sacrifice: prefs::get(&prefs::load(), &s.topic, "overlap") >= 0.2 || is_important(&s.opts) })
                .and_then(|sp| regions::place_named(size, &sp, &kin.iter().chain(foreign.iter()).copied().collect::<Vec<_>>()))
                .map(|((rx, ry), name)| {
                    region_name = Some(name);
                    placement::clamp_inside(&placement::R { l: rx, t: ry, r: rx + size.0, b: ry + size.1 })
                });
            // Order: --at, then a region the card NAMES, then the spot you carried this topic to, then the tier ladder for everything else.
            let named = s.opts.region.is_some() || s.opts.align.is_some();
            let (explicit, ladder) = if named { (explicit.or(by_region), None) } else { (explicit, by_region) };
            if !named && remembered.is_some() {
                region_name = None;   // a remembered spot is not a ladder region: keep the card's own alignment
            }
            let (x, y) = match (explicit, remembered) {
                (Some(p), _) => p,   // the agent did the math: --at X,Y wins
                (None, Some((rx, ry))) => placement::clamp_inside(&placement::R { l: rx, t: ry, r: rx + size.0, b: ry + size.1 }),
                (None, None) if ladder.is_some() => ladder.unwrap(),
                // Nothing in the ladder fits: start from the right (docs/chips/INDEX.md, Tier 0), top-right of the work area, not the scorer's corner.
                (None, None) if !explicit_anchor => {
                    let (w, m) = (placement::foreground().work, crate::policy::load().margin);
                    placement::clamp_inside(&placement::R { l: w.r - size.0 - m, t: w.t + m, r: w.r - m, b: w.t + m + size.1 })
                }
                (None, None) => placement::pick(size, s.opts.anchor.as_deref().unwrap_or("right"), scope_of(&s.topic), &kin, &foreign, ow, cohesion_for(s)),
            };
            s.window.set_outer_position(PhysicalPosition::new(x - il, y - it));
            if std::env::var("MORC_DEBUG").is_ok() {
                let f = placement::foreground();
                util::log(&format!("PLACE {} content={}x{} inset=({},{},{},{}) pick=({},{}) fg={:?} work={:?} kin={} foreign={} remembered={:?}", s.id, size.0, size.1, il, it, ir, ib, x, y, f.rect, f.work, kin.len(), foreign.len(), remembered));
            }
            // Tell the page what is behind it (before it appears) so it can pick contrasting colors.
            if s.opts.contrast.as_deref() != Some("off") {
                if let Some(rgb) = backdrop::average_rgb(x, y, size.0, size.1) {
                    s.send(&json!({"t": "backdrop", "rgb": rgb}));
                }
            }
            // Contents start at the edge NEAREST the active window by default: a card left of the window is right-aligned (it hugs the window), a card to its
            // right, above or below is left-aligned. A card whose JSON names an alignment (`align`) keeps it.
            if s.opts.align.is_none() {
                let right = region_name.as_deref().map(|n| n.contains("left")).unwrap_or(false);
                s.send(&json!({"t": "align", "right": right}));
            }
            winstyle::show_topmost_noactivate(s.hwnd());
            s.phase = Phase::Visible;
            s.phase_since = Instant::now();
            s.life.restart(s.opts.ttl, s.opts.sticky || s.opts.urgent, Instant::now());
            s.send(&json!({"t": "enter"}));
        }
    }
}


/// [[x, y, w, h, radius], ...] in CSS px -> (left, top, right, bottom, radius) in physical px.
pub(super) fn px_rects(v: &Value, scale: f64) -> Vec<(i32, i32, i32, i32, i32)> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|r| {
                    let v: Vec<f64> = r.as_array()?.iter().filter_map(|x| x.as_f64()).collect();
                    (v.len() == 5).then(|| ((v[0] * scale) as i32, (v[1] * scale) as i32, ((v[0] + v[2]) * scale) as i32, ((v[1] + v[3]) * scale) as i32, (v[4] * scale) as i32))
                })
                .collect()
        })
        .unwrap_or_default()
}
