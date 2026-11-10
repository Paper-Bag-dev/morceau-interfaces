//! The Windows side of src/platform: the same modules and function signatures as platform/macos, so shared code calls
//! `crate::platform::os::<module>::<fn>` and never names an OS.

pub mod app;
pub mod backdrop;
pub mod clock;
pub mod desktop;
pub mod foreground_watch;
pub mod hotkeys;
pub mod process;
pub mod sys;
pub mod window_style;
