//! Crate root for nusion, a general game
//! modding framework and utilities crate.

// Public modules
pub mod patch;

// Macro crate re-exports
pub use nusion_macros::*;

// Unit tests
#[cfg(tests)]
mod tests;

