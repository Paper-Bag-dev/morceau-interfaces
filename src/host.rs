//! The long-lived host: owns every surface, reacts to commands, page messages and the active-window changes.
//! Idle cost: the event loop sleeps (ControlFlow::Wait) when nothing is on screen.

use crate::policy;
use crate::regions;
use crate::{
    backdrop, cache, engagement,
    ev::{Ev, Proxy},
    fgwatch, handlers, inbox, literacy,
    lifetime::{Life, Outcome},
    paths, placed, placement, prefs,
    protocol::{Opts, Request},
    server, sources,
    surface::{Move, Phase, Surface},
    util, wake, winstyle,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::TcpListener;
use std::time::{Duration, Instant};
use tao::{
    dpi::{LogicalSize, PhysicalPosition},
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget},
};

const SPARE_KEEP: Duration = Duration::from_secs(180);
const MOVE_SECS: f64 = 0.24;

struct Parked {
    id: String,
    name: String,
    opts: Opts,
    init: Value,
}

struct Host {
    proxy: Proxy,
    surfaces: HashMap<u64, Surface>,
    by_id: HashMap<String, u64>,
    next_key: u64,
    parked: Vec<Parked>,
    recent: Vec<Parked>,
    last_use: Instant,
    quit: bool,
}

pub fn run() -> ! {
    let listener = match TcpListener::bind((paths::HOST, paths::PORT)) {
        Ok(l) => l,
        Err(_) => {
            eprintln!("morc host already running");
            std::process::exit(0);
        }
    };
    // Release builds abort on panic and have no console: write the reason to host.log first, so a crash is never silent.
    std::panic::set_hook(Box::new(|info| util::log(&format!("PANIC: {info}"))));
    let token = format!("{:016x}{:016x}", rand_u64(), rand_u64());
    let _ = std::fs::write(paths::token_file(), &token);

    let event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    server::spawn(listener, token, proxy.clone());
    sources::spawn(proxy.clone());
    crate::anticipate::spawn(proxy.clone());
    fgwatch::install(proxy.clone());
    crate::hotkeys::spawn(proxy.clone());
    winstyle::drag_cursor_off(); // in case a previous host died mid-carry with the move cursor installed
    cache::write_registry();
    util::log("host started");

    let mut host = Host { proxy, surfaces: HashMap::new(), by_id: HashMap::new(), next_key: 1, parked: vec![], recent: vec![], last_use: Instant::now(), quit: false };
    event_loop.run(move |event, target, flow| {
        if let Event::UserEvent(ev) = event {
            host.handle(ev, target);
        }
        host.tick(target);
        if host.quit {
            winstyle::drag_cursor_off(); // never leave the move cursor behind
        }
        *flow = if host.quit { ControlFlow::Exit } else { host.next_flow() };
    })
}

fn rand_u64() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    std::collections::hash_map::RandomState::new().build_hasher().finish()
}

fn merge_opts(over: &Opts, base: &Opts) -> Opts {
    Opts {
        ttl: over.ttl.or(base.ttl),
        sticky: over.sticky || base.sticky,
        urgent: over.urgent || base.urgent,
        anchor: over.anchor.clone().or_else(|| base.anchor.clone()),
        at: over.at.or(base.at),
        region: over.region.clone().or_else(|| base.region.clone()),
        align: over.align.clone().or_else(|| base.align.clone()),
        min: over.min.or(base.min),
        dx: over.dx.or(base.dx),
        dy: over.dy.or(base.dy),
        topic: over.topic.clone().or_else(|| base.topic.clone()),
        theme: over.theme.clone().or_else(|| base.theme.clone()),
        importance: over.importance.clone().or_else(|| base.importance.clone()),
        fill: over.fill || base.fill,
        max_frac: over.max_frac.or(base.max_frac),
        contrast: over.contrast.clone().or_else(|| base.contrast.clone()),
        lint: over.lint.clone(),
        auto: over.auto,
        force: over.force,
        prefetch: over.prefetch.clone().or_else(|| base.prefetch.clone()),
    }
}

impl Host {
    fn handle(&mut self, ev: Ev, target: &EventLoopWindowTarget<Ev>) {
        match ev {
            Ev::Cmd(req, reply) => {
                let r = self.cmd(req, target);
                if let Some(tx) = reply {
                    let _ = tx.send(r.to_string());
                }
            }
            Ev::Ipc(key, text) => self.ipc(key, &text),
            Ev::Foreground => self.on_foreground(),
        }
    }

    // ---------- commands ----------

    fn cmd(&mut self, req: Request, target: &EventLoopWindowTarget<Ev>) -> Value {
        match req {
            Request::Show { id, opts, components, data, sources } => self.show(id, opts, "", components, data, sources, target),
            Request::Use { name, id, opts, data } => {
                let Some(c) = cache::load(&name) else { return json!({"ok": false, "err": format!("no cached component '{name}'")}) };
                let base: Opts = serde_json::from_value(c["opts"].clone()).unwrap_or_default();
                let mut d = c["data"].clone();
                if d.is_null() {
                    d = json!({});
                }
                util::deep_merge(&mut d, &data);
                cache::bump_use(&name);
                let comps = c["components"].as_array().cloned().unwrap_or_default();
                self.show(id, merge_opts(&opts, &base), &name, comps, d, c["sources"].clone(), target)
            }
            Request::Patch { id, data, components, attach } => match self.by_id.get(&id).and_then(|k| self.surfaces.get(k)) {
                Some(s) => {
                    s.send(&json!({"t": "patch", "data": data, "components": components, "attach": attach}));
                    json!({"ok": true})
                }
                None => json!({"ok": false, "err": "no such surface (expired?)"}),
            },
            Request::Close { id } => match self.by_id.get(&id).copied() {
                Some(k) => {
                    self.begin_leave(k, None);
                    json!({"ok": true})
                }
                None => json!({"ok": true, "note": "not open"}),
            },
            Request::Save { name, desc, params, opts, components, data, sources } => match crate::lint::check(&components).as_slice() {
                p if !p.is_empty() && opts.lint.as_deref() != Some("off") => json!({"ok": false, "err": "dead controls: not cached", "problems": p}),
                _ => match cache::save(&name, &desc, &params, &opts, &components, &data, &sources) {
                    Ok(p) => json!({"ok": true, "path": p.display().to_string()}),
                    Err(e) => json!({"ok": false, "err": e}),
                },
            },
            Request::Summon => self.summon(target),
            Request::Act { id, action } => {
                let Some(s) = self.by_id.get(&id).and_then(|k| self.surfaces.get(k)) else { return json!({"ok": false, "err": "no such surface"}) };
                match handlers::run(&s.id, &s.topic, &action, &self.proxy) {
                    handlers::Effect::Patch(path, v) => s.send(&json!({"t": "patch", "data": {path: v}})),
                    handlers::Effect::Wake(p) => util::log(&format!("wake: {}", wake::maybe_wake(&p))),
                    handlers::Effect::None => {}
                }
                json!({"ok": true})
            }
            Request::DismissAll => {
                let keys: Vec<u64> = self.surfaces.values().filter(|s| matches!(s.phase, Phase::Visible | Phase::Waiting)).map(|s| s.key).collect();
                let n = keys.len();
                for k in keys {
                    self.begin_leave(k, Some(Outcome::Dismissed));
                }
                let parked = self.parked.len();
                self.parked.clear();
                json!({"ok": true, "dismissed": n, "parked_cleared": parked})
            }
            Request::List => json!({"ok": true, "components": cache::all().iter().map(|c| json!({"name": c["name"], "desc": c["desc"], "params": c["params"]})).collect::<Vec<_>>()}),
            Request::Chore { reason, note } => json!({"ok": true, "result": wake::chore(&reason, &note)}),
            Request::Space => regions::snapshot(&self.surfaces.values().filter(|s| s.phase == Phase::Visible).filter_map(|s| s.rect()).collect::<Vec<_>>()),
            Request::Status => json!({
                "ok": true,
                "surfaces": self.surfaces.values().filter(|s| s.phase != Phase::Spare).map(|s| json!({"id": s.id, "topic": s.topic, "visible": s.phase == Phase::Visible, "pinned": s.life.pinned, "rect": s.rect().map(|r| vec![r.l, r.t, r.r, r.b]), "theme_seen": s.opts.theme})).collect::<Vec<_>>(),
                "parked": self.parked.iter().map(|p| p.id.clone()).collect::<Vec<_>>(),
                "spare": self.surfaces.values().any(|s| s.phase == Phase::Spare),
                "claude_session_age_s": inbox::session_age_s(),
            }),
            Request::Shutdown => {
                self.quit = true;
                json!({"ok": true})
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn show(&mut self, id: Option<String>, mut opts: Opts, name: &str, components: Vec<Value>, data: Value, srcs: Value, target: &EventLoopWindowTarget<Ev>) -> Value {
        let topic = opts.topic.clone().unwrap_or_else(|| if name.is_empty() { "adhoc".into() } else { name.to_string() });
        let id = id.unwrap_or_else(|| if name.is_empty() { format!("s{}", self.next_key) } else { name.to_string() });

        // The no-dead-controls rule: refuse specs with buttons that do nothing visible (see lint.rs).
        if opts.lint.as_deref() != Some("off") {
            let problems = crate::lint::check(&components);
            if !problems.is_empty() {
                return json!({"ok": false, "err": "dead controls (every click must cause a visible change)", "problems": problems});
            }
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

        let mut payload = json!({"t": "init", "id": id, "components": components, "data": if data.is_null() { json!({}) } else { data },
                             "opts": {"urgent": opts.urgent, "sticky": opts.sticky || opts.urgent, "topic": topic, "theme": opts.theme}, "literacy": literacy::snapshot()});
        self.parked.retain(|p| p.id != id);
        sources::register(&id, &srcs);
        self.last_use = Instant::now();

        // Same id already on screen: refresh it in place.
        if let Some(&k) = self.by_id.get(&id) {
            if self.surfaces.get(&k).map(|s| s.phase != Phase::Leaving).unwrap_or(false) {
                let s = self.surfaces.get_mut(&k).unwrap();
                payload["view"] = view_for(s, &opts, &prefs, &topic);
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
        let s = self.surfaces.get_mut(&key).unwrap();
        s.id = id.clone();
        s.name = name.to_string();
        s.topic = topic.clone();
        s.opts = opts.clone();
        s.phase = Phase::Waiting;
        s.phase_since = Instant::now();
        s.user_moved = false;
        s.life = Life::new(opts.ttl, opts.sticky || opts.urgent, Instant::now());
        payload["view"] = view_for(s, &opts, &prefs, &topic);
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

    /// Take the warm spare, or build a surface cold.
    fn claim(&mut self, target: &EventLoopWindowTarget<Ev>) -> Result<u64, String> {
        if let Some(k) = self.surfaces.values().filter(|s| s.phase == Phase::Spare).map(|s| s.key).next() {
            return Ok(k);
        }
        self.spawn(target)
    }

    fn spawn(&mut self, target: &EventLoopWindowTarget<Ev>) -> Result<u64, String> {
        let key = self.next_key;
        self.next_key += 1;
        let s = Surface::create(target, key, self.proxy.clone())?;
        self.surfaces.insert(key, s);
        Ok(key)
    }

    // ---------- messages from pages ----------

    fn ipc(&mut self, key: u64, text: &str) {
        let Ok(m) = serde_json::from_str::<Value>(text) else { return };
        let now = Instant::now();
        match m["t"].as_str().unwrap_or("") {
            "ready" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.ready = true;
                    if let Some(init) = s.init.take() {
                        s.send(&init);
                    }
                }
            }
            "layout" => self.layout(key, &m),
            "hover" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.on_hover(m["v"].as_bool().unwrap_or(false), now);
                }
            }
            "pin" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.pinned = m["v"].as_bool().unwrap_or(false);
                    s.life.on_interact(now);
                }
            }
            "drag" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    if s.drag_started.is_some() {
                        return;
                    }
                    s.life.dragging = true;
                    s.mv = None;
                    let from = s.rect();
                    s.drag_from = from;
                    s.drag_started = Some(Instant::now());
                    // onStart: tell the agent a carry began ...
                    inbox::push(&s.id, &s.topic, "drag_start", &json!({"from": from.as_ref().map(placed::rect_json), "via": m["src"].as_str().unwrap_or("hold")}));
                    winstyle::drag_cursor_on(); // pointer becomes the move cursor for the whole carry
                    // The native move loop runs inside the message pump AFTER this returns, so the end is detected in tick()
                    // (mouse button released) rather than here.
                    let _ = s.window.drag_window();
                }
            }
            "kbd" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    let on = m["v"].as_bool().unwrap_or(false);
                    winstyle::set_noactivate(s.hwnd(), !on);
                    if on {
                        s.window.set_focus();
                    }
                    s.life.on_interact(now);
                }
            }
            "act" => {
                let (mut patch, mut wake_payload) = (None, None);
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.on_interact(now);
                    match handlers::run(&s.id, &s.topic, &m["action"], &self.proxy) {
                        handlers::Effect::Patch(path, v) => patch = Some((path, v)),
                        handlers::Effect::Wake(p) => wake_payload = Some(p),
                        handlers::Effect::None => {}
                    }
                    if let Some((path, v)) = patch {
                        s.send(&json!({"t": "patch", "data": {path: v}}));
                    }
                }
                if let Some(p) = wake_payload {
                    let r = wake::maybe_wake(&p);
                    util::log(&format!("wake: {r}"));
                    // a click that asked for background work but got nobody (agent off, rate limited): put the button back instead of leaving skeletons forever
                    if p["bg"].as_bool().unwrap_or(false) && !matches!(r, "headless-spawned" | "queued" | "dry-run") {
                        if let Some(s) = self.surfaces.get(&key) {
                            s.send(&json!({"t": "patch", "data": {"/state": "idle"}}));
                        }
                    }
                }
            }
            "close" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.engaged = true;
                }
                self.begin_leave(key, Some(Outcome::Engaged));
            }
            "literacy" => {
                if let Some(s) = self.surfaces.get_mut(&key) {
                    s.life.on_interact(now);
                    literacy::record(&s.id, &s.topic, m["kind"].as_str().unwrap_or(""), m["v"].as_str().unwrap_or(""));
                }
            }
            "dismiss" => self.begin_leave(key, Some(Outcome::Dismissed)),
            "left" => self.destroy(key),
            "log" => util::log(&format!("page[{key}]: {}", m["msg"])),
            _ => {}
        }
    }

    /// The page told us its content size and the shapes to keep clickable.
    fn layout(&mut self, key: u64, m: &Value) {
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
            let remembered = placed::get(&s.topic).filter(|&(rx, ry)| !placement::covers_foreground(&placement::R { l: rx, t: ry, r: rx + size.0, b: ry + size.1 }));
            let explicit = s.opts.at.map(|[ax, ay]| placement::clamp_inside(&placement::R { l: ax, t: ay, r: ax + size.0, b: ay + size.1 }));
            // Region placement (--region/--align/--dx/--dy, or anchor "auto"): the host divides the free space and puts the content in a region.
            let by_region = (s.opts.region.is_some() || s.opts.align.is_some() || s.opts.anchor.as_deref() == Some("auto"))
                .then(|| regions::Spec { region: s.opts.region.clone(), align: s.opts.align.clone().unwrap_or_default(), dx: s.opts.dx.unwrap_or(0), dy: s.opts.dy.unwrap_or(0), sacrifice: prefs::get(&prefs::load(), &s.topic, "overlap") >= 0.2 || is_important(&s.opts) })
                .and_then(|sp| regions::place(size, &sp, &kin.iter().chain(foreign.iter()).copied().collect::<Vec<_>>()))
                .map(|(rx, ry)| placement::clamp_inside(&placement::R { l: rx, t: ry, r: rx + size.0, b: ry + size.1 }));
            let explicit = explicit.or(by_region);
            let (x, y) = match (explicit, remembered) {
                (Some(p), _) => p,   // the agent did the math: --at X,Y wins
                (None, Some((rx, ry))) => placement::clamp_inside(&placement::R { l: rx, t: ry, r: rx + size.0, b: ry + size.1 }),
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
            winstyle::show_topmost_noactivate(s.hwnd());
            s.phase = Phase::Visible;
            s.phase_since = Instant::now();
            s.life.restart(s.opts.ttl, s.opts.sticky || s.opts.urgent, Instant::now());
            s.send(&json!({"t": "enter"}));
        }
    }

    // ---------- lifecycle ----------

    fn begin_leave(&mut self, key: u64, outcome: Option<Outcome>) {
        let Some(s) = self.surfaces.get_mut(&key) else { return };
        if s.phase == Phase::Leaving {
            return;
        }
        if let Some(o) = outcome {
            engagement::record(&s.topic, &s.name, s.opts.auto, o.as_str(), s.life.shown.elapsed().as_secs_f64());
            prefs::adapt(&s.topic); // passive feedback: nudges size/frequency for this topic after a clear run of outcomes
            if s.opts.urgent && o == Outcome::Stale {
                self.parked.push(Parked { id: s.id.clone(), name: s.name.clone(), opts: s.opts.clone(), init: s.last_init.clone() });
            }
        }
        // remember it for "undo dismiss" (Ctrl+Alt+Space with nothing on screen); keep the last 5
        if !s.id.is_empty() {
            self.recent.retain(|p| p.id != s.id);
            self.recent.push(Parked { id: s.id.clone(), name: s.name.clone(), opts: s.opts.clone(), init: s.last_init.clone() });
            if self.recent.len() > 5 {
                self.recent.remove(0);
            }
        }
        s.phase = Phase::Leaving;
        s.phase_since = Instant::now();
        s.send(&json!({"t": "leave"}));
    }

    fn destroy(&mut self, key: u64) {
        if let Some(s) = self.surfaces.remove(&key) {
            if !s.id.is_empty() && self.by_id.get(&s.id) == Some(&key) {
                self.by_id.remove(&s.id);
                sources::drop_surface(&s.id);
            }
            winstyle::hide(s.hwnd());
        }
    }

    fn summon(&mut self, target: &EventLoopWindowTarget<Ev>) -> Value {
        let parked = std::mem::take(&mut self.parked);
        let n = parked.len();
        for p in parked {
            let mut opts = p.opts.clone();
            opts.force = true;
            opts.anchor = Some("near_focus".into());
            let comps = p.init["components"].as_array().cloned().unwrap_or_default();
            self.show(Some(p.id), opts, &p.name, comps, p.init["data"].clone(), Value::Null, target);
        }
        // Nothing parked and nothing on screen: bring back the most recently closed one ("undo dismiss").
        let any_visible = self.surfaces.values().any(|s| s.phase == Phase::Visible);
        let mut restored_recent = false;
        if n == 0 && !any_visible {
            if let Some(p) = self.recent.pop() {
                let mut opts = p.opts.clone();
                opts.force = true;
                let comps = p.init["components"].as_array().cloned().unwrap_or_default();
                self.show(Some(p.id), opts, &p.name, comps, p.init["data"].clone(), Value::Null, target);
                restored_recent = true;
            }
        }
        // Everything visible: pull it next to what you're doing right now, with a pulse so you see where it went.
        let keys: Vec<u64> = self.surfaces.values().filter(|s| s.phase == Phase::Visible).map(|s| s.key).collect();
        for k in &keys {
            self.replace(*k, true);
            if let Some(s) = self.surfaces.get(k) {
                s.send(&json!({"t": "pulse"}));
            }
        }
        json!({"ok": true, "restored": n, "restored_recent": restored_recent, "moved": keys.len()})
    }

    /// Recompute where a surface should sit and glide there.
    fn replace(&mut self, key: u64, force: bool) {
        let (kin, foreign) = self.estates(key);
        let Some(s) = self.surfaces.get_mut(&key) else { return };
        let Some(r) = s.rect() else { return };
        // Never relocate while the user is interacting with it (a click can change the active window): wait ~2 s of calm.
        let interacting = s.life.last_touch.elapsed() < Duration::from_millis(2000) || s.life.hover;
        if !force && (interacting || s.user_moved || s.life.pinned || s.life.dragging || !placement::covers_foreground(&r)) {
            return;
        }
        let ow = 1.2 - prefs::get(&prefs::load(), &s.topic, "overlap");
        let (il, it, ir, ib) = s.inset;
        let (x, y) = placement::pick((r.w() - il - ir, r.h() - it - ib), s.opts.anchor.as_deref().unwrap_or("right"), scope_of(&s.topic), &kin, &foreign, ow, cohesion_for(s));
        let (tx, ty) = (x - il, y - it);
        if !force {
            // Stay put. Opening/closing the neighbour must not make surfaces hop: only a surface that is really covered moves, at most
            // once every 10 s, and only for a SMALL shift (within a quarter of the screen). A big jump across the screen is worse than
            // being partly covered, and a tiny one is just noise.
            let work = placement::foreground().work;
            let covered = placement::foreground().rect.map(|f| r.overlap(&f) as f64 / r.area().max(1) as f64).unwrap_or(0.0);
            let dist = (((tx - r.l).abs() as f64) / work.w().max(1) as f64).max(((ty - r.t).abs() as f64) / work.h().max(1) as f64);
            let cooling = s.auto_moved_at.map(|t| t.elapsed().as_secs_f64() < policy::load().auto_move.cooldown_s).unwrap_or(false);
            let am = policy::load().auto_move;
            if covered < am.min_cover || dist > am.max_dist || dist < am.min_dist || cooling {
                return;
            }
        }
        s.auto_moved_at = Some(Instant::now());
        s.mv = Some(Move { from: (r.l, r.t), to: (tx, ty), start: Instant::now() });
    }

    fn on_foreground(&mut self) {
        crate::worker::note_foreground(); // remember what you are working in (exe + title) for the chore agent
        let keys: Vec<u64> = self.surfaces.values().filter(|s| s.phase == Phase::Visible).map(|s| s.key).collect();
        for k in keys {
            self.replace(k, false);
        }
    }

    /// onEnd: the carry finished (mouse button released). Reports where it landed and remembers the spot for the topic.
    fn finish_drags(&mut self) {
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

    fn tick(&mut self, target: &EventLoopWindowTarget<Ev>) {
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

    fn next_flow(&self) -> ControlFlow {
        let now = Instant::now();
        if self.surfaces.values().any(|s| s.mv.is_some()) {
            ControlFlow::WaitUntil(now + Duration::from_millis(16))
        } else if self.surfaces.values().any(|s| s.drag_started.is_some()) {
            ControlFlow::WaitUntil(now + Duration::from_millis(30)) // watching for the mouse release
        } else if self.surfaces.values().any(|s| s.phase != Phase::Spare) {
            ControlFlow::WaitUntil(now + Duration::from_millis(250))
        } else {
            ControlFlow::WaitUntil(now + Duration::from_secs(20))
        }
    }
}

fn left_button_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}

/// Important or urgent surfaces may cover the active window; everything else keeps off it (unless nothing fits anywhere).
fn is_important(opts: &Opts) -> bool {
    opts.urgent || match opts.importance.as_deref() {
        Some("high") | Some("critical") => true,
        Some(x) => x.parse::<f64>().map(|v| v >= 1.2).unwrap_or(false),
        None => false,
    }
}

fn cohesion_for(s: &Surface) -> placement::Cohesion {
    let p = prefs::load();
    let pol = policy::load();
    // An explicit bottom/corner anchor is an intent: it overrides the pull toward sibling surfaces and the toast-zone avoidance,
    // so a long list can really use the space below instead of tucking under its siblings. Which anchors count is policy data.
    let explicit = s.opts.anchor.as_deref().map(|a| pol.explicit_anchors.iter().any(|e| e == a)).unwrap_or(false);
    placement::Cohesion {
        dock: prefs::get(&p, &s.topic, "dock"),
        harmony: prefs::get(&p, &s.topic, "harmony") * if explicit { pol.weights.explicit_harmony } else { 1.0 },
        cover: if is_important(&s.opts) { pol.weights.cover_important } else { pol.weights.cover_normal },
        toast: if explicit { pol.weights.toast_explicit } else { pol.weights.toast },
    }
}

/// A context scope is the first segment of the topic ("stats.glance" -> "stats"): related surfaces share a scope.
fn scope_of(topic: &str) -> &str {
    topic.split('.').next().unwrap_or(topic)
}

/// How this surface should be drawn: scale (importance x size_bias), the 1/4-screen cap, motion, boldness, simplicity.
fn view_for(s: &Surface, opts: &Opts, prefs: &prefs::Prefs, topic: &str) -> Value {
    let imp = match opts.importance.as_deref() {
        Some("low") => 0.9,
        Some("high") => 1.2,
        Some("critical") => 1.45,
        Some(x) => x.parse::<f64>().unwrap_or(1.0),
        None => 1.0,
    };
    let sf = s.window.scale_factor();
    let work = placement::foreground().work;
    // Width the page should WRAP to (css px, unzoomed) so the window fits the gap beside the neighbour instead of forcing a slab.
    // Skipped for important/urgent surfaces and for working windows (graphs/3D ask for max_frac > 0.5).
    // If the surface names a region (--region / anchor auto), size to THAT region, not to the gap beside the active window.
    let reg = (opts.region.is_some() || opts.anchor.as_deref() == Some("auto"))
        .then(|| regions::size_for(opts.region.as_deref(), prefs::get(prefs, topic, "overlap") >= 0.2 || is_important(opts), opts.min.map(|[w, h]| ((w as f64 * sf) as i32, (h as f64 * sf) as i32))))
        .flatten();
    let room = reg.map(|(w, _)| w as f64).unwrap_or(placement::dock_room() as f64) / sf;
    let strip_h = reg.map(|(_, h)| h as f64).unwrap_or(placement::dock_below() as f64);
    let big = opts.max_frac.unwrap_or(0.5) > 0.5;
    let wrap = if prefs::get(prefs, topic, "morph") >= 0.3 && !is_important(opts) && !big {
        if room >= 230.0 { (room - 14.0).clamp(230.0, 420.0) } else { 360.0 }   // no gap beside a neighbour: still wrap long text instead of forcing a slab
    } else {
        0.0
    };
    json!({
        "scale": (imp * prefs::get(prefs, topic, "size_bias")).clamp(0.5, 2.2),
        "fill": opts.fill,
        // default cap: 1/4 of the usable area = half the width x half the height; working windows (graphs) ask for more via max_frac
        "maxw": work.w() as f64 / sf * opts.max_frac.unwrap_or(0.5).clamp(0.3, 0.92),
        "maxh": work.h() as f64 / sf * opts.max_frac.unwrap_or(0.5).clamp(0.3, 0.92),
        "theme_default": prefs.theme.clone().unwrap_or_else(|| "coral".into()),
        "debug": std::env::var("MORC_DEBUG").is_ok(),
        "motion": prefs::get(prefs, topic, "motion"),
        "boldness": prefs::get(prefs, topic, "boldness"),
        "simplicity": prefs::get(prefs, topic, "simplicity"),
        // Width the page should WRAP to (css px, unzoomed) so the window fits the gap beside the neighbour instead of forcing a slab.
        // Skipped for important/urgent surfaces and for working windows (graphs/3D ask for max_frac > 0.5).
        "wrap": wrap,
        "dock_h": ((strip_h - 22.0) / sf).max(0.0),   // css px of height available (the named region, else the strip under the active window): a long list flows into it
        "dock": prefs::get(prefs, topic, "dock"),
        "morph": prefs::get(prefs, topic, "morph"),
        "harmony": prefs::get(prefs, topic, "harmony"),
        "dock_w": placement::dock_room() as f64 / sf,   // css px free beside the active window: the width a docked surface may use
    })
}

fn beep() {
    unsafe {
        let _ = windows::Win32::System::Diagnostics::Debug::MessageBeep(windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_STYLE(0x40));
    }
}

impl Host {
    /// Rects of my visible surfaces other than `key`, split into (same scope, different scope).
    fn estates(&self, key: u64) -> (Vec<placement::R>, Vec<placement::R>) {
        let me = self.surfaces.values().find(|s| s.key == key).map(|s| scope_of(&s.topic).to_string()).unwrap_or_default();
        let (mut kin, mut foreign) = (vec![], vec![]);
        for s in self.surfaces.values().filter(|s| s.key != key && s.phase == Phase::Visible) {
            if let Some(r) = s.rect() {
                if scope_of(&s.topic) == me { kin.push(r) } else { foreign.push(r) }
            }
        }
        (kin, foreign)
    }
}
