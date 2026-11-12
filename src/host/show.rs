//! Showing a card: the lint, the fullscreen hold, the annoyance vector, refresh-in-place or a new window, its first view. Split out of host/mod.rs.

use super::*;

impl Host {
    pub(super) fn show(&mut self, id: Option<String>, mut opts: Opts, name: &str, components: Vec<Value>, data: Value, srcs: Value, target: &EventLoopWindowTarget<Ev>) -> Value {
        let topic = opts.topic.clone().unwrap_or_else(|| if name.is_empty() { "adhoc".into() } else { name.to_string() });
        let id = id.unwrap_or_else(|| if name.is_empty() { format!("s{}", self.next_key) } else { name.to_string() });

        // The no-dead-controls rule: refuse specs with buttons that do nothing visible (see lint.rs).
        if opts.lint.as_deref() != Some("off") {
            let problems = crate::lint::check(&components);
            if !problems.is_empty() {
                return json!({"ok": false, "err": "dead controls (every click must cause a visible change)", "problems": problems});
            }
        }

        // A fullscreen app is active (a game, a video): there is no window to dock beside and any card would sit on top of it, so every push that is
        // not urgent or forced is HELD. A card already on screen may still be refreshed. (`--force` or an urgent ask shows it anyway.)
        if !opts.urgent && !opts.force && !self.by_id.contains_key(&id) && placement::foreground().fullscreen {
            return json!({"ok": true, "status": "deferred", "why": "a fullscreen app is active: held (urgent or --force shows it)"});
        }

        // The annoyance vector decides first: is this push welcome right now, and how big/long/loud may it be?
        let prefs = prefs::load();
        let pv = |n: &str| prefs::get(&prefs, &topic, n);
        if opts.auto && !opts.force {
            let defer = |why: &str| json!({"ok": true, "status": "deferred", "why": why});
            if pv("mute") > 0.5 {
                return defer("topic is muted in the annoyance vector");
            }
            if prefs::quiet_now(&prefs) {
                return defer("quiet hours");
            }
            if placement::fg_exe().map(|e| prefs::app_muted(&prefs, &e)).unwrap_or(false) {
                return defer("a muted app is in front");
            }
            if placement::foreground().fullscreen {
                return defer("fullscreen app in front");
            }
            let autos = self.surfaces.values().filter(|s| s.phase == Phase::Visible && s.opts.auto).count();
            if autos as f64 >= pv("max_visible") && !self.by_id.contains_key(&id) {
                return defer("enough auto surfaces already");
            }
        }
        opts.ttl = Some(opts.ttl.unwrap_or(crate::lifetime::DEFAULT_TTL) * pv("ttl_bias"));
        let may_beep = pv("sound") > 0.5;

        let mut data = if data.is_null() { json!({}) } else { data };
        if opts.persist {
            state::restore(&id, &mut data);   // the last snapshot wins over the spec's starting data
        }
        let mut payload = json!({"t": "init", "id": id, "components": components, "data": data,
                             "opts": {"urgent": opts.urgent, "pulse": matches!(opts.importance.as_deref(), Some("high") | Some("critical")), "sticky": opts.sticky || opts.urgent, "topic": topic, "theme": opts.theme, "align": opts.align, "size": opts.size, "idle": opts.idle, "actions": opts.actions, "persist": opts.persist}, "literacy": literacy::snapshot()});
        if self.root.is_none() {
            payload["shared"] = self.shared.clone();   // unlinked cards start from the host's copy of the shared store
        }
        payload["sources"] = srcs.clone(); // kept in last_init so a summoned or restored card is bound to its live data again
        self.parked.retain(|p| p.id != id);
        sources::register(&id, &srcs);
        self.last_use = Instant::now();

        // Same id already on screen: refresh it in place.
        if let Some(&k) = self.by_id.get(&id) {
            if self.surfaces.get(&k).map(|s| s.phase != Phase::Leaving).unwrap_or(false) {
                let (kin, foreign) = self.estates(k);
                let widgets: Vec<placement::R> = kin.into_iter().chain(foreign).collect();
                let s = self.surfaces.get_mut(&k).unwrap();
                payload["view"] = view_for(s, &opts, &prefs, &topic, &widgets);
                s.send(&payload);
                s.last_init = payload;
                s.opts = opts.clone();
                s.life.restart(opts.ttl, opts.sticky || opts.urgent, Instant::now());
                return json!({"ok": true, "status": "shown", "id": id, "note": "refreshed"});
            }
            self.destroy(k);
        }

        let key = match self.claim(target) {
            Ok(k) => k,
            Err(e) => return json!({"ok": false, "err": e}),
        };
        // the cards already on screen: the new card is sized to the room that is REALLY left (a third or a side, minus them), not the biggest empty one
        let (kin, foreign) = self.estates(key);
        let widgets: Vec<placement::R> = kin.into_iter().chain(foreign).collect();
        let s = self.surfaces.get_mut(&key).unwrap();
        s.id = id.clone();
        s.name = name.to_string();
        s.topic = topic.clone();
        s.opts = opts.clone();
        s.phase = Phase::Waiting;
        s.phase_since = Instant::now();
        s.user_moved = false;
        s.life = Life::new(opts.ttl, opts.sticky || opts.urgent, Instant::now());
        payload["view"] = view_for(s, &opts, &prefs, &topic, &widgets);
        s.last_init = payload.clone();
        if s.ready {
            s.send(&payload);
        } else {
            s.init = Some(payload);
        }
        self.by_id.insert(id.clone(), key);
        if opts.urgent && may_beep {
            beep();
        }
        // one step ahead: a card with a "show more" starts its background preparation the moment it appears
        if let Some(mut p) = opts.prefetch.clone() {
            if let Some(o) = p.as_object_mut() {
                o.insert("surface".into(), Value::String(id.clone()));
                o.insert("bg".into(), Value::Bool(true));
                o.entry("prefetch").or_insert(Value::Bool(true));
            }
            util::log(&format!("prefetch for {id}: {}", wake::maybe_wake(&p)));
        }
        json!({"ok": true, "status": "shown", "id": id})
    }
}

