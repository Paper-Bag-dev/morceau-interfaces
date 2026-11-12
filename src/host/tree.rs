//! The trees (docs/trees.md). Two kinds, never mixed on write:
//!   scene tree   host-owned: screen -> displays -> windows front to back -> cards as MOUNT nodes, plus `covers` edges (A above B, overlapping).
//!   card tree    page-owned: one per card, built by the page from its components and the DOM (ui/core/tree.js): ids, types, rects, flags.
//! `morc tree` = the scene; `morc tree <card>` = one card; `morc tree --all` = the scene with every card's tree mounted under its card node.
//! Card trees (and named-action results, host/state.rs) come back over ipc, so a query waits for the pages (TREE_WAIT at most) and answers with what arrived; a card that did not
//! answer is marked `"tree": "no reply"` instead of holding the reply.

use super::*;
use crate::placement::scene_now;
use std::sync::mpsc::Sender;

const TREE_WAIT: Duration = Duration::from_millis(1200);

pub(super) struct TreeQuery {
    tx: Option<Sender<String>>,
    started: Instant,
    /// None = a single card's tree is the whole answer; Some = the scene, with card trees mounted into it.
    scene: Option<Value>,
    single: Option<u64>,
    want: Vec<u64>,
    got: HashMap<u64, Value>,
}

impl Host {
    pub(super) fn tree(&mut self, id: Option<String>, depth: Option<u32>, node: Option<String>, all: bool, tx: Option<Sender<String>>) {
        let msg = json!({"t": "tree", "depth": depth, "node": node});
        if let Some(id) = id {
            return self.ask_card(&id, msg, tx);
        }
        let ask = |s: &Surface, rid: u64| {
            let mut m = msg.clone();
            m["rid"] = json!(rid);
            s.send(&m)
        };
        let rid = rand_u64() & ((1 << 52) - 1);   // a JS number holds 53 bits exactly: a bigger id comes back rounded and matches nothing
        let scene = self.scene();
        let want: Vec<u64> = if all { self.surfaces.values().filter(|s| s.phase == Phase::Visible && s.ready).map(|s| s.key).collect() } else { vec![] };
        if want.is_empty() {
            if let Some(tx) = tx {
                let _ = tx.send(json!({"ok": true, "scene": scene}).to_string());
            }
            return;
        }
        for k in &want {
            if let Some(s) = self.surfaces.get(k) {
                ask(s, rid);
            }
        }
        self.trees.insert(rid, TreeQuery { tx, started: Instant::now(), scene: Some(scene), single: None, want, got: HashMap::new() });
    }

    /// Send one card a request that the page answers (tree, do); the CLI reply is the page's answer. `msg` gets the reply id.
    pub(super) fn ask_card(&mut self, id: &str, mut msg: Value, tx: Option<Sender<String>>) {
        let Some(s) = self.by_id.get(id).and_then(|k| self.surfaces.get(k)).filter(|s| s.ready) else {
            if let Some(tx) = tx {
                let _ = tx.send(json!({"ok": false, "err": format!("no card '{id}' on screen")}).to_string());
            }
            return;
        };
        let rid = rand_u64() & ((1 << 52) - 1);   // a JS number holds 53 bits exactly: a bigger id comes back rounded and matches nothing
        msg["rid"] = json!(rid);
        s.send(&msg);
        self.trees.insert(rid, TreeQuery { tx, started: Instant::now(), scene: None, single: Some(s.key), want: vec![s.key], got: HashMap::new() });
    }

    /// A page answered (ipc `{"t":"reply","rid":..,"body":{ok, ...}}`).
    pub(super) fn reply_part(&mut self, key: u64, m: &Value) {
        let Some(rid) = m["rid"].as_u64() else { return };
        let Some(q) = self.trees.get_mut(&rid) else { return };
        q.got.insert(key, m["body"].clone());
        if q.want.iter().all(|k| q.got.contains_key(k)) {
            let q = self.trees.remove(&rid).unwrap();
            self.finish_tree(q);
        }
    }

    /// Called from tick: answer queries (trees, actions) whose pages are too slow with what arrived.
    pub(super) fn tree_timeouts(&mut self) {
        let late: Vec<u64> = self.trees.iter().filter(|(_, q)| q.started.elapsed() > TREE_WAIT).map(|(r, _)| *r).collect();
        for rid in late {
            let q = self.trees.remove(&rid).unwrap();
            self.finish_tree(q);
        }
    }

    fn finish_tree(&self, mut q: TreeQuery) {
        let out = match (q.single, q.scene.take()) {
            (Some(k), _) => q.got.remove(&k).unwrap_or_else(|| json!({"ok": false, "err": "the card did not answer"})),
            (None, Some(mut scene)) => {
                mount(&mut scene, &|card_id: &str| {
                    let key = self.by_id.get(card_id)?;
                    Some(q.got.get(key).map(|b| b["tree"].clone()).unwrap_or(json!("no reply")))
                });
                json!({"ok": true, "scene": scene})
            }
            (None, None) => json!({"ok": false, "err": "empty query"}),
        };
        if let Some(tx) = q.tx {
            let _ = tx.send(out.to_string());
        }
    }

    /// The scene tree, built fresh on every call (no live model to keep in sync): displays, then every window front to back on the display
    /// it overlaps most, our cards as mount nodes (their VISIBLE content rect, not the see-through window around it), and `covers` edges for each pair where the upper one overlaps the lower.
    fn scene(&self) -> Value {
        let (displays, wins) = scene_now();
        let cards: Vec<&Surface> = self.surfaces.values().filter(|s| s.phase == Phase::Visible).collect();
        // Front-to-back list of (node id, kind, rect, extra). A card is matched to its own window in the OS list by rect; if the OS list
        // has none of ours (Windows), cards go on top, which is where their floating level puts them.
        let mut stack: Vec<(String, Value, placement::R)> = vec![];
        let mut placed: Vec<u64> = vec![];
        for w in &wins {
            if w.mine {
                if let Some(s) = cards.iter().find(|s| !placed.contains(&s.key) && s.rect().map(|r| near(&r, &w.rect)).unwrap_or(false)) {
                    placed.push(s.key);
                    stack.push((format!("card:{}", s.id), card_node(s), s.content_rect().unwrap_or(w.rect)));
                }
                continue;
            }
            stack.push((format!("win:{}", w.id), json!({"kind": "window", "app": w.app}), w.rect));
        }
        let mut top: Vec<(String, Value, placement::R)> = cards.iter().filter(|s| !placed.contains(&s.key)).filter_map(|s| Some((format!("card:{}", s.id), card_node(s), s.content_rect()?))).collect();
        top.append(&mut stack);
        let stack = top;

        let mut covers = vec![];
        for (i, (a, _, ra)) in stack.iter().enumerate() {
            for (b, _, rb) in stack.iter().skip(i + 1) {
                let o = ra.overlap(rb);
                if o > 0 && (a.starts_with("card:") || b.starts_with("card:")) {
                    covers.push(json!({"above": a, "below": b, "px": o}));
                }
            }
        }
        let disp: Vec<Value> = displays
            .iter()
            .enumerate()
            .map(|(di, d)| {
                let kids: Vec<Value> = stack
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, _, r))| displays.iter().enumerate().max_by_key(|(_, dd)| dd.frame.overlap(r)).map(|(j, _)| j) == Some(di))
                    .map(|(z, (id, n, r))| {
                        let mut n = n.clone();
                        n["id"] = json!(id);
                        n["z"] = json!(z);
                        n["rect"] = json!([r.l, r.t, r.w(), r.h()]);
                        n
                    })
                    .collect();
                json!({"id": format!("display:{di}"), "kind": "display", "rect": [d.frame.l, d.frame.t, d.frame.w(), d.frame.h()], "work": [d.work.l, d.work.t, d.work.w(), d.work.h()], "children": kids})
            })
            .collect();
        json!({"id": "screen", "kind": "screen", "at": util::now_ms() as u64, "units": "physical px, [x, y, w, h], z 0 = front", "children": disp, "covers": covers})
    }
}

fn card_node(s: &Surface) -> Value {
    json!({"kind": "card", "card": s.id, "chip": s.name, "topic": s.topic, "pinned": s.life.pinned, "layer": if s.depth > 0 { "below" } else { "top" }, "depth": s.depth, "mount": true})
}

fn near(a: &placement::R, b: &placement::R) -> bool {
    (a.l - b.l).abs() <= 2 && (a.t - b.t).abs() <= 2 && (a.r - b.r).abs() <= 2 && (a.b - b.b).abs() <= 2
}

/// Put each card's tree under its mount node (`tree`), walking the scene.
fn mount(v: &mut Value, tree_of: &dyn Fn(&str) -> Option<Value>) {
    if v["mount"].as_bool() == Some(true) {
        if let Some(t) = v["card"].as_str().and_then(tree_of) {
            v["tree"] = t;
        }
    }
    if let Some(kids) = v.get_mut("children").and_then(|k| k.as_array_mut()) {
        for k in kids {
            mount(k, tree_of);
        }
    }
}
