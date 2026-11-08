//! The background agent and everything that wakes or feeds it: the headless chore agent (wake), its settings and gates (worker), the event inbox the hooks read (inbox), trigger rules that show things before you ask (triggers), live data sources (live_sources).

pub mod inbox;
pub mod live_sources;
pub mod triggers;
pub mod wake;
pub mod worker;

