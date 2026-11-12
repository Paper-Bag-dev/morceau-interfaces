//! Serves ui/ files to the webviews through the custom morc:// protocol (http://morc.localhost/ on Windows).
//! Also serves your own plugins (<home>/plugins) at user-plugins/. Read from disk on every request, so editing a component takes effect on the next surface with no rebuild.

use crate::paths;
use std::borrow::Cow;
use wry::http::{Request, Response};

fn mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "glb" => "model/gltf-binary",
        "gltf" => "model/gltf+json",
        _ => "application/octet-stream",
    }
}

pub fn handle(req: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let mut rel = req.uri().path().trim_start_matches('/').to_string();
    if rel.is_empty() {
        rel = "index.html".into();
    }
    // Plugin discovery: the page asks which files are in ui/plugins/ and imports each (components, actions, anything that registers itself).
    if rel == "plugins/index.json" {
        let list = |dir: std::path::PathBuf| -> Vec<String> {
            let mut v: Vec<String> = std::fs::read_dir(dir)
                .map(|rd| rd.filter_map(|e| e.ok()).filter_map(|e| e.file_name().into_string().ok()).filter(|n| n.ends_with(".js")).collect())
                .unwrap_or_default();
            v.sort();
            v
        };
        // the repo's ui/plugins first, then yours (<home>/plugins, outside the repo: served as user-plugins/, listed as "user/<file>")
        let mut names = list(paths::ui_dir().join("plugins"));
        names.extend(list(paths::user_plugins_dir()).into_iter().map(|n| format!("user/{n}")));
        return Response::builder().header("Content-Type", "application/json").header("Cache-Control", "no-store").body(Cow::Owned(serde_json::to_vec(&names).unwrap())).unwrap();
    }
    let bad = rel.split('/').any(|p| p == ".." || p.contains(':') || p.contains('\\'));
    let file = match rel.strip_prefix("user-plugins/") {
        Some(r) => paths::user_plugins_dir().join(r),
        None => paths::ui_dir().join(&rel),
    };
    let body = if bad { None } else { std::fs::read(file).ok() };
    match body {
        Some(b) => Response::builder()
            .header("Content-Type", mime(&rel))
            .header("Cache-Control", "no-store")
            .body(Cow::Owned(b))
            .unwrap(),
        None => Response::builder().status(404).body(Cow::Borrowed(&b""[..])).unwrap(),
    }
}
