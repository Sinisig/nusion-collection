//! Crate root for nusion, a general game
//! modding framework and utilities crate.

// Public modules
pub mod console;
pub mod env;
pub mod patch;

// Crate re-exports
pub use nusion_proc_macros as proc_macros;
pub use nusion_sys         as sys;

// Public module re-exports
pub use proc_macros::*;
pub use env::Environment;
pub use patch::Patch;

// Unit tests
#[cfg(tests)]
mod tests;

