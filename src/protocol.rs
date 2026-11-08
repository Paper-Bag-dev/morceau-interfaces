//! The wire protocol: one JSON object per line over localhost TCP.
//! Envelope: {"token": "...", "req": {"t": "...", ...}}  ->  reply: one JSON line.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Opts {
    /// Seconds the surface stays with no interaction (default 15).
    pub ttl: Option<f64>,
    /// Never auto-expire by timer (still removed quietly if stale for 10 min).
    pub sticky: bool,
    /// Needs the user: sticky, pulses, and goes to the pending list instead of vanishing.
    pub urgent: bool,
    /// right (default: the right edge, upper-middle) | near_focus | corner | center | side | bottom
    pub anchor: Option<String>,
    /// Exact placement, physical px of the visible content's top-left: the agent did the math (see `morc space`). Wins over anchor and the remembered spot.
    pub at: Option<[i32; 2]>,
    /// The smallest this surface can be, css px at zoom 1: [width, height] after it has wrapped/flowed as much as it can. Declared when the
    /// component is generated (opts.min in the saved spec), so the host can pick a region that fits BEFORE measuring.
    pub min: Option<[i32; 2]>,
    /// Place INTO a free region: right | below | left | above | mid | gap | auto (first that fits, filling right then down then left).
    pub region: Option<String>,
    /// Alignment inside the region: left|right|top|bottom|middle, combined with '-' (e.g. "top-right", "bottom", "middle"). Default right-top.
    pub align: Option<String>,
    /// Pixel offsets (physical) from the aligned spot; positive = right / down. Fine granularity inside a region.
    pub dx: Option<i32>,
    pub dy: Option<i32>,
    /// Engagement bucket, e.g. "stats.minutes". Defaults to the cached component name.
    pub topic: Option<String>,
    /// Color theme: a name (ocean, violet, rose, ...), "random", or omitted for topic-matched + daily drift.
    pub theme: Option<String>,
    /// low | normal | high | critical (or a number): scales the surface. Capped to 1/4 of the screen's usable area.
    pub importance: Option<String>,
    /// Grow as large as the size cap allows (big charts worth looking at).
    pub fill: bool,
    /// Size cap as a fraction of the usable width AND height (default 0.5 = a quarter of the screen). Working windows such as graphs use ~0.8.
    pub max_frac: Option<f64>,
    /// "off" disables backdrop-aware colors.
    pub contrast: Option<String>,
    /// "off" skips the dead-control linter (demos and tests only: every control must cause a visible change).
    pub lint: Option<String>,
    /// Pushed by an anticipation trigger rather than asked for: throttled by engagement.
    pub auto: bool,
    /// Ignore throttling and fullscreen deferral.
    pub force: bool,
    /// A wake payload fired ONCE when this surface first appears: the background agent starts preparing what the card's "show more"
    /// button will reveal, so the click is answered at once instead of after the agent has started. The host adds `surface` and `bg`.
    pub prefetch: Option<Value>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Request {
    /// Show (or replace, if the id exists) a surface.
    Show {
        id: Option<String>,
        #[serde(default)]
        opts: Opts,
        #[serde(default)]
        components: Vec<Value>,
        #[serde(default)]
        data: Value,
        #[serde(default)]
        sources: Value,
    },
    /// Show a cached component by name, overriding its data.
    Use {
        name: String,
        id: Option<String>,
        #[serde(default)]
        opts: Opts,
        #[serde(default)]
        data: Value,
    },
    /// Mutate a live surface. `data` is a flat map of JSON pointer -> value.
    Patch {
        id: String,
        #[serde(default)]
        data: serde_json::Map<String, Value>,
        #[serde(default)]
        components: Vec<Value>,
        /// Mount the first component into this parent instead of rebuilding the surface: {"parent": "root", "before": "__build"}.
        /// This is how a surface is built up piece by piece (see 2ui stream).
        #[serde(default)]
        attach: Value,
    },
    Close { id: String },
    /// Cache a spec under a name (also updates cache/registry.md).
    Save {
        name: String,
        #[serde(default)]
        desc: String,
        #[serde(default)]
        params: Vec<String>,
        #[serde(default)]
        opts: Opts,
        #[serde(default)]
        components: Vec<Value>,
        #[serde(default)]
        data: Value,
        #[serde(default)]
        sources: Value,
    },
    /// Re-show everything that was hidden because it needed the user.
    Summon,
    /// Dismiss every floating UI at once (also bound to Ctrl+Alt+X).
    DismissAll,
    /// Run a host-side action (run / emit / wake / open) on a live surface as if its button was clicked.
    /// For testing and for the agent; page-local actions (set / toggle / close) are not handled here.
    Act { id: String, action: Value },
    List,
    Status,
    /// A token / theme / limits file changed: re-send the UI data (tokens, themes, limits) to every card. Sent by the host's own file watcher.
    Reload,
    /// Where is free space right now: the active window, the strips around it, other windows, my surfaces.
    Space,
    /// A chore wake for the background agent (gated: live session, PC idle, rate limit, chore cooldown). From a trigger rule, a wake flag, an external tool, or `morc chore`.
    Chore {
        reason: String,
        #[serde(default)]
        note: Value,
    },
    Shutdown,
}

#[derive(Deserialize, Debug)]
pub struct Envelope {
    pub token: String,
    pub req: Request,
}
