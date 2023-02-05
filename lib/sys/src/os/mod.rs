//! Module containing all OS-specific abstractions.
//! This module is only public for the sake of
//! providing public re-exports of OS API crates
//! to allow proper expansion of certain macros.

// Platform support check
#[cfg(not(any(
   target_os = "windows",
)))] compile_error! (
   "Unsupported target operating system",
);

// Public modules
#[cfg(target_os = "windows")]
pub mod windows;

// Public re-exports
#[cfg(target_os = "windows")]
pub use windows::*;

