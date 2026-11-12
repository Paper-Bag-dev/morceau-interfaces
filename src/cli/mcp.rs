//! `morc mcp`: a stdio MCP server (newline-delimited JSON-RPC) over the same host requests the CLI sends (docs/mcp.md).
//! Seven generic tools, fixed: a new chip, action or plugin never changes this list (they are data the tools take).
//! The CLI stays the main interface; this is a thin adapter, no logic of its own beyond the inbox wait.

use super::{ensure_host, send};
use crate::inbox;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::time::{Duration, Instant};

fn tools() -> Value {
    let obj = |props: Value, req: &[&str]| json!({"type": "object", "properties": props, "required": req});
    json!([
        {"name": "scene", "description": "The scene tree: displays, app windows front to back, morc cards as mount nodes, `covers` edges (what overlaps what, physical px). all=true mounts every card's own tree too.",
         "inputSchema": obj(json!({"all": {"type": "boolean"}, "depth": {"type": "integer", "description": "card tree depth when all=true"}}), &[])},
        {"name": "card_tree", "description": "One card's tree: its components with ids, types, rects (css px) and flags (cut = text cut off, clipped = content hidden without a scroll, scrolls, offcard, hidden), its data keys and named actions.",
         "inputSchema": obj(json!({"card": {"type": "string"}, "depth": {"type": "integer"}, "node": {"type": "string", "description": "start at this component id"}}), &["card"])},
        {"name": "show_chip", "description": "Show a cached chip (morc use): name, data to merge over its defaults, optional id and opts (ttl, sticky, region, size, persist, actions ...).",
         "inputSchema": obj(json!({"name": {"type": "string"}, "id": {"type": "string"}, "data": {"type": "object"}, "opts": {"type": "object"}}), &["name"])},
        {"name": "patch", "description": "Change data on a live card without re-showing it: data = {\"/pointer\": value}. card \"@shared\" = the store all linked cards can bind to.",
         "inputSchema": obj(json!({"card": {"type": "string"}, "data": {"type": "object"}}), &["card", "data"])},
        {"name": "do_action", "description": "Run a card's named action (declared in its opts.actions) with JSON args; returns the page's answer.",
         "inputSchema": obj(json!({"card": {"type": "string"}, "action": {"type": "string"}, "args": {}}), &["card", "action"])},
        {"name": "card", "description": "A card as a resource. No card: every card the host knows (shown, hidden, closed). card only: its state (state, layer, rect, frac, chip). card + set: the state you want, e.g. {\"state\":\"hidden\"}, {\"state\":\"shown\"} (a closed card comes back with its data and spot), {\"layer\":\"below\"} (behind the active window; \"top\" brings it back), {\"frac\":0.9,\"center\":true}, {\"at\":[x,y]}. Never shows a card twice.",
         "inputSchema": obj(json!({"card": {"type": "string"}, "set": {"type": "object"}}), &[])},
        {"name": "wait_events", "description": "Wait (up to timeout_s, default 30, max 300) for the user's interactions with cards (inbox rows: clicks, answers, copies). Returns them and marks them read.",
         "inputSchema": obj(json!({"timeout_s": {"type": "number"}, "surface": {"type": "string", "description": "only rows from this card"}}), &[])}
    ])
}

fn call(name: &str, a: &Value) -> Result<Value, String> {
    let req = match name {
        "scene" => json!({"t": "tree", "all": a["all"].as_bool().unwrap_or(false), "depth": a["depth"]}),
        "card_tree" => json!({"t": "tree", "id": a["card"], "depth": a["depth"], "node": a["node"]}),
        "show_chip" => json!({"t": "use", "name": a["name"], "id": a["id"], "data": if a["data"].is_null() { json!({}) } else { a["data"].clone() }, "opts": if a["opts"].is_null() { json!({}) } else { a["opts"].clone() }}),
        "patch" => json!({"t": "patch", "id": a["card"], "data": a["data"]}),
        "do_action" => json!({"t": "do", "id": a["card"], "name": a["action"], "args": a["args"]}),
        "card" => match (a["card"].as_str(), a["set"].is_object()) {
            (None, _) => json!({"t": "cards"}),
            (Some(id), false) => json!({"t": "card", "id": id, "op": "get"}),
            (Some(id), true) => json!({"t": "card", "id": id, "op": "set", "set": a["set"]}),
        },
        "wait_events" => {
            let until = Instant::now() + Duration::from_secs_f64(a["timeout_s"].as_f64().unwrap_or(30.0).clamp(0.0, 300.0));
            let only = a["surface"].as_str();
            loop {
                let rows: Vec<Value> = inbox::unread(true).into_iter().filter(|r| only.map(|s| r["surface"] == s).unwrap_or(true)).collect();
                if !rows.is_empty() {
                    inbox::unread(false);   // mark read only once something matched
                    return Ok(json!({"ok": true, "events": rows}));
                }
                if Instant::now() >= until {
                    return Ok(json!({"ok": true, "events": [], "note": "timed out"}));
                }
                std::thread::sleep(Duration::from_millis(250));
            }
        }
        other => return Err(format!("unknown tool {other}")),
    };
    ensure_host()?;
    send(req)
}

pub fn serve() -> i32 {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let Ok(m) = serde_json::from_str::<Value>(&line) else { continue };
        let Some(id) = m.get("id").cloned() else { continue };   // notifications need no answer
        let result = match m["method"].as_str().unwrap_or("") {
            "initialize" => Ok(json!({
                "protocolVersion": m["params"]["protocolVersion"].as_str().unwrap_or("2025-06-18"),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "morc", "version": env!("CARGO_PKG_VERSION")}
            })),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({"tools": tools()})),
            "tools/call" => {
                let p = &m["params"];
                Ok(match call(p["name"].as_str().unwrap_or(""), &p["arguments"]) {
                    Ok(v) => json!({"content": [{"type": "text", "text": v.to_string()}], "isError": !v["ok"].as_bool().unwrap_or(false)}),
                    Err(e) => json!({"content": [{"type": "text", "text": e}], "isError": true}),
                })
            }
            other => Err(json!({"code": -32601, "message": format!("method not found: {other}")})),
        };
        let reply = match result {
            Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
            Err(e) => json!({"jsonrpc": "2.0", "id": id, "error": e}),
        };
        if writeln!(out, "{reply}").and_then(|_| out.flush()).is_err() {
            break;
        }
    }
    0
}
