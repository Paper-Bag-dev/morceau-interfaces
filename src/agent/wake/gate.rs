//! The gates every wake passes: enabled, live session (optional), PC idle, rate limit, chore cooldown. Split out of wake/mod.rs.

use super::*;

/// `direct` = the user just clicked a button that asked for background work (payload "bg": true). They are plainly at the PC, so the
/// live-session and idle gates are skipped; the enabled switch and the rate limit still apply.
pub(super) fn gate(w: &worker::Worker, chore: bool, direct: bool, click: bool) -> Result<Vec<u128>, &'static str> {
    if !w.enabled {
        return Err("disabled");
    }
    if !direct {
        if w.defer_to_session {
            if let Some(age) = inbox::session_age_s() {
                if age < w.session_fresh_s {
                    return Err("session-live");
                }
            }
        }
        if worker::idle_seconds() > w.idle_s {
            return Err("pc-idle");
        }
    }
    let now = util::now_ms();
    let log = fs::read_to_string(paths::wake_log_file()).unwrap_or_default();
    let recent: Vec<u128> = log.lines().filter_map(|l| l.trim().parse().ok()).filter(|t| now.saturating_sub(*t) < 3_600_000).collect();
    // The rate limit guards against the host waking the agent by itself (triggers, prefetch). A click the user just made is never limited:
    // they asked, and a limit that eats their click leaves skeletons on screen.
    if !click && (recent.len() >= w.max_per_hour || recent.iter().any(|t| now.saturating_sub(*t) < (w.min_gap_s as u128) * 1000)) {
        return Err("rate-limited");
    }
    if chore && (now.saturating_sub(worker::last_chore_ms()) as f64) < w.chore_cooldown_min * 60_000.0 {
        return Err("chore-cooldown");
    }
    Ok(recent)
}
