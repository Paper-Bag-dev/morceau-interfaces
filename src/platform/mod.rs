//! The operating-system specific parts (Windows today): window shaping and styles, sampling the backdrop color, global hotkeys, watching the foreground window. A macOS port replaces this folder.

pub mod backdrop;
pub mod foreground_watch;
pub mod hotkeys;
pub mod window_style;

