//! Live card operations (docs/cards.md): what "hide that for a bit", "bring it to the front", "make it big and centred" need, done to
//! the card that is on screen. Nothing is shown again: the page, its data and its spot are kept, so the card never pops twice.
//!   hide   off screen, page alive, clock stopped (Phase::Hidden: placement, expiry and the scene tree skip it)
//!   show   back where it was, in front, clock restarted
//!   front  in front of the other windows, without taking focus (and back on top if it was lowered)
//!   lower / raise   one step down / up the window stack (`depth`: 0 = on top, n = under the n-th window overlapping it; the card's
//!                   layer button toggles 0 <-> 1; Up / Down while the pointer is on the card step it too); `layer` = top | below
//!   move   --at X,Y (visible content's top-left, physical px), --center (its display's work area) or --snap (nearest edge); glides there
//!   size   --frac F: the size cap as a fraction of the work area (fill), the page re-fits; --center centres it once measured
//!   get / set   the card as a resource: read its state, or set the state you want ({"state","at","center","frac"})
//!
//! Records (every card the host knows, live, hidden or closed: state, spot, chip, spec, last data) live in memory and go to
//! <runtime>/cards.json with a FALLOFF: a change only marks them dirty; the tick writes once things are quiet for WRITE_QUIET, and at
//! most every WRITE_MAX_WAIT while they never are (a card dragged around for fun costs one write per 5 s, not one per frame). Spots are
//! read from the windows at write time, never stored per move. Temp file + rename: never half a file.

use super::*;

const WRITE_QUIET: Duration = Duration::from_millis(1000);
const WRITE_MAX_WAIT: Duration = Duration::from_secs(5);
const KEEP: usize = 60;   // closed cards remembered (newest first); live and hidden ones are always kept

fn file() -> std::path::PathBuf {
    paths::runtime_dir().join("cards.json")
}

/// The records from the last run. A card that was shown died with that host: it is "closed" now (bring it back with `show`); a hidden
/// one stays "hidden" (it was put away on purpose).
pub(super) fn load() -> serde_json::Map<String, Value> {
    let mut m = std::fs::read_to_string(file()).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| v.as_object().cloned()).unwrap_or_default();
    for r in m.values_mut() {
        if r["state"] != "hidden" {
            r["state"] = json!("closed");
        }
    }
    m
}

/// The write falloff: when the first unsaved change happened and when the latest did.
#[derive(Default)]
pub(super) struct Dirty {
    first: Option<Instant>,
    last: Option<Instant>,
}

impl Dirty {
    pub(super) fn mark(&mut self) {
        let now = Instant::now();
        self.first.get_or_insert(now);
        self.last = Some(now);
    }
    /// Due when quiet long enough, or when changes have kept coming for WRITE_MAX_WAIT.
    fn due(&self) -> bool {
        match (self.first, self.last) {
            (Some(f), Some(l)) => l.elapsed() >= WRITE_QUIET || f.elapsed() >= WRITE_MAX_WAIT,
            _ => false,
        }
    }
}

/// Where the window must go for the card's visible content to sit in the middle of its display's work area.
pub(super) fn centered(s: &Surface) -> Option<(i32, i32)> {
    let r = s.rect()?;
    let c = s.content_rect()?;
    let work = placement::work_at((c.l + c.r) / 2, (c.t + c.b) / 2);
    let (x, y) = (work.l + (work.w() - c.w()) / 2, work.t + (work.h() - c.h()) / 2);
    Some((x - (c.l - r.l), y - (c.t - r.t)))
}

impl Host {
    /// Put a card at a place in the window stack (0 = on top; n = under the n-th window overlapping it): the platform does it (and says
    /// how low it could really go), the card's button shows it, the record and the scene tree say it. Returns the depth applied.
    pub(super) fn set_depth(&mut self, key: u64, depth: usize) -> usize {
        let Some(s) = self.surfaces.get_mut(&key) else { return 0 };
        if s.phase == Phase::Visible {
            if let Some(r) = s.content_rect() {
                s.depth = winstyle::set_depth(s.hwnd(), depth, r);
            }
        } else {
            s.depth = depth;   // hidden: applied when it is shown again
        }
        s.send(&json!({"t": "layer", "below": s.depth > 0, "depth": s.depth}));
        s.life.on_interact(Instant::now());
        let (d, id) = (s.depth, s.id.clone());
        self.note_card(&id);
        d
    }

    /// Ctrl+Option+Up / Down: every visible card to the top of the stack, or as low as each can go.
    pub(super) fn stack_all(&mut self, dir: i32) {
        let keys: Vec<u64> = self.surfaces.values().filter(|s| s.phase == Phase::Visible).map(|s| s.key).collect();
        for k in keys {
            self.set_depth(k, if dir < 0 { 0 } else { usize::MAX });   // the platform clamps to how low it can really go
        }
    }

    /// Up / Down while the pointer is on a card: one step down or up the stack (as low as it can go, as high as the top).
    pub(super) fn step(&mut self, dir: i32) {
        let Some(k) = self.hovered else { return };
        let Some(d) = self.surfaces.get(&k).map(|s| s.depth) else { return };
        let want = if dir < 0 { d.saturating_sub(1) } else { d + 1 };
        self.set_depth(k, want);
    }

    /// The relayed shared store (unlinked cards): keep the change and pass it to every other card (`from` already has it).
    pub(super) fn shared_set(&mut self, from: Option<u64>, path: &str, value: Value) {
        util::set_pointer(&mut self.shared, path, value.clone());
        for s in self.surfaces.values().filter(|s| s.ready && Some(s.key) != from && s.phase != Phase::Spare) {
            s.send(&json!({"t": "shared", "path": path, "value": value}));
        }
    }

    /// A record as an answer: the stored one, with the live state and spot read now.
    fn card_json(&self, id: &str) -> Value {
        let mut r = self.records.get(id).cloned().unwrap_or(json!({"id": id}));
        if let Some(s) = self.by_id.get(id).and_then(|k| self.surfaces.get(k)) {
            r["state"] = json!(match s.phase { Phase::Hidden => "hidden", Phase::Leaving => "closing", Phase::Visible => "shown", _ => "opening" });
            if let Some(c) = s.content_rect() {
                r["rect"] = json!([c.l, c.t, c.w(), c.h()]);
            }
            r["pinned"] = json!(s.life.pinned);
            r["layer"] = json!(if s.depth > 0 { "below" } else { "top" });
            r["depth"] = json!(s.depth);
            r["frac"] = json!(s.opts.max_frac);
        }
        for k in ["components", "data", "sources", "opts"] {
            if let Some(o) = r.as_object_mut() {
                o.remove(k);   // the spec stays on disk for restoring; the answer is the state
            }
        }
        r["ok"] = json!(true);
        r
    }

    pub(super) fn cards_list(&self) -> Value {
        let mut ids: Vec<String> = self.records.keys().cloned().collect();
        ids.extend(self.surfaces.values().filter(|s| !s.id.is_empty() && s.phase != Phase::Spare && !self.records.contains_key(&s.id)).map(|s| s.id.clone()));
        let mut cards: Vec<Value> = ids.iter().map(|i| self.card_json(i)).collect();
        cards.sort_by_key(|c| std::cmp::Reverse(c["at"].as_u64().unwrap_or(0)));
        json!({"ok": true, "cards": cards})
    }

    /// Bring the record up to date with the live card (state, spot, spec) and mark it for the next write.
    pub(super) fn note_card(&mut self, id: &str) {
        let Some(s) = self.by_id.get(id).and_then(|k| self.surfaces.get(k)) else { return };
        let r = self.records.entry(id.to_string()).or_insert_with(|| json!({"id": id}));
        r["state"] = json!(if s.phase == Phase::Hidden { "hidden" } else { "shown" });
        r["layer"] = json!(if s.depth > 0 { "below" } else { "top" });
        r["depth"] = json!(s.depth);
        r["chip"] = json!(s.name);
        r["topic"] = json!(s.topic);
        r["at"] = json!(util::now_ms() as u64);
        if let Some(c) = s.content_rect().filter(|c| c.w() > 0) {
            r["rect"] = json!([c.l, c.t, c.w(), c.h()]);
        }
        r["opts"] = serde_json::to_value(&s.opts).unwrap_or(json!({}));
        for k in ["components", "sources"] {
            r[k] = s.last_init[k].clone();
        }
        if r["data"].is_null() {
            r["data"] = s.last_init["data"].clone();
        }
        self.cards_dirty.mark();
    }

    pub(super) fn note_shown(&mut self, reply: &Value) {
        if reply["status"] == "shown" {
            if let Some(id) = reply["id"].as_str() {
                if let Some(r) = self.records.get_mut(id) {
                    r["data"] = Value::Null;   // a fresh show: its own data wins over what an older one left
                }
                self.note_card(id);
            }
        }
    }

    /// The page's data as it left (ipc "final").
    pub(super) fn note_final(&mut self, key: u64, data: &Value) {
        let Some(s) = self.surfaces.get(&key) else { return };
        let id = s.id.clone();
        self.note_card(&id);
        if let Some(r) = self.records.get_mut(&id) {
            r["data"] = data.clone();
        }
    }

    pub(super) fn note_closed(&mut self, key: u64) {
        let Some(s) = self.surfaces.get(&key) else { return };
        if s.id.is_empty() || s.phase == Phase::Spare {
            return;
        }
        if let Some(r) = self.records.get_mut(&s.id) {
            r["state"] = json!("closed");   // the spot was taken when the close began (begin_leave), not mid-animation
            r["at"] = json!(util::now_ms() as u64);
            self.cards_dirty.mark();
        }
    }

    /// From tick: write the records if the falloff says so.
    pub(super) fn flush_cards(&mut self, force: bool) {
        if !(force || self.cards_dirty.due()) || self.cards_dirty.first.is_none() {
            return;
        }
        self.cards_dirty = Dirty::default();
        let live: Vec<String> = self.records.keys().filter(|i| self.by_id.contains_key(*i)).cloned().collect();
        for id in live {
            if let Some(c) = self.by_id.get(&id).and_then(|k| self.surfaces.get(k)).filter(|s| s.phase == Phase::Visible).and_then(|s| s.content_rect()).filter(|c| c.w() > 0) {
                self.records[&id]["rect"] = json!([c.l, c.t, c.w(), c.h()]);
            }
        }
        // closed cards fall off the end once there are more than KEEP of them
        let mut closed: Vec<(String, u64)> = self.records.iter().filter(|(_, r)| r["state"] == "closed").map(|(i, r)| (i.clone(), r["at"].as_u64().unwrap_or(0))).collect();
        closed.sort_by_key(|c| std::cmp::Reverse(c.1));
        for (id, _) in closed.into_iter().skip(KEEP) {
            self.records.remove(&id);
        }
        let tmp = file().with_extension("json.tmp");
        if std::fs::write(&tmp, Value::Object(self.records.clone()).to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, file());
        }
    }

    /// Bring a card that is not live (hidden before a restart, or closed) back from its record: its spec, last data and spot.
    fn restore(&mut self, id: &str, target: &EventLoopWindowTarget<Ev>) -> Option<Value> {
        let r = self.records.get(id)?.clone();
        let mut opts: Opts = serde_json::from_value(r["opts"].clone()).unwrap_or_default();
        if let Some(a) = r["rect"].as_array() {
            if let (Some(x), Some(y)) = (a.first().and_then(|v| v.as_i64()), a.get(1).and_then(|v| v.as_i64())) {
                opts.at = Some([x as i32, y as i32]);   // back where it was
            }
        }
        let comps = r["components"].as_array().cloned().unwrap_or_default();
        let reply = self.show(Some(id.to_string()), opts, r["chip"].as_str().unwrap_or(""), comps, r["data"].clone(), r["sources"].clone(), target);
        let depth = r["depth"].as_u64().unwrap_or(if r["layer"] == "below" { 1 } else { 0 }) as usize;
        if depth > 0 {
            if let Some(s) = self.by_id.get(id).and_then(|k| self.surfaces.get_mut(k)) {
                s.depth = depth;   // applied when it appears (layout.rs)
            }
        }
        self.note_shown(&json!({"status": reply["status"], "id": id}));
        Some(reply)
    }

    /// `morc card <id> set {...}`: the wanted state, applied as the ops it needs.
    pub(super) fn card_set(&mut self, id: &str, want: &Value, target: &EventLoopWindowTarget<Ev>) -> Value {
        let at = want["at"].as_array().and_then(|a| Some([a.first()?.as_i64()? as i32, a.get(1)?.as_i64()? as i32]));
        let center = want["center"].as_bool().unwrap_or(false);
        let frac = want["frac"].as_f64();
        let mut last = json!({"ok": true});
        // the stack: "depth": n (0 = top), or "layer": "top" | "below" (= depth 1)
        let depth = match (want["depth"].as_u64(), want["layer"].as_str()) {
            (Some(d), _) => Some(d as usize),
            (None, Some("top")) => Some(0),
            (None, Some("below")) => Some(1),
            (None, Some(other)) => return json!({"ok": false, "err": format!("layer '{other}': below or top")}),
            (None, None) => None,
        };
        if let Some(d) = depth {
            match self.by_id.get(id).copied() {
                Some(k) => {
                    let got = self.set_depth(k, d);
                    last = json!({"ok": true, "id": id, "depth": got});
                }
                None => return json!({"ok": false, "err": format!("no card '{id}' on screen")}),
            }
        }
        match want["state"].as_str() {
            Some("hidden") => return self.card_op(id, "hide", None, false, None, target),
            Some("shown") => last = self.card_op(id, "show", None, false, None, target),
            Some("closed") => {
                if let Some(&k) = self.by_id.get(id) {
                    self.begin_leave(k, None);
                }
                return json!({"ok": true, "id": id, "state": "closed"});
            }
            Some(other) => return json!({"ok": false, "err": format!("state '{other}': shown, hidden or closed")}),
            None => {}
        }
        if !last["ok"].as_bool().unwrap_or(false) {
            return last;
        }
        if let Some(f) = frac {
            last = self.card_op(id, "size", None, center && at.is_none(), Some(f), target);
        }
        if at.is_some() || (center && frac.is_none()) {
            last = self.card_op(id, "move", at, center, None, target);
        }
        last
    }

    pub(super) fn card_op(&mut self, id: &str, op: &str, at: Option<[i32; 2]>, center: bool, frac: Option<f64>, target: &EventLoopWindowTarget<Ev>) -> Value {
        if op == "get" {
            return if self.records.contains_key(id) || self.by_id.contains_key(id) { self.card_json(id) } else { json!({"ok": false, "err": format!("no card '{id}' (morc cards lists them)")}) };
        }
        if !self.by_id.contains_key(id) && matches!(op, "show" | "front") {
            return self.restore(id, target).unwrap_or_else(|| json!({"ok": false, "err": format!("no card '{id}' to bring back")}));
        }
        let Some(&key) = self.by_id.get(id) else { return json!({"ok": false, "err": format!("no card '{id}' on screen (morc cards lists them)")}) };
        let (kin, foreign) = self.estates(key);
        let op_snap = op == "snap";
        let op = if op_snap { "move" } else { op };
        let mut snap_targets: Vec<placement::R> = placement::foreground().rect.into_iter().collect();
        snap_targets.extend(self.surfaces.values().filter(|o| o.key != key && o.phase == Phase::Visible).filter_map(|o| o.content_rect()));
        let Some(s) = self.surfaces.get_mut(&key) else { return json!({"ok": false, "err": "gone"}) };
        let live = matches!(s.phase, Phase::Visible | Phase::Hidden);
        if !live {
            return json!({"ok": false, "err": "the card is still opening or already leaving"});
        }
        let now = Instant::now();
        match op {
            "hide" => {
                if s.phase == Phase::Visible {
                    winstyle::hide(s.hwnd());
                    s.phase = Phase::Hidden;
                    s.phase_since = now;
                    s.mv = None;
                }
            }
            "show" | "front" => {
                if s.phase == Phase::Hidden {
                    s.phase = Phase::Visible;
                    s.phase_since = now;
                    s.life.restart(s.opts.ttl, s.opts.sticky || s.opts.urgent, now);
                }
                winstyle::show_topmost_noactivate(s.hwnd());
                if op == "front" && s.depth > 0 {
                    s.depth = 0;   // "bring it to the front" means on top again
                    s.send(&json!({"t": "layer", "below": false, "depth": 0}));
                }
                if s.depth > 0 {
                    if let Some(r) = s.content_rect() {
                        s.depth = winstyle::set_depth(s.hwnd(), s.depth, r);   // shown again, at its place in the stack
                    }
                }
                // hiding dropped the click-through shape: the page measures again and the host sets it back (layout.rs)
                let widgets: Vec<placement::R> = kin.into_iter().chain(foreign).collect();
                let view = view_for(s, &s.opts.clone(), &prefs::load(), &s.topic.clone(), &widgets);
                s.send(&json!({"t": "view", "view": view}));
            }
            "move" => {
                if s.phase != Phase::Visible {
                    return json!({"ok": false, "err": "hidden: show it first"});
                }
                let (Some(r), Some(c)) = (s.rect(), s.content_rect()) else { return json!({"ok": false, "err": "no window rect"}) };
                let to = if op_snap {
                    // `move --snap`: pull it onto the nearest edge from where it is now (the same magnets as a carry, a wider reach)
                    let scale = s.window.scale_factor();
                    let reach = (super::snap_setting("snap-px", 22.0).max(1.0) * 4.0 * scale) as i32;
                    let pol = policy::load();
                    let m = super::snap::Magnets { targets: &snap_targets, work: placement::work_at((c.l + c.r) / 2, (c.t + c.b) / 2), radius: reach, breakaway: reach, gap: (super::snap_setting("snap-gap", 12.0) * scale).round() as i32, margin: pol.margin };
                    let ((x, y), _) = super::snap::apply(&c, &m, Default::default());
                    (x - (c.l - r.l), y - (c.t - r.t))
                } else { match (at, center) {
                    (Some([x, y]), _) => (x - (c.l - r.l), y - (c.t - r.t)),
                    (None, true) => match centered(s) {
                        Some(p) => p,
                        None => return json!({"ok": false, "err": "no window rect"}),
                    },
                    (None, false) => return json!({"ok": false, "err": "move needs --at X,Y, --center or --snap"}),
                } };
                s.user_moved = true;   // placement must not move it back
                s.mv = Some(Move { from: (r.l, r.t), to, start: now });
            }
            "size" => {
                let Some(f) = frac else { return json!({"ok": false, "err": "size needs --frac F (0.2 .. 0.98)"}) };
                s.opts.max_frac = Some(f.clamp(0.2, 0.98));
                s.opts.fill = true;
                s.user_moved = true;
                s.recenter = center;
                let widgets: Vec<placement::R> = kin.into_iter().chain(foreign).collect();
                let view = view_for(s, &s.opts.clone(), &prefs::load(), &s.topic.clone(), &widgets);
                s.send(&json!({"t": "view", "view": view}));
            }
            "lower" | "raise" => {
                let want = if op == "lower" { s.depth + 1 } else { s.depth.saturating_sub(1) };
                let d = self.set_depth(key, want);
                return json!({"ok": true, "id": id, "layer": if d > 0 { "below" } else { "top" }, "depth": d});
            }
            other => return json!({"ok": false, "err": format!("unknown op '{other}' (get, set, hide, show, front, lower, raise, move, snap, size)")}),
        }
        let state = if s.phase == Phase::Hidden { "hidden" } else { "shown" };
        json!({"ok": true, "id": id, "state": state})
    }
}
