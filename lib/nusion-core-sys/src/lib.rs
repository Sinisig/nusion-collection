//! Crate root for nusion-core-sys, a collection
//! of raw system abstractions for internal
//! use in nusion-core.
//!
//! It is not recommended to use this crate
//! directly, but instead use safe(r) high-level
//! wrappers found in nusion-core.

// Internal modules
mod os;
mod cpu;

// Public-internal module re-exports
pub use os::osapi as __osapi;

// Public modules
pub mod console;
pub mod compiler;
pub mod environment;
pub mod memory;
pub mod process;

