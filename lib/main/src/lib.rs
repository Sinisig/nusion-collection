//! Crate root for nusion, a general game
//! modding framework and utilities crate.

// Internal crate re-exports
use nusion_proc_macros  as proc_macros;
use nusion_sys          as sys;

// Public-internal module re-exports
pub use sys::osapi               as __osapi;
pub use sys::build_slib_entry    as __build_slib_entry;
pub use sys::os_build_slib_entry as __os_build_slib_entry;

// Public modules
pub mod console;
pub mod environment;
pub mod patch;
pub mod process;

// Public module re-exports
pub use proc_macros::*;
pub use patch::{
   PatchAlignment,
   Patch,
};
pub use process::{
   ProcessSnapshot,
   ModuleSnapshot,
   ProcessSnapshotList,
   ModuleSnapshotList,
};

// Unit tests
#[cfg(tests)]
mod tests;

