//! The operating-system specific parts. `windows/` and `macos/` export the SAME modules with the SAME functions (app, clock, process,
//! sys, desktop, window_style, backdrop, hotkeys, foreground_watch); `os` is the one for this build. Shared code calls
//! `crate::platform::os::...` and never names an OS: a new platform is a new folder with the same functions, no shared file changes.

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos as os;

#[cfg(windows)]
pub mod windows;
#[cfg(windows)]
pub use self::windows as os;

pub use os::{backdrop, foreground_watch, hotkeys, window_style};
