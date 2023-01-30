//! Module containing all OS-specific abstractions.

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

