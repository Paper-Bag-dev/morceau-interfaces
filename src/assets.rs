//! Serves ui/ files to the webviews through the custom morc:// protocol (http://morc.localhost/ on Windows).
//! Read from disk on every request, so editing a component takes effect on the next surface with no rebuild.

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
        let mut names: Vec<String> = std::fs::read_dir(paths::ui_dir().join("plugins"))
            .map(|rd| rd.filter_map(|e| e.ok()).filter_map(|e| e.file_name().into_string().ok()).filter(|n| n.ends_with(".js")).collect())
            .unwrap_or_default();
        names.sort();
        return Response::builder().header("Content-Type", "application/json").header("Cache-Control", "no-store").body(Cow::Owned(serde_json::to_vec(&names).unwrap())).unwrap();
    }
    let bad = rel.split('/').any(|p| p == ".." || p.contains(':') || p.contains('\\'));
    let body = if bad { None } else { std::fs::read(paths::ui_dir().join(&rel)).ok() };
    match body {
        Some(b) => Response::builder()
            .header("Content-Type", mime(&rel))
            .header("Cache-Control", "no-store")
            .body(Cow::Owned(b))
            .unwrap(),
        None => Response::builder().status(404).body(Cow::Borrowed(&b""[..])).unwrap(),
    }
}
