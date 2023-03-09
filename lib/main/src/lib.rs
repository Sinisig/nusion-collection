//! Crate root for nusion, a general game
//! modding framework and utilities crate.

// Internal crate re-exports
use nusion_lib_proc  as proc;
use nusion_lib_sys   as sys;

// Public-internal module re-exports
pub use sys::__osapi as __osapi;

// Public modules
pub mod console;
pub mod environment;
pub mod macros;
pub mod patch;
pub mod process;

// Public module re-exports
pub use proc::*;
pub use process::{
   ProcessSnapshot,
   ModuleSnapshot,
   ProcessSnapshotList,
   ModuleSnapshotList,
};

// Unit tests
#[cfg(tests)]
mod tests;

