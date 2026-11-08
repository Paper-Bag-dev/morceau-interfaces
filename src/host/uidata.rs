//! The UI as data: tokens, limits, themes and the component map, read from small JSON files, merged with your overrides, cached in memory and re-sent when a file changes.
//! (Split out of host/mod.rs: this file owns reading and caching those files and nothing else.)

use crate::paths;
use serde_json::{json, Value};

/// The UI as data. Shipped in <install>/ui, overridden in <data>/prefs; all of it hot-reloads (see ui_stamp and Request::Reload):
///   tokens: one small JSON per component in ui/tokens/*.json (chart.json, text.json, plate.json ...), overridden by prefs/tokens/*.json. Keys become `--key`.
///   limits: ui/limits.json (min / max sizes and timings), themes: ui/themes.json (accents, base colors, schemes); both overridable under
///   "limits" / "themes" in prefs/ui.json. "about" keys are documentation. The result is cached in memory until a file changes.
pub(super) fn ui_files() -> Vec<std::path::PathBuf> {
    let mut v = vec![paths::ui_dir().join("limits.json"), paths::ui_dir().join("themes.json"), paths::ui_dir().join("components.json"), paths::prefs_dir().join("ui.json")];
    for dir in [paths::ui_dir().join("tokens"), paths::prefs_dir().join("tokens")] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut f: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map(|x| x == "json").unwrap_or(false)).collect();
            f.sort();
            v.extend(f);
        }
    }
    v
}

/// A number that changes whenever any UI data file is added, removed or edited.
pub(super) fn ui_stamp() -> u128 {
    ui_files().iter().filter_map(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok()).filter_map(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis()).sum::<u128>()
        + ui_files().len() as u128
}

pub(super) fn ui_bundle() -> Value {
    static CACHE: std::sync::Mutex<Option<(u128, Value)>> = std::sync::Mutex::new(None);
    let stamp = ui_stamp();
    if let Some((s, v)) = CACHE.lock().unwrap().as_ref() {
        if *s == stamp {
            return v.clone();
        }
    }
    let read = |p: std::path::PathBuf| std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str::<Value>(s.trim_start_matches('\u{feff}')).ok()).unwrap_or(json!({}));
    let strip = |mut v: Value| {
        if let Some(o) = v.as_object_mut() {
            o.remove("about");
        }
        v
    };
    let merge = |base: &mut Value, over: &Value| {
        if let (Some(b), Some(o)) = (base.as_object_mut(), over.as_object()) {
            for (k, v) in o {
                if k == "about" {
                    continue;
                }
                match (b.get_mut(k), v) {
                    (Some(Value::Object(bo)), Value::Object(vo)) => {
                        for (kk, vv) in vo {
                            bo.insert(kk.clone(), vv.clone());
                        }
                    }
                    _ => {
                        b.insert(k.clone(), v.clone());
                    }
                }
            }
        }
    };
    let mut tokens = json!({});
    for dir in [paths::ui_dir().join("tokens"), paths::prefs_dir().join("tokens")] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut f: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map(|x| x == "json").unwrap_or(false)).collect();
            f.sort();
            for p in f {
                merge(&mut tokens, &read(p));
            }
        }
    }
    let mut limits = strip(read(paths::ui_dir().join("limits.json")));
    let mut themes = strip(read(paths::ui_dir().join("themes.json")));
    let over = read(paths::prefs_dir().join("ui.json"));
    merge(&mut tokens, &over["tokens"]);
    merge(&mut limits, &over["limits"]);
    merge(&mut themes, &over["themes"]);
    // the component map (ui/components.json, written by `morc ui build`) is held in memory with the rest: the CLI pulls it with `morc uimap`
    let components = read(paths::ui_dir().join("components.json"));
    let bundle = json!({"tokens": tokens, "limits": limits, "themes": themes, "components": components});
    *CACHE.lock().unwrap() = Some((stamp, bundle.clone()));
    bundle
}

