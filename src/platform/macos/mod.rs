//! The macOS side of src/platform: the same module names and function signatures as the Windows files one folder up, so the host is
//! unchanged. `sys.rs` holds the raw AppKit / CoreGraphics / libdispatch calls the port needs, in one place.

pub mod backdrop;
pub mod foreground_watch;
pub mod hotkeys;
pub mod sys;
pub mod window_style;
