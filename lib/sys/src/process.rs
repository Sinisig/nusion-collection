//! Utilities for enumerating and
//! creating snapshots of processes
//! and modules contained within
//! processes and performing actions
//! on them.

use core::ffi::c_void;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Error type for describing some
/// issue relating to a process or
/// module function failing.
#[derive(Debug)]
pub enum ProcessError {
   BadExecutableFileName,
   Unknown,
}

/// A Result type with Err variant
/// ProcessError.
pub type Result<T> = std::result::Result<T, ProcessError>;

/// A snapshot of a process and its
/// information.
pub struct ProcessSnapshot {
   os_snapshot : crate::os::process::ProcessSnapshot,
}

/// A snapshot of a module within
/// a given process snapshot.
pub struct ModuleSnapshot<'l> {
   os_snapshot : crate::os::process::ModuleSnapshot<'l>,
}

//////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ProcessSnapshotError //
//////////////////////////////////////////////////

impl std::fmt::Display for ProcessError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::BadExecutableFileName
            => "Process executable file name contains invalid UTF-8",
         Self::Unknown
            => "Unknown error",
      });
   }
}

impl std::error::Error for ProcessError {
}

///////////////////////////////
// METHODS - ProcessSnapshot //
///////////////////////////////

impl ProcessSnapshot {
   /// Creates a snapshot of every
   /// process currently running on
   /// the system.
   pub fn all(
   ) -> Result<Vec<Self>> {
      let list = crate::os::process::ProcessSnapshot::all()?;
      let list = list.into_iter().map(|snap| {
         Self{ os_snapshot : snap }
      }).collect();

      return Ok(list);
   }

   /// Creates a snapshot of the
   /// local process.  Useful for
   /// enumerating loaded modules
   /// in the local running process.
   pub fn local(
   ) -> Result<Self> {
      return Ok(Self{
         os_snapshot : crate::os::process::ProcessSnapshot::local()?,
      });
   }

   /// Enumerates all modules within the
   /// given process.
   pub fn modules<'l>(
      &'l self,
   ) -> Result<Vec<ModuleSnapshot<'l>>> {
      return ModuleSnapshot::all_within(self);
   }

   /// Retrieves the file name of the
   /// main executable for the process.
   /// This only contains the file name
   /// and extension.  The full path is
   /// not included.
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return self.os_snapshot.executable_file_name();
   }
}

//////////////////////////////
// METHODS - ModuleSnapshot //
//////////////////////////////

impl<'l> ModuleSnapshot<'l> {
   /// Creates a snapshot of every module
   /// within a given process.
   pub fn all_within(
      parent_process : &'l ProcessSnapshot
   ) -> Result<Vec<Self>> {
      let list = crate::os::process::ModuleSnapshot::all(&parent_process.os_snapshot)?;
      let list = list.into_iter().map(|snap| {
         Self{ os_snapshot : snap }
      }).collect();

      return Ok(list);
   }

   /// Gets the address range within
   /// the process occupied by the
   /// module.
   pub fn address_range(
      &'l self,
   ) -> &'l std::ops::Range<* const c_void> {
      return self.os_snapshot.address_range();
   }

   /// Retrieves the fil name of the
   /// module executable.  This only
   /// contains the file name and
   /// extension.  The full path is
   /// not included.
   pub fn executable_file_name(
      &'l self,
   ) -> &'l str {
      return self.os_snapshot.executable_file_name();
   }
}

