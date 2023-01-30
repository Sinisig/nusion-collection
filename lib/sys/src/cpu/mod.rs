//! Module containing all CPU-specific abstractions.

// Platform support check
#[cfg(not(any(
   target_arch = "x86_64",
)))] compile_error! (
   "Unsupported CPU architecture",
);

// Public modules
#[cfg(target_arch = "x86_64")]
pub mod amd64;

// Public re-exports
#[cfg(target_arch = "x86_64")]
pub use amd64::*;

