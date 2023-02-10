//! OS Implementations for Windows.

// OS API public re-export
pub use winapi as osapi;

// Public modules
pub mod console;
pub mod entry;
pub mod environment;
pub mod memory;
pub mod process;

