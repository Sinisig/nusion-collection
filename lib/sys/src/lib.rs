//! Crate root for nusion-sys, a collection
//! of raw system abstractions for internal
//! use in nusion.
//!
//! It is not recommended to use this crate
//! directly, but instead use safe(r) high-level
//! wrappers found in nusion.

// Platform-specific modules
pub mod os;
pub mod cpu;

// Public modules
pub mod console;
pub mod env;
pub mod macros;
pub mod mem;

// Unit tests
#[cfg(tests)]
mod tests;

