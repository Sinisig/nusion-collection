//! Utilities for enumerating and
//! creating snapshots of processes
//! and modules contained within
//! processes and performing actions
//! on them.

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

/// Result type with error variant
/// <code>ProcessError</code>.
pub type Result<T> = std::result::Result<T, ProcessError>;

/// A snapshot of a process and its
/// information.
pub struct ProcessSnapshot {
   snapshot : crate::os::process::ProcessSnapshot,
}

/// A snapshot of a module within
/// a given process snapshot.
pub struct ModuleSnapshot {
   snapshot : crate::os::process::ModuleSnapshot,
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
            => "Associated executable file name contains invalid UTF-8",
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
         Self{snapshot : snap}
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
         snapshot : crate::os::process::ProcessSnapshot::local()?,
      });
   }

   /// Enumerates all modules within
   /// the process.  This is equivalent
   /// to <code><ModuleSnapshot::all_within(self)</code>.
   pub fn modules<'l>(
      &'l self,
   ) -> Result<Vec<ModuleSnapshot>> {
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
      return &self.snapshot.executable_name;
   }
}

//////////////////////////////
// METHODS - ModuleSnapshot //
//////////////////////////////

impl ModuleSnapshot {
   /// Creates a snapshot of every module
   /// within a given process.  This is
   /// equivalent to <code>ProcessSnapshot::modules(self)</code>.
   pub fn all_within(
      parent_process : & ProcessSnapshot
   ) -> Result<Vec<Self>> {
      let list = crate::os::process::ModuleSnapshot::all(&parent_process.snapshot)?;
      let list = list.into_iter().map(|snap| {
         Self{snapshot : snap}
      }).collect();

      return Ok(list);
   }

   /// Gets the address range within
   /// the process occupied by the module.
   /// This may or may not be physical or
   /// virtual memory depending on the
   /// operating system.  It is the
   /// memory address range as known
   /// within the containing process.
   pub fn address_range<'l>(
      &'l self,
   ) -> &'l std::ops::Range<usize> {
      return &self.snapshot.address_range;
   }

   /// Retrieves the fil name of the
   /// module executable.  This only
   /// contains the file name and
   /// extension.  The full path is
   /// not included.
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return &self.snapshot.module_name;
   }
}

