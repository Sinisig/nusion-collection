//! Module containing all OS-specific abstractions.

// Platform support check
#[cfg(not(any(
   target_os = "windows",
)))] compile_error! (
   "Unsupported target operating system",
);

// OS abstraction modules
#[cfg(target_os = "windows")]
pub mod windows;

// OS abstraction re-exports
#[cfg(target_os = "windows")]
pub use windows::*;

