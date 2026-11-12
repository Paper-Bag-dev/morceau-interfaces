//! Card state on disk (docs/state.md). Only for cards with `opts.persist`; everything else stays in memory and dies with the card.
//!   <home>/state/<id>.json           the latest snapshot {schema, saved_at, data}, written to a temp file then renamed (never half a file)
//!   <home>/state/<id>.history.jsonl  every named action as it ran {ts, name, args}, appended in batches every 0.5 s (a burst = one write)
//! The page sends snapshots with a falloff (1 s after the last change, at most every 5 s while changes keep coming: core/main.js); the
//! host only writes what it is given.

use super::*;
use std::io::Write;

fn dir() -> std::path::PathBuf {
    let d = paths::home_dir().join("state");
    let _ = std::fs::create_dir_all(&d);
    d
}

/// Card ids become file names: keep them to a safe alphabet.
fn file_id(id: &str) -> String {
    id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' }).collect()
}

/// Merge the last snapshot (if any) over a card's starting data.
pub(super) fn restore(id: &str, data: &mut Value) {
    let Ok(text) = std::fs::read_to_string(dir().join(format!("{}.json", file_id(id)))) else { return };
    if let Ok(snap) = serde_json::from_str::<Value>(&text) {
        util::deep_merge(data, &snap["data"]);
    }
}

/// Action lines waiting to be appended: (first unsaved, card file id -> lines). A burst of actions is one append per card, not one per action.
static HISTORY: std::sync::Mutex<(Option<Instant>, std::collections::BTreeMap<String, Vec<String>>)> = std::sync::Mutex::new((None, std::collections::BTreeMap::new()));

/// From tick (and on shutdown): append the buffered lines once they are HISTORY_WAIT old. Lines are written in order, so the log stays
/// complete; at most HISTORY_WAIT of actions is in memory at any time.
pub(super) fn flush_history(force: bool) {
    const HISTORY_WAIT: Duration = Duration::from_millis(500);
    let Ok(mut b) = HISTORY.lock() else { return };
    if !(force || b.0.map(|t| t.elapsed() >= HISTORY_WAIT).unwrap_or(false)) {
        return;
    }
    b.0 = None;
    for (id, lines) in std::mem::take(&mut b.1) {
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir().join(format!("{id}.history.jsonl"))) {
            let _ = f.write_all((lines.join("\n") + "\n").as_bytes());
        }
    }
}

impl Host {
    fn persisted(&self, key: u64) -> Option<&Surface> {
        self.surfaces.get(&key).filter(|s| s.opts.persist)
    }

    /// A named action ran on the page (ipc `{"t":"did","name","args"}`).
    pub(super) fn did(&mut self, key: u64, m: &Value) {
        let Some(s) = self.persisted(key) else { return };
        let line = json!({"ts": util::now_ms() as u64, "name": m["name"], "args": m["args"]});
        if let Ok(mut b) = HISTORY.lock() {
            b.0.get_or_insert_with(Instant::now);
            b.1.entry(file_id(&s.id)).or_default().push(line.to_string());
        }
    }

    /// The page's debounced snapshot (ipc `{"t":"snap","data"}`).
    pub(super) fn snap(&mut self, key: u64, m: &Value) {
        let Some(s) = self.persisted(key) else { return };
        let path = dir().join(format!("{}.json", file_id(&s.id)));
        let tmp = path.with_extension("json.tmp");
        let body = json!({"schema": 1, "saved_at": util::now_ms() as u64, "data": m["data"]});
        if std::fs::write(&tmp, body.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}
