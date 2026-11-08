//! Where cards go. One responsibility per file:
//!   rect.rs        a screen rectangle (physical px)
//!   desktop.rs     what the desktop looks like now: windows, monitors, the active window, fullscreen (Windows API)
//!   scored.rs      scored placement: the cheapest candidate spot (the fallback when the regions do not fit)
//!   regions.rs     the region model: the 3x3 around the active window, tiers, thirds for fullscreen
//!   policy.rs      the placement weights and anchors (prefs/placement.json, hot-reloaded)
//!   remembered.rs  the spots you carried cards to

mod desktop;
mod rect;
mod scored;
pub mod policy;
pub mod regions;
pub mod remembered;

pub use desktop::*;
pub use rect::R;
pub use scored::{pick, Cohesion};

