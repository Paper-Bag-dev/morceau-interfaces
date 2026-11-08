//! `morc stream`: show a spec piece by piece, skeleton cards first. Split out of cli/mod.rs.

use super::*;

/// Placeholder size for a component that has not arrived yet: (width, height, variant). Containers get a small bar; their children get their own
/// skeletons the moment the container mounts.
pub(super) fn skeleton_size(c: &Value) -> (u64, u64, &'static str) {
    let num = |k: &str| c[k].as_u64();
    match c["type"].as_str().unwrap_or("") {
        "stat" => (190, 72, "stat"),
        "progress" => (240, 48, "bar"),
        "chart" => (num("width").unwrap_or(280), num("height").unwrap_or(110) + 30, "chart"),
        "graph" => (num("width").unwrap_or(460), num("height").unwrap_or(320) + 30, "chart"),
        "model3d" => (num("width").unwrap_or(560), num("height").unwrap_or(340) + 30, "chart"),
        "list" => (280, 5 * 34, "list"),
        "text" => if matches!(c["size"].as_str(), Some("lg" | "xl" | "hero")) { (190, 34, "text") } else { (160, 24, "text") },
        "button" => (96, 30, "btn"),
        "row" => (220, 34, "btn"),
        _ => (260, 60, "text"),
    }
}

/// morc stream [spec.json|-] [--delay ms] [--id X]   Build a surface piece by piece. The frame appears with a skeleton card for every piece still
/// to come (so nothing jumps), then each is replaced in place by the real component, breadth-first from `root`, fading in; a small "building"
/// indicator fades out when done. Same spec format as `show`. The whole spec is linted up front because patches skip the host's show-time check.
pub(super) fn stream(rest: &[String]) -> i32 {
    use std::collections::{HashMap, VecDeque};
    let r: Result<(), String> = (|| {
        let file = rest.first().filter(|a| !a.starts_with("--"));
        let spec = read_input(file)?;
        let comps = spec["components"].as_array().ok_or("spec has no components")?.clone();
        if spec["opts"]["lint"] != "off" {
            let p = crate::lint::check(&comps);
            if !p.is_empty() {
                return Err(format!("dead controls: {}", p.join(" | ")));
            }
        }
        let by_id: HashMap<String, &Value> = comps.iter().filter_map(|c| Some((c["id"].as_str()?.to_string(), c))).collect();
        let root_id = if by_id.contains_key("root") { "root".to_string() } else { comps.first().and_then(|c| c["id"].as_str()).ok_or("empty spec")?.to_string() };
        let kids_of = |id: &str| -> Vec<String> { by_id.get(id).and_then(|c| c["children"].as_array()).map(|a| a.iter().filter_map(|k| k.as_str().map(String::from)).collect()).unwrap_or_default() };
        // a placeholder for `cid`, unless the component is conditional (`when`), which just appears when its turn comes
        let skel = |cid: &str| -> Option<Value> {
            let c = by_id.get(cid)?;
            if c.get("when").is_some() { return None; }
            let (w, h, v) = skeleton_size(c);
            Some(json!({"id": format!("__sk_{cid}"), "type": "skeleton", "w": w, "h": h, "variant": v}))
        };
        let mut order: Vec<(String, String)> = vec![];
        let mut q = VecDeque::from([root_id.clone()]);
        while let Some(id) = q.pop_front() {
            for k in kids_of(&id) {
                order.push((k.clone(), id.clone()));
                q.push_back(k);
            }
        }
        let delay = flag(rest, "--delay").and_then(|d| d.parse::<u64>().ok()).unwrap_or(200);
        let sid = flag(rest, "--id").or_else(|| spec["id"].as_str().map(String::from)).unwrap_or_else(|| "stream".into());

        // first paint: the root, a skeleton for each of its children, and the "building" indicator
        let mut root = by_id[&root_id].clone();
        let root_skels: Vec<Value> = kids_of(&root_id).iter().filter_map(|k| skel(k)).collect();
        let mut root_children: Vec<Value> = root_skels.iter().map(|s| s["id"].clone()).collect();
        root_children.push(json!("__build"));
        root["children"] = json!(root_children);
        let building = json!({"id": "__build", "type": "spinner", "label": spec["building"].as_str().unwrap_or("Building"), "when": {"path": "/__building", "eq": true}});
        let mut first: Vec<Value> = vec![root];
        first.extend(root_skels);
        first.push(building);
        let mut data = if spec["data"].is_object() { spec["data"].clone() } else { json!({}) };
        data["__building"] = json!(true);
        let mut opts = if spec["opts"].is_object() { spec["opts"].clone() } else { json!({}) };
        let need = 25.0 + order.len() as f64 * (delay as f64 / 1000.0);
        opts["ttl"] = json!(opts["ttl"].as_f64().unwrap_or(15.0).max(need));

        ensure_host()?;
        let ok = |v: Value| if v["ok"].as_bool().unwrap_or(false) { Ok(v) } else { Err(v.to_string()) };
        ok(send(json!({"t": "show", "id": sid, "opts": opts, "components": first, "data": data, "sources": spec["sources"]}))?)?;
        std::thread::sleep(Duration::from_millis(delay * 2));   // let the skeleton frame register before the first swap
        for (cid, parent) in &order {
            let mut c = by_id[cid].clone();
            let heavy = matches!(c["type"].as_str(), Some("chart" | "graph" | "list" | "model3d"));
            let child_skels: Vec<Value> = kids_of(cid).iter().filter_map(|k| skel(k)).collect();
            if c["children"].is_array() {
                c["children"] = json!(child_skels.iter().map(|s| s["id"].clone()).collect::<Vec<_>>());   // its children swap in as their own steps
            }
            let attach = if skel(cid).is_some() { json!({"parent": parent, "replace": format!("__sk_{cid}")}) }
                else if *parent == root_id { json!({"parent": root_id, "before": "__build"}) } else { json!({"parent": parent}) };
            let mut batch = vec![c];
            batch.extend(child_skels);
            ok(send(json!({"t": "patch", "id": sid, "components": batch, "attach": attach}))?)?;
            std::thread::sleep(Duration::from_millis(if heavy { delay * 3 / 2 } else { delay }));
        }
        ok(send(json!({"t": "patch", "id": sid, "data": {"/__building": false}}))?)?;
        println!("{}", json!({"ok": true, "id": sid, "steps": order.len()}));
        Ok(())
    })();
    match r {
        Ok(()) => 0,
        Err(e) => { eprintln!("morc stream: {e}"); 1 }
    }
}

