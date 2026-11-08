// Console-subsystem on purpose: the CLI must be waitable and its output capturable (PowerShell does neither for GUI exes).
// The host never flashes a console because `morc host` is always spawned detached with CREATE_NO_WINDOW (cli/mod.rs).
//
// The code is split by responsibility, one folder each (a README in a folder says what is in it):
//   cli/        the command-line client          host/       the windows, the event loop, showing cards
//   placement/  where cards go (regions, policy) surface/    one card: its lifetime, actions, linter
//   agent/      the background agent and triggers  learning/  annoyance, engagement, literacy
//   ipc/        the CLI <-> host protocol        data/       paths, the component cache, served files
//   platform/   the Windows-specific parts       util.rs     small shared helpers

mod agent;
mod cli;
mod data;
mod host;
mod ipc;
mod learning;
mod placement;
mod platform;
mod surface;
mod util;

// Short names for the modules inside the folders, so code anywhere says `crate::paths`, `crate::prefs`, `crate::wake` ...
use agent::{inbox, live_sources as sources, triggers as anticipate, wake, worker};
use data::{assets, cache, paths};
use ipc::{events as ev, protocol, server};
use learning::{annoyance as prefs, engagement, literacy};
use placement::{policy, regions, remembered as placed};
use platform::{backdrop, foreground_watch as fgwatch, hotkeys, window_style as winstyle};
use surface::{actions as handlers, lifetime, lint};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("host") {
        host::run();
    }
    std::process::exit(cli::run(args));
}
