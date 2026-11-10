//! The operating-system specific parts: window shaping and styles, sampling the backdrop color, global hotkeys, watching the foreground window.
//! Windows: the files in this folder. macOS: the same module names in macos/ (see macos/README.md). The rest of the crate calls
//! `crate::winstyle`, `crate::backdrop`, `crate::hotkeys`, `crate::fgwatch` (main.rs) and never knows which one it got.

#[cfg(windows)]
pub mod backdrop;
#[cfg(windows)]
pub mod foreground_watch;
#[cfg(windows)]
pub mod hotkeys;
#[cfg(windows)]
pub mod window_style;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{backdrop, foreground_watch, hotkeys, window_style};
