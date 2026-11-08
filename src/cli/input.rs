//! Reading the command line: a JSON argument (inline, @file or stdin), flags, values. Split out of cli/mod.rs.

use super::*;

pub(super) fn read_input(arg: Option<&String>) -> Result<Value, String> {
    let mut s = String::new();
    match arg.map(|a| a.as_str()) {
        None | Some("-") => {
            std::io::stdin().read_to_string(&mut s).map_err(|e| e.to_string())?;
        }
        Some(p) => s = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?,
    }
    serde_json::from_str(s.trim_start_matches('\u{feff}')).map_err(|e| format!("bad JSON: {e}"))
}

pub(super) fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned()
}

pub(super) fn has(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

pub(super) fn parse_value(s: &str) -> Value {
    serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.to_string()))
}

