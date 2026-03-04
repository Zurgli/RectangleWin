//! Win32 FFI for window, monitor, cursor, process.
//! Only compiled on Windows.

#[cfg(windows)]
mod hotkey_hook;
#[cfg(windows)]
mod impl_;

#[cfg(windows)]
pub use hotkey_hook::{set_hotkeys, start as start_lowlevel_hook};
#[cfg(windows)]
pub use impl_::*;
