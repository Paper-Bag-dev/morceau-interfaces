//! The no-dead-controls rule (11-08, the user's rule): every control on screen must cause a visible, causal change
//! (move to the next step, change something shown, run an action, or close). Empty boxes and buttons that do nothing
//! never ship. Specs that break the rule are refused with a message saying which control and why, so the fix is one edit.
//!
//! Checked: a button needs an action; `set`/`toggle` must target a path that something on the surface actually shows;
//! `run` needs somewhere visible to put its output; a control with no label and no icon is an empty box; an icon-only
//! control must be self-evident (check / x) or carry a `title`. `opts.lint = "off"` skips this (demos/tests only).

use serde_json::Value;
use std::collections::HashSet;

/// Icons a user reads without a label. Anything else, icon-only, needs a `title`.
const SELF_EVIDENT: &[&str] = &["check", "x"];

fn walk_refs(v: &Value, out: &mut HashSet<String>) {
    match v {
        Value::Object(m) => {
            if let Some(Value::String(p)) = m.get("bind") {
                out.insert(p.clone());
            }
            // `when: {path}` shows/hides a component, so that path is visibly "shown" too
            if let Some(Value::String(p)) = m.get("when").and_then(|w| w.get("path")) {
                out.insert(p.clone());
            }
            for x in m.values() {
                walk_refs(x, out);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| walk_refs(x, out)),
        Value::String(s) => {
            // templates: {/path} or {/path:2}
            let mut rest = s.as_str();
            while let Some(i) = rest.find("{/") {
                let after = &rest[i + 1..];
                if let Some(j) = after.find('}') {
                    let p = after[..j].split(':').next().unwrap_or("");
                    out.insert(p.to_string());
                    rest = &after[j..];
                } else {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn shown(refs: &HashSet<String>, path: &str) -> bool {
    refs.iter().any(|r| r == path || r.starts_with(&format!("{path}/")) || path.starts_with(&format!("{r}/")))
}

fn actions(a: &Value) -> Vec<&Value> {
    match a {
        Value::Array(v) => v.iter().collect(),
        Value::Null => vec![],
        o => vec![o],
    }
}

fn check_action(owner: &str, a: &Value, refs: &HashSet<String>, errs: &mut Vec<String>) {
    match a["do"].as_str().unwrap_or("") {
        "set" | "toggle" => {
            let p = a["path"].as_str().unwrap_or("");
            if !shown(refs, p) {
                errs.push(format!("{owner}: changes {p} but nothing on screen shows it (a click that changes nothing visible). Bind a component to {p}, or make the action close/replace the surface."));
            }
        }
        "run" => {
            let into = a["into"].as_str().unwrap_or("/_out");
            if !shown(refs, into) {
                errs.push(format!("{owner}: run puts its output at {into} but nothing shows it. Add a text bound to {into}."));
            }
        }
        "copy" | "open" | "close" | "emit" | "wake" => {}
        "" => errs.push(format!("{owner}: action has no `do`")),
        other => errs.push(format!("{owner}: unknown action '{other}'")),
    }
}

/// Returns the problems found (empty = fine).
pub fn check(components: &[Value]) -> Vec<String> {
    let mut refs = HashSet::new();
    // what is shown: everything referenced except from action blocks themselves
    for c in components {
        let mut c2 = c.clone();
        if let Some(m) = c2.as_object_mut() {
            m.remove("action");
            m.remove("submit");
        }
        walk_refs(&c2, &mut refs);
    }
    // parent links + `when` paths, to check that answers resolve (the question must go away or change after a click)
    let mut parent: std::collections::HashMap<String, String> = Default::default();
    let mut when_of: std::collections::HashMap<String, String> = Default::default();
    for c in components {
        let id = c["id"].as_str().unwrap_or("?").to_string();
        if let Some(kids) = c["children"].as_array() {
            for k in kids.iter().filter_map(|k| k.as_str()) {
                parent.insert(k.to_string(), id.clone());
            }
        }
        if let Some(p) = c["when"]["path"].as_str() {
            when_of.insert(id, p.to_string());
        }
    }
    let resolves = |start: &str, paths: &[String]| {
        let mut cur = Some(start.to_string());
        while let Some(id) = cur {
            if when_of.get(&id).map(|w| paths.contains(w)).unwrap_or(false) {
                return true;
            }
            cur = parent.get(&id).cloned();
        }
        false
    };

    let mut errs = vec![];
    for c in components {
        let id = c["id"].as_str().unwrap_or("?");
        match c["type"].as_str().unwrap_or("") {
            "button" => {
                let owner = format!("button '{id}'");
                // Answers must resolve: a button that sets a value but stays on screen lets the user flip their answer forever
                // (a dead loop). The button, or a parent, must hide/transform on that value (`when`), or the action must close.
                // Reversible navigation (tabs) opts out explicitly with role:"tab".
                let acts_all = actions(&c["action"]);
                let sets: Vec<String> = acts_all.iter().filter(|a| a["do"] == "set").filter_map(|a| a["path"].as_str().map(String::from)).collect();
                let closes = acts_all.iter().any(|a| a["do"] == "close");
                if !sets.is_empty() && !closes && c["role"] != "tab" && !resolves(id, &sets) {
                    errs.push(format!("{owner}: sets {} but stays on screen unchanged, so the answer can be flipped forever (a dead loop). Put `when` on this button or its row (e.g. {{\"path\":\"{}\",\"eq\":false}}) so the question disappears and the result shows, or close; use role:\"tab\" only for reversible navigation.", sets.join(", "), sets[0]));
                }
                let has_label = !c["label"].is_null() && c["label"] != "";
                let icon = c["icon"].as_str().unwrap_or("");
                let swatch = !c["swatch"].is_null();
                if swatch && c["title"].is_null() {
                    errs.push(format!("{owner}: a colour swatch needs a `title` saying what it is"));
                }
                if !has_label && icon.is_empty() && !swatch {
                    errs.push(format!("{owner}: empty box (no label, no icon)"));
                }
                if !has_label && !icon.is_empty() && !SELF_EVIDENT.contains(&icon) && c["title"].is_null() {
                    errs.push(format!("{owner}: icon-only '{icon}' is not self-evident; add a label or a `title` (only check and x read without words)"));
                }
                let acts = actions(&c["action"]);
                if acts.is_empty() {
                    errs.push(format!("{owner}: no action (a dead button). Every click must cause a visible change: next step, replace content, run something, or close."));
                }
                for a in acts {
                    check_action(&owner, a, &refs, &mut errs);
                }
            }
            "input" => {
                let owner = format!("input '{id}'");
                if actions(&c["submit"]).is_empty() {
                    errs.push(format!("{owner}: no submit action (typing goes nowhere)"));
                }
                for a in actions(&c["submit"]) {
                    check_action(&owner, a, &refs, &mut errs);
                }
            }
            _ => {}
        }
    }
    errs
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dead_button_is_refused() {
        let e = check(&[json!({"id": "b", "type": "button", "label": "Details"})]);
        assert!(e.iter().any(|m| m.contains("dead button")));
    }

    #[test]
    fn set_must_be_shown() {
        let hidden = check(&[json!({"id": "b", "type": "button", "label": "+", "action": {"do": "set", "path": "/x", "value": 1}})]);
        assert!(hidden.iter().any(|m| m.contains("nothing on screen shows")));
        let shown = check(&[
            json!({"id": "b", "type": "button", "label": "+", "role": "tab", "action": {"do": "set", "path": "/x", "value": 1}}),
            json!({"id": "t", "type": "text", "text": "x is {/x}"}),
        ]);
        assert!(shown.is_empty(), "{shown:?}");
    }

    #[test]
    fn answers_must_resolve() {
        // the old demo's loop: pick yes/no, buttons stay, flip forever
        let loopy = check(&[
            json!({"id": "y", "type": "button", "label": "Yes", "action": {"do": "set", "path": "/msg", "value": "ok"}}),
            json!({"id": "t", "type": "text", "text": "{/msg}"}),
        ]);
        assert!(loopy.iter().any(|m| m.contains("dead loop")), "{loopy:?}");
        // fixed: the row hides once answered, and a result shows
        let fixed = check(&[
            json!({"id": "root", "type": "column", "children": ["row", "res"]}),
            json!({"id": "row", "type": "row", "when": {"path": "/answered", "eq": false}, "children": ["y"]}),
            json!({"id": "y", "type": "button", "label": "Yes", "action": [{"do": "set", "path": "/res", "value": "Good."}, {"do": "set", "path": "/answered", "value": true}]}),
            json!({"id": "res", "type": "text", "when": {"path": "/answered", "eq": true}, "text": "{/res}"}),
        ]);
        assert!(fixed.is_empty(), "{fixed:?}");
        // closing also resolves
        assert!(check(&[json!({"id": "b", "type": "button", "label": "Done", "action": [{"do": "set", "path": "/x", "value": 1}, {"do": "close"}]}), json!({"id": "t", "type": "text", "text": "{/x}"})]).is_empty());
    }

    #[test]
    fn icons_need_to_be_clear() {
        assert!(check(&[json!({"id": "b", "type": "button", "icon": "check", "action": {"do": "close"}})]).is_empty());
        assert!(!check(&[json!({"id": "b", "type": "button", "icon": "refresh", "action": {"do": "close"}})]).is_empty());
        assert!(check(&[json!({"id": "b", "type": "button", "icon": "refresh", "title": "Reload", "action": {"do": "close"}})]).is_empty());
    }

    #[test]
    fn swatches_need_a_title_and_a_visible_effect() {
        let ok = check(&[
            json!({"id": "s", "type": "button", "swatch": "#f00", "title": "Red", "role": "tab", "action": {"do": "set", "path": "/paint", "value": "#f00"}}),
            json!({"id": "m", "type": "model3d", "paint": {"bind": "/paint"}}),
        ]);
        assert!(ok.is_empty(), "{ok:?}");
        let bad = check(&[json!({"id": "s", "type": "button", "swatch": "#f00", "role": "tab", "action": {"do": "set", "path": "/paint", "value": "#f00"}}), json!({"id": "m", "type": "model3d", "paint": {"bind": "/paint"}})]);
        assert!(bad.iter().any(|m| m.contains("needs a `title`")), "{bad:?}");
    }

    #[test]
    fn run_needs_a_visible_home() {
        let e = check(&[json!({"id": "b", "type": "button", "label": "Report", "action": {"do": "run", "id": "show-report", "into": "/out"}})]);
        assert!(e.iter().any(|m| m.contains("run puts its output")));
    }
}
