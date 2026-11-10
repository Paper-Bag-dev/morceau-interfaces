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

mod uidata;
mod view;
mod crowd;
mod ipc;
mod layout;
mod show;
mod tick;
use uidata::{ui_bundle, ui_stamp};
use view::{cohesion_for, is_important, scope_of, view_for};

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
    /// when the last tick ran: while the user is away, every card's clock is held for the time between ticks
    last_tick: Instant,
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

    let mut event_loop = EventLoopBuilder::<Ev>::with_user_event().build();
    crate::platform::os::app::configure_event_loop(&mut event_loop);
    let proxy = event_loop.create_proxy();
    server::spawn(listener, token, proxy.clone());
    sources::spawn(proxy.clone());
    crate::anticipate::spawn(proxy.clone());
    // hot reload: when a token / theme / limits file changes, every card gets the new values within a couple of seconds, no restart
    {
        let p = proxy.clone();
        std::thread::spawn(move || {
            let mut last = ui_stamp();
            loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let now = ui_stamp();
                if now != last {
                    last = now;
                    let _ = p.send_event(Ev::Cmd(Request::Reload, None));
                }
            }
        });
    }
    fgwatch::install(proxy.clone());
    winstyle::watch_hover(proxy.clone());
    crate::hotkeys::spawn(proxy.clone());
    winstyle::drag_cursor_off(); // in case a previous host died mid-carry with the move cursor installed
    cache::write_registry();
    util::log("host started");

    let mut host = Host { proxy, surfaces: HashMap::new(), by_id: HashMap::new(), next_key: 1, parked: vec![], recent: vec![], last_use: Instant::now(), last_tick: Instant::now(), quit: false };
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
            Ev::Pointer(win, x, y) => {
                if let Some(s) = self.surfaces.values().find(|s| s.hwnd() == win) {
                    s.send(&json!({"t": "pointer", "x": x, "y": y}));
                }
            }
            Ev::Hover(win, on) => {
                if let Some(s) = self.surfaces.values().find(|s| s.hwnd() == win) {
                    s.send(&json!({"t": "hover", "on": on}));
                }
            }
        }
    }

    // ---------- commands ----------

    fn cmd(&mut self, req: Request, target: &EventLoopWindowTarget<Ev>) -> Value {
        match req {
            Request::Show { id, opts, components, data, sources } => self.show(id, opts, "", components, data, sources, target),
            Request::Use { name, id, opts, data } => {
                let Some(c) = cache::load(&name) else { return json!({"ok": false, "err": cache::explain_missing(&name)}) };
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
            Request::UiMap => {
                let b = ui_bundle();
                json!({"ok": true, "root": paths::ui_dir().display().to_string(), "map": b["components"]})
            }
            Request::Reload => {
                let ui = ui_bundle();
                for s in self.surfaces.values() {
                    s.send(&json!({"t": "ui", "ui": ui}));
                }
                json!({"ok": true})
            }
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
            self.show(Some(p.id), opts, &p.name, comps, p.init["data"].clone(), p.init["sources"].clone(), target);
        }
        // Nothing parked and nothing on screen: bring back the most recently closed one ("undo dismiss").
        let any_visible = self.surfaces.values().any(|s| s.phase == Phase::Visible);
        let mut restored_recent = false;
        if n == 0 && !any_visible {
            if let Some(p) = self.recent.pop() {
                let mut opts = p.opts.clone();
                opts.force = true;
                let comps = p.init["components"].as_array().cloned().unwrap_or_default();
                self.show(Some(p.id), opts, &p.name, comps, p.init["data"].clone(), p.init["sources"].clone(), target);
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
        placement::invalidate(); // a new active window: the next placement must look again
        crate::worker::note_foreground(); // remember what you are working in (exe + title) for the chore agent
        let keys: Vec<u64> = self.surfaces.values().filter(|s| s.phase == Phase::Visible).map(|s| s.key).collect();
        for k in keys {
            self.replace(k, false);
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

use crate::platform::os::sys::{beep, left_button_down};

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
