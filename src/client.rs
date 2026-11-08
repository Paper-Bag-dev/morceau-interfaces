//! The CLI side: `morc <command>`. Talks to the host over localhost, starting it first if it isn't running.
//!
//!   morc show [file|-]                     show a spec: {id?, opts?, components, data?, sources?}
//!   morc save [file|-]                     cache a component: {name, desc, params?, opts?, components, data?, sources?}
//!   morc use <name> [--data JSON|@file.json] [--id X] [--ttl N] [--sticky] [--urgent] [--topic T] [--auto]
//!                 [--region right|below|left|above|top-right|bottom-right|bottom-left|top-left|auto] [--align top-right|middle|...]
//!                 [--dx N --dy N] [--at X,Y] [--anchor A] [--importance low|high|critical|N] [--max-frac F]
//!   morc stream [file|-] [--delay MS]      show a spec piece by piece: skeleton cards first, then each component fades in
//!   morc patch <id> <pointer> <value> ...  change data on a live surface; value is JSON if it parses, else a string
//!   morc act <id> <action.json>            run an action on a surface as if it had been clicked
//!   morc graph [--file g.json | exporter args] [--title T]   show a node-link graph (docs/graphs.md)
//!   morc close <id> | dismiss-all | summon | list | status | stop
//!   morc space                             JSON: the free regions around the active window (see also `morc free`)
//!   morc uimap                             JSON: the UI component map (paths, types, deps, tokens) from the host's memory
//!   morc annoy [show|set|nudge|quiet|mute-app|mute-topic|theme ...]   the preference vector (docs/agent.md)
//!   morc chore <reason> [--note JSON]      ask for a gated wake of the background chore agent
//!   morc registry                          rewrite cache/registry.md
//!   morc inbox [--peek]                    unread UI interactions meant for the agent (JSON lines)
//!   morc hook <EventName>                  for Claude Code hooks: prints additionalContext JSON if there is anything unread
//!   morc commands                          list the CLI plugins (tools/cli) with one-line descriptions
//!   morc <plugin> ...                      any script in tools/cli: init, compose, cache, wake, place, free, sketchfab, bike

use crate::{cache, inbox, paths};
use serde_json::{json, Map, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

fn read_input(arg: Option<&String>) -> Result<Value, String> {
    let mut s = String::new();
    match arg.map(|a| a.as_str()) {
        None | Some("-") => {
            std::io::stdin().read_to_string(&mut s).map_err(|e| e.to_string())?;
        }
        Some(p) => s = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?,
    }
    serde_json::from_str(s.trim_start_matches('\u{feff}')).map_err(|e| format!("bad JSON: {e}"))
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}
fn has(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn send(req: Value) -> Result<Value, String> {
    let token = std::fs::read_to_string(paths::token_file()).map_err(|_| "no token".to_string())?;
    let mut s = TcpStream::connect((paths::HOST, paths::PORT)).map_err(|e| e.to_string())?;
    s.set_read_timeout(Some(Duration::from_secs(8))).ok();
    writeln!(s, "{}", json!({"token": token.trim(), "req": req})).map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| format!("bad reply: {e}"))
}

/// Make sure a host is up, starting a hidden detached one if needed.
fn ensure_host() -> Result<(), String> {
    if TcpStream::connect_timeout(&format!("{}:{}", paths::HOST, paths::PORT).parse().unwrap(), Duration::from_millis(200)).is_ok() && paths::token_file().exists() {
        return Ok(());
    }
    use std::os::windows::process::CommandExt;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // The detached host must not inherit the pipes this CLI was started with. Windows gives a child EVERY inheritable handle of its parent,
    // not just the std ones we null below, so a shell, a script or an agent capturing this CLI's output would wait for the host to exit
    // (it never does) and the first command would appear to hang for minutes. Mark our own std handles non-inheritable before spawning.
    unsafe {
        use windows::Win32::Foundation::{SetHandleInformation, HANDLE_FLAGS, HANDLE_FLAG_INHERIT};
        use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
        for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            if let Ok(h) = GetStdHandle(id) {
                if !h.is_invalid() {
                    let _ = SetHandleInformation(h, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0));
                }
            }
        }
    }
    std::process::Command::new(exe)
        .arg("host")
        .creation_flags(0x0800_0008) // CREATE_NO_WINDOW | DETACHED_PROCESS
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let t0 = Instant::now();
    while t0.elapsed() < Duration::from_secs(6) {
        if TcpStream::connect_timeout(&format!("{}:{}", paths::HOST, paths::PORT).parse().unwrap(), Duration::from_millis(200)).is_ok() && paths::token_file().exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("host did not start".into())
}

/// The annoyance vector: `morc annoy` shows it; the rest change it (and log why). Run when the user complains.
///   morc annoy set <param> <value> [--topic T] [--why "..."]     absolute
///   morc annoy nudge <param> <factor> [--topic T] [--why "..."]  multiply, e.g. size_bias 0.85 = "a bit smaller"
///   morc annoy quiet <HH:MM> <HH:MM> | off [--why ...]            no auto pushes in that window
///   morc annoy mute-app <exe> | unmute-app <exe> [--why ...]     no auto pushes while that app is in front
///   morc annoy mute-topic <topic> | unmute-topic <topic>
fn annoy(rest: &[String]) -> i32 {
    use crate::prefs;
    let why = flag(rest, "--why").unwrap_or_default();
    let topic = flag(rest, "--topic");
    let r: Result<String, String> = (|| match rest.first().map(|s| s.as_str()) {
        None | Some("show") => Ok(prefs::describe().to_string()),
        Some("set") | Some("nudge") => {
            let (name, val) = (rest.get(1).ok_or("annoy set|nudge <param> <value>")?, rest.get(2).ok_or("missing value")?.parse::<f64>().map_err(|e| e.to_string())?);
            let v = if rest[0] == "set" { prefs::set(topic.as_deref(), name, val, "user", &why)? } else { prefs::nudge(topic.as_deref(), name, val, "user", &why)? };
            Ok(format!("{{\"ok\":true,\"param\":\"{name}\",\"value\":{v}}}"))
        }
        Some("quiet") => {
            let range = match rest.get(1).map(|s| s.as_str()) { Some("off") => None, Some(a) => Some((a.to_string(), rest.get(2).ok_or("quiet <HH:MM> <HH:MM>")?.clone())), None => return Err("quiet <HH:MM> <HH:MM> | off".into()) };
            prefs::set_quiet(range, &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some("theme") => {
            prefs::set_theme(rest.get(1).ok_or("annoy theme <coral|ocean|violet|rose|amber|emerald|lime|sky|fuchsia|teal|auto>")?, &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some(c @ ("mute-app" | "unmute-app")) => {
            prefs::mute_app(rest.get(1).ok_or("need an exe name, e.g. chrome.exe")?, c == "mute-app", &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some(c @ ("mute-topic" | "unmute-topic")) => {
            let t = rest.get(1).ok_or("need a topic")?;
            prefs::set(Some(t), "mute", if c == "mute-topic" { 1.0 } else { 0.0 }, "user", &why)?;
            Ok("{\"ok\":true}".into())
        }
        Some(o) => Err(format!("unknown annoy command '{o}'")),
    })();
    match r {
        Ok(s) => { println!("{s}"); 0 }
        Err(e) => { eprintln!("morc annoy: {e}"); 2 }
    }
}

/// Extension scripts live in tools/cli/. They get MORC_BIN (this exe) and MORC_HOME, so they can call back into morc to show things.
fn external(cmd: &str) -> Option<std::path::PathBuf> {
    if cmd.is_empty() || !cmd.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    [paths::home_dir(), paths::app_dir()].iter().flat_map(|root| ["py", "ps1", "cmd"].iter().map(move |e| root.join("tools").join("cli").join(format!("{cmd}.{e}")))).find(|p| p.exists())
}

fn run_external(p: &std::path::Path, args: &[String]) -> i32 {
    let mut c = match p.extension().and_then(|e| e.to_str()) {
        Some("py") => { let mut c = std::process::Command::new("py"); c.arg(p); c }
        Some("ps1") => { let mut c = std::process::Command::new("powershell"); c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"]).arg(p); c }
        _ => { let mut c = std::process::Command::new("cmd"); c.arg("/c").arg(p); c }
    };
    c.args(args).env("MORC_HOME", paths::home_dir()).env("MORC_APP", paths::app_dir());
    if let Ok(me) = std::env::current_exe() {
        c.env("MORC_BIN", me);
    }
    match c.status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => { eprintln!("morc: could not run {}: {e}", p.display()); 1 }
    }
}

/// (name, description) of each extension; the description is the first line containing "morc:" (e.g. `# morc: what it does`).
fn external_list() -> Vec<(String, String)> {
    // your own tools (data folder) and the shipped ones (install folder); a tool in your folder shadows a shipped one of the same name
    let mut v: Vec<(String, String)> = vec![];
    for root in [paths::home_dir(), paths::app_dir()] {
        let Ok(rd) = std::fs::read_dir(root.join("tools").join("cli")) else { continue };
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            let Some(name) = p.file_stem().map(|s| s.to_string_lossy().to_string()) else { continue };
            if !matches!(p.extension().and_then(|x| x.to_str()), Some("py" | "ps1" | "cmd")) || name.starts_with('_') || v.iter().any(|(n, _)| *n == name) {
                continue;
            }
            // description: the first of the first lines that reads "# morc: ..." or "# morc <name>: ..."
            let desc = std::fs::read_to_string(&p).ok().and_then(|t| t.lines().take(6).find(|l| l.trim_start().starts_with("# morc") && l.contains(':')).map(|l| l.split_once(':').map(|x| x.1).unwrap_or("").trim().to_string())).unwrap_or_default();
            v.push((name, desc));
        }
    }
    v.sort();
    v
}

/// Placeholder size for a component that has not arrived yet: (width, height, variant). Containers get a small bar; their children get their own
/// skeletons the moment the container mounts.
fn skeleton_size(c: &Value) -> (u64, u64, &'static str) {
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
fn stream(rest: &[String]) -> i32 {
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
fn parse_value(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.to_string()))
}

pub fn run(args: Vec<String>) -> i32 {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let rest = &args[1.min(args.len())..];

    // Commands that never need the host.
    match cmd {
        "registry" => {
            cache::write_registry();
            println!("{}", paths::cache_dir().join("registry.md").display());
            return 0;
        }
        "inbox" => {
            for e in inbox::unread(has(rest, "--peek")) {
                println!("{e}");
            }
            return 0;
        }
        "hook" => {
            let out = inbox::hook_json(rest.first().map(|s| s.as_str()).unwrap_or("UserPromptSubmit"));
            if !out.is_empty() {
                println!("{out}");
            }
            return 0;
        }
        "annoy" => return annoy(rest),
        "stream" => return stream(rest),
        "commands" => {
            for (n, d) in external_list() {
                println!("{n:<16}{d}");
            }
            return 0;
        }
        "help" | "--help" | "-h" => {
            // the help text is the `//!   morc ...` block at the top of this file, so it cannot drift from the code beside it
            println!("{}", include_str!("client.rs").lines().take_while(|l| l.starts_with("//!")).skip(2).map(|l| l.trim_start_matches("//!").trim_end()).collect::<Vec<_>>().join("\n"));
            return 0;
        }
        _ => {}
    }

    // Extension point: any script in tools/cli/ (<name>.py / .ps1 / .cmd) is a subcommand `morc <name> ...` (built-ins win).
    const BUILTIN: &[&str] = &["show", "save", "use", "patch", "act", "close", "summon", "list", "status", "stop", "dismiss-all", "graph", "registry", "inbox", "hook", "annoy", "help", "stream", "commands", "space", "chore"];
    if !BUILTIN.contains(&cmd) {
        if let Some(p) = external(cmd) {
            return run_external(&p, rest);
        }
    }

    let req: Result<Value, String> = (|| match cmd {
        "show" => {
            let mut v = read_input(rest.first())?;
            v["t"] = json!("show");
            Ok(v)
        }
        "save" => {
            let mut v = read_input(rest.first())?;
            v["t"] = json!("save");
            Ok(v)
        }
        "use" => {
            let name = rest.first().ok_or("use <name>")?;
            let mut opts = Map::new();
            if let Some(t) = flag(rest, "--ttl").and_then(|t| t.parse::<f64>().ok()) { opts.insert("ttl".into(), json!(t)); }
            if has(rest, "--sticky") { opts.insert("sticky".into(), json!(true)); }
            if has(rest, "--urgent") { opts.insert("urgent".into(), json!(true)); }
            if has(rest, "--auto") { opts.insert("auto".into(), json!(true)); }
            if let Some(a) = flag(rest, "--anchor") { opts.insert("anchor".into(), json!(a)); }
            if let Some(a) = flag(rest, "--region") { opts.insert("region".into(), json!(a)); }   // right|below|left|above|mid|gap|auto
            if let Some(a) = flag(rest, "--align") { opts.insert("align".into(), json!(a)); }     // top-right, bottom, middle ...
            if let Some(n) = flag(rest, "--dx").and_then(|t| t.parse::<i32>().ok()) { opts.insert("dx".into(), json!(n)); }
            if let Some(n) = flag(rest, "--dy").and_then(|t| t.parse::<i32>().ok()) { opts.insert("dy".into(), json!(n)); }
            if let Some(a) = flag(rest, "--at") {   // --at X,Y : exact spot (physical px), the agent's own placement math
                let v: Vec<i32> = a.split(',').filter_map(|n| n.trim().parse().ok()).collect();
                if v.len() == 2 { opts.insert("at".into(), json!([v[0], v[1]])); }
            }
            if let Some(a) = flag(rest, "--topic") { opts.insert("topic".into(), json!(a)); }
            if let Some(a) = flag(rest, "--theme") { opts.insert("theme".into(), json!(a)); }
            if let Some(a) = flag(rest, "--importance") { opts.insert("importance".into(), json!(a)); }
            if let Some(a) = flag(rest, "--contrast") { opts.insert("contrast".into(), json!(a)); }
            if has(rest, "--fill") { opts.insert("fill".into(), json!(true)); }
            if let Some(f) = flag(rest, "--max-frac").and_then(|t| t.parse::<f64>().ok()) { opts.insert("max_frac".into(), json!(f)); }
            // --data accepts inline JSON, @file (a JSON file) or @- (stdin). Windows shells and launchers mangle the quotes in inline JSON, so a file is the reliable form.
            let data: Value = match flag(rest, "--data") {
                None => json!({}),
                Some(d) => match d.strip_prefix('@') {
                    Some(path) => read_input(Some(&path.to_string()))?,
                    None => serde_json::from_str(&d).map_err(|e| format!("--data: {e} (on Windows pass a file instead: --data @data.json)"))?,
                },
            };
            Ok(json!({"t": "use", "name": name, "id": flag(rest, "--id"), "opts": opts, "data": data}))
        }
        "patch" => {
            // morc patch <id> [pointer value ...] [--components file.json]   (components are upserted by id: how the agent streams
            // new visuals into a surface that is already on screen, e.g. the "ext" slot of a report)
            let id = rest.first().ok_or("patch <id> <pointer> <value> ... [--components file.json]")?;
            let mut components = Value::Array(vec![]);
            let mut args: Vec<&String> = rest[1..].iter().collect();
            if let Some(i) = args.iter().position(|a| a.as_str() == "--components") {
                let f = args.get(i + 1).ok_or("--components needs a file")?;
                components = read_input(Some(*f))?;
                args.drain(i..=i + 1);
            }
            let mut data = Map::new();
            let mut it = args.into_iter();
            while let (Some(p), Some(v)) = (it.next(), it.next()) {
                data.insert(p.clone(), parse_value(v));
            }
            Ok(json!({"t": "patch", "id": id, "data": data, "components": components}))
        }
        "act" => {
            let id = rest.first().ok_or("act <id> <action.json>")?;
            Ok(json!({"t": "act", "id": id, "action": read_input(rest.get(1))?}))
        }
        "close" => Ok(json!({"t": "close", "id": rest.first().ok_or("close <id>")?})),
        "summon" => Ok(json!({"t": "summon"})),
        "dismiss-all" => Ok(json!({"t": "dismiss_all"})),
        "graph" => {
            // morc graph [--file graph.json | <exporter args>] [--title T] [--ttl N] [--id X]
            // Without --file it runs tools/graph_export.py (code-graph index -> graph schema, see docs/graphs.md) with the same
            // arguments (--root, --langs, --level, --flow, --symbol, --depth, --title) and shows the result in `graph-view`.
            let graph: Value = if let Some(f) = flag(rest, "--file") {
                read_input(Some(&f))?
            } else {
                let mut args: Vec<String> = vec!["-I".into(), paths::app_dir().join("tools").join("graph_export.py").display().to_string()];
                let mut it = rest.iter();
                while let Some(a) = it.next() {
                    if a == "--ttl" || a == "--id" { it.next(); continue; }
                    args.push(a.clone());
                }
                let out = crate::util::hidden(std::process::Command::new("py").args(&args).current_dir(paths::home_dir()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()))
                    .output().map_err(|e| format!("py: {e}"))?;
                if !out.status.success() {
                    return Err(format!("graph_export failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
                }
                serde_json::from_slice(&out.stdout).map_err(|e| format!("graph_export gave bad JSON: {e}"))?
            };
            let title = flag(rest, "--title").or_else(|| graph["title"].as_str().map(String::from)).unwrap_or_else(|| "Graph".into());
            let ttl = flag(rest, "--ttl").and_then(|t| t.parse::<f64>().ok()).unwrap_or(120.0);
            Ok(json!({"t": "use", "name": "graph-view", "id": flag(rest, "--id").unwrap_or_else(|| "graph".into()), "opts": {"ttl": ttl, "topic": "viz.graph", "max_frac": flag(rest, "--max-frac").and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.82)}, "data": {"graph": graph, "title": title}}))
        }
        "list" => Ok(json!({"t": "list"})),
        "status" => Ok(json!({"t": "status"})),
        "uimap" => Ok(json!({"t": "ui_map"})),
        "space" => Ok(json!({"t": "space"})),
        // morc chore <reason> [--note JSON] : ask for a chore wake (the host applies the gates: live session, PC idle, rate limit, cooldown)
        "chore" => Ok(json!({"t": "chore", "reason": rest.first().cloned().unwrap_or_else(|| "periodic".into()), "note": flag(rest, "--note").and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).unwrap_or(json!({}))})),
        "stop" => Ok(json!({"t": "shutdown"})),
        other => Err(format!("unknown command '{other}' (try: morc help)")),
    })();

    let req = match req {
        Ok(r) => r,
        Err(e) => {
            eprintln!("morc: {e}");
            return 2;
        }
    };
    if cmd == "stop" && TcpStream::connect((paths::HOST, paths::PORT)).is_err() {
        println!("{{\"ok\":true,\"note\":\"not running\"}}");
        return 0;
    }
    if let Err(e) = ensure_host() {
        eprintln!("morc: {e}");
        return 1;
    }
    match send(req) {
        Ok(v) => {
            println!("{v}");
            if v["ok"].as_bool().unwrap_or(false) { 0 } else { 1 }
        }
        Err(e) => {
            eprintln!("morc: {e}");
            1
        }
    }
}
