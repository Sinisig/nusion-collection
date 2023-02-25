//! Crate root for nusion-sys, a collection
//! of raw system abstractions for internal
//! use in nusion.
//!
//! It is not recommended to use this crate
//! directly, but instead use safe(r) high-level
//! wrappers found in nusion.

// Internal modules
mod os;
mod cpu;

// Public-internal module re-exports
pub use os::osapi as osapi;

// Public modules
pub mod console;
pub mod compiler;
pub mod environment;
pub mod memory;
pub mod process;

// Unit tests
#[cfg(tests)]
mod tests;

