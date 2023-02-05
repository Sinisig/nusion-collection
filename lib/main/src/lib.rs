//! Crate root for nusion, a general game
//! modding framework and utilities crate.

// Public modules
pub mod patch;

// Public crate re-exports
pub use nusion_sys         as sys;
pub use nusion_proc_macros as proc_macros;

// Public module re-exports
pub use proc_macros::*;

// Unit tests
#[cfg(tests)]
mod tests;

