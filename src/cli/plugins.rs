//! CLI plugins: any script in tools/cli (your data folder first, then the install) becomes `morc <name>`. Split out of cli/mod.rs.

use super::*;

/// Extension scripts live in tools/cli/. They get MORC_BIN (this exe) and MORC_HOME, so they can call back into morc to show things.
pub(super) fn external(cmd: &str) -> Option<std::path::PathBuf> {
    if cmd.is_empty() || !cmd.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    [paths::home_dir(), paths::app_dir()].iter().flat_map(|root| ["py", "ps1", "cmd"].iter().map(move |e| root.join("tools").join("cli").join(format!("{cmd}.{e}")))).find(|p| p.exists())
}

pub(super) fn run_external(p: &std::path::Path, args: &[String]) -> i32 {
    let mut c = match p.extension().and_then(|e| e.to_str()) {
        Some("py") => { let mut c = std::process::Command::new(crate::util::PYTHON); c.arg(p); c }
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
pub(super) fn external_list() -> Vec<(String, String)> {
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

