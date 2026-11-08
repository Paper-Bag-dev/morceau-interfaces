// Console-subsystem on purpose: the CLI must be waitable and its output capturable (PowerShell does neither for GUI exes).
// The host never flashes a console because `morc host` is always spawned detached with CREATE_NO_WINDOW (client.rs).

mod anticipate;
mod assets;
mod backdrop;
mod prefs;
mod cache;
mod client;
mod engagement;
mod ev;
mod fgwatch;
mod handlers;
mod host;
mod hotkeys;
mod inbox;
mod lifetime;
mod lint;
mod literacy;
mod paths;
mod placed;
mod placement;
mod policy;
mod protocol;
mod regions;
mod server;
mod sources;
mod surface;
mod util;
mod wake;
mod winstyle;
mod worker;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(|s| s.as_str()) == Some("host") {
        host::run();
    }
    std::process::exit(client::run(args));
}
