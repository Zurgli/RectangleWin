//! Win32 FFI for window, monitor, cursor, process.
//! Only compiled on Windows.

#[cfg(windows)]
mod hotkey_hook;
#[cfg(windows)]
mod impl_;
#[cfg(windows)]
mod move_size_hook;

#[cfg(windows)]
pub use hotkey_hook::{set_hotkeys, start as start_lowlevel_hook};
#[cfg(windows)]
pub use impl_::*;
#[cfg(windows)]
pub use move_size_hook::start as start_move_size_end_hook;
