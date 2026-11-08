//! Live data without spending tokens: a surface declares `sources` (file + JSON pointer + interval) and this
//! thread patches the values into it. Claude never has to relay numbers it could not see anyway.
//!   "sources": {"/video": {"file": "%LOCALAPPDATA%/myapp/stats.json", "pointer": "/today/minutes", "every": 5}}

use crate::{ev::{Ev, Proxy}, protocol::Request, util};
use serde_json::{Map, Value};
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct Source {
    surface: String,
    path: String,
    file: String,
    pointer: String,
    every: Duration,
    last: Option<Instant>,
    prev: Option<Value>,
}

static SOURCES: Mutex<Vec<Source>> = Mutex::new(Vec::new());

pub fn register(surface: &str, sources: &Value) {
    let Some(map) = sources.as_object() else { return };
    let mut g = SOURCES.lock().unwrap();
    g.retain(|s| s.surface != surface);
    for (path, spec) in map {
        let Some(file) = spec["file"].as_str() else { continue };
        g.push(Source {
            surface: surface.to_string(),
            path: path.clone(),
            file: util::expand_env(file),
            pointer: spec["pointer"].as_str().unwrap_or("").to_string(),
            every: Duration::from_secs_f64(spec["every"].as_f64().unwrap_or(5.0).max(1.0)),
            last: None,
            prev: None,
        });
    }
}

pub fn drop_surface(surface: &str) {
    SOURCES.lock().unwrap().retain(|s| s.surface != surface);
}

/// Read one pointer out of a JSON file (shared with the anticipation triggers).
pub fn read_pointer(file: &str, pointer: &str) -> Option<Value> {
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(util::expand_env(file)).ok()?).ok()?;
    if pointer.is_empty() { Some(doc) } else { doc.pointer(pointer).cloned() }
}

pub fn spawn(proxy: Proxy) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let mut batches: Vec<(String, Map<String, Value>)> = vec![];
        {
            let mut g = SOURCES.lock().unwrap();
            let now = Instant::now();
            for s in g.iter_mut() {
                if s.last.map(|l| now.duration_since(l) < s.every).unwrap_or(false) {
                    continue;
                }
                s.last = Some(now);
                if let Some(v) = read_pointer(&s.file, &s.pointer) {
                    if s.prev.as_ref() != Some(&v) {
                        s.prev = Some(v.clone());
                        match batches.iter_mut().find(|(id, _)| *id == s.surface) {
                            Some((_, m)) => {
                                m.insert(s.path.clone(), v);
                            }
                            None => {
                                let mut m = Map::new();
                                m.insert(s.path.clone(), v);
                                batches.push((s.surface.clone(), m));
                            }
                        }
                    }
                }
            }
        }
        for (id, data) in batches {
            let _ = proxy.send_event(Ev::Cmd(Request::Patch { id, data, components: vec![], attach: Value::Null }, None));
        }
    });
}
