//! What a click can do on the Rust side. Specs can only name actions; they can never carry a command.
//! Anything that runs a program must be listed in actions.json (host-side allowlist, written by a human or Claude
//! outside the spec). Nothing here can run anything that is not listed in actions.json.
//!
//! Handled in the page (JS): set, toggle, copy, close.   Handled here: open, run, emit, wake.

use crate::{ev::{Ev, Proxy}, inbox, paths, protocol::Request, util};
use serde_json::{json, Value};
use std::process::{Command, Stdio};

pub enum Effect {
    None,
    /// Patch the surface: JSON pointer -> value
    Patch(String, Value),
    /// Ask the host to wake Claude with this payload
    Wake(Value),
}

pub fn run(surface: &str, topic: &str, action: &Value, proxy: &Proxy) -> Effect {
    match action["do"].as_str().unwrap_or("") {
        "open" => {
            let t = action["target"].as_str().unwrap_or("");
            let ok = t.starts_with("https://") || t.starts_with("http://") || std::path::Path::new(t).exists();
            if ok {
                let _ = util::hidden(Command::new("rundll32").args(["url.dll,FileProtocolHandler", t])).spawn();
            }
            Effect::None
        }
        "run" => {
            // Asynchronous: the surface keeps animating while the command runs. When it finishes, its stdout is parsed as JSON
            // (else kept as text) and patched into `into`, together with any `then` values (e.g. {"/stage":"report"} to reveal the
            // result). If it fails, `fail` values are patched instead (e.g. back to the idle state) so the UI never hangs on a loader.
            let id = action["id"].as_str().unwrap_or("");
            let allow: Value = std::fs::read_to_string(paths::actions_file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(json!({}));
            let Some(spec) = allow.get(id) else {
                return Effect::Patch(into(action), json!(format!("(action '{id}' is not in actions.json)")));
            };
            let argv: Vec<String> = spec["argv"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(util::expand_env)).collect()).unwrap_or_default();
            if argv.is_empty() {
                return Effect::None;
            }
            let cwd = spec["cwd"].as_str().map(util::expand_env);
            let (into_path, then, fail, surface_id, proxy) = (into(action), action["then"].clone(), action["fail"].clone(), surface.to_string(), proxy.clone());
            std::thread::spawn(move || {
                let mut cmd = Command::new(&argv[0]);
                cmd.args(&argv[1..]).stdout(Stdio::piped()).stderr(Stdio::piped());
                if let Some(c) = cwd {
                    cmd.current_dir(c);
                }
                let mut data = serde_json::Map::new();
                match util::hidden(&mut cmd).output() {
                    Ok(o) if o.status.success() => {
                        let text = String::from_utf8_lossy(&o.stdout).to_string();
                        let v = serde_json::from_str::<Value>(text.trim()).unwrap_or_else(|_| json!(text.trim().chars().take(4000).collect::<String>()));
                        data.insert(into_path, v);
                        if let Some(m) = then.as_object() {
                            data.extend(m.clone());
                        }
                    }
                    _ => {
                        if let Some(m) = fail.as_object() {
                            data.extend(m.clone());
                        }
                    }
                }
                let _ = proxy.send_event(Ev::Cmd(Request::Patch { id: surface_id, data, components: vec![], attach: Value::Null }, None));
            });
            Effect::None
        }
        "emit" => {
            inbox::push(surface, topic, "emit", &action["payload"]);
            Effect::None
        }
        "wake" => {
            inbox::push(surface, topic, "wake", &action["payload"]);
            // the agent needs to know which surface to patch: add it to the payload it is woken with
            let mut p = action["payload"].clone();
            if let Some(o) = p.as_object_mut() {
                o.entry("surface").or_insert_with(|| Value::String(surface.to_string()));
            }
            Effect::Wake(p)
        }
        _ => Effect::None,
    }
}

fn into(action: &Value) -> String {
    action["into"].as_str().unwrap_or("/_out").to_string()
}
