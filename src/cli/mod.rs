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

mod input;
mod connect;
mod plugins;
mod stream;
mod annoy;
use input::*;
use connect::*;
use plugins::*;
use stream::*;
use annoy::*;

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
            println!("{}", include_str!("mod.rs").lines().take_while(|l| l.starts_with("//!")).skip(2).map(|l| l.trim_start_matches("//!").trim_end()).collect::<Vec<_>>().join("\n"));
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
