//! The macOS side of src/platform: the same module names and function signatures as the Windows files one folder up, so the host is
//! unchanged. `sys.rs` holds the raw AppKit / CoreGraphics / libdispatch calls the port needs, in one place.

pub mod app;
pub mod backdrop;
pub mod clock;
pub mod desktop;
pub mod foreground_watch;
pub mod glass;
pub mod hotkeys;
pub mod pipe;
pub mod process;
pub mod sys;
pub mod window_style;
