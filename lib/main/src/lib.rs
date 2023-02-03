//! Crate root for nusion, a general game
//! modding framework and utilities crate.

// Public modules
pub mod patch;
pub mod macros;

// Public re-exports
pub use nusion_proc_macros::*;

// Unit tests
#[cfg(tests)]
mod tests;

