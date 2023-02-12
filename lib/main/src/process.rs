//! Retrieve information about running
//! processes and create a snapshot of
//! processes and their loaded libraries.

use core::ffi::c_void;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error enum containing the
/// reason behind a process function
/// failing.
#[derive(Debug)]
pub enum ProcessError {
   BadExecutableFileName,
   Unknown,
}

/// A Result type with Err variant
/// ProcessError
pub type Result<T> = std::result::Result<T, ProcessError>;

/// A snapshot of various information
/// about a process.
pub struct ProcessSnapshot {
   snap  : crate::sys::process::ProcessSnapshot,
}

/// A snapshot of a loaded library
/// or executable within a process.
pub struct ModuleSnapshot {
   snap  : crate::sys::process::ModuleSnapshot,
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ProcessError //
//////////////////////////////////////////

impl std::fmt::Display for ProcessError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::BadExecutableFileName
            => "Executable file name contains invalid characters",
         Self::Unknown
            => "Unknown",
      });
   }
}

impl std::error::Error for ProcessError {
}

impl From<crate::sys::process::ProcessError> for ProcessError {
   fn from(
      item : crate::sys::process::ProcessError,
   ) -> Self {
      use crate::sys::process::ProcessError::*;
      return match item {
         BadExecutableFileName
            => Self::BadExecutableFileName,
         Unknown
            => Self::Unknown,
      };
   }
}

///////////////////////////////
// METHODS - ProcessSnapshot //
///////////////////////////////

impl ProcessSnapshot {
   /// Creates a snapshot of the
   /// program's own process.
   pub fn local(
   ) -> Result<Self> {
      return Ok(Self{
         snap : crate::sys::process::ProcessSnapshot::local()?,
      });
   }

   /// Gets the file name of the
   /// executable which spawned the
   /// process.  This only includes
   /// the file name and extension
   /// without the containing file
   /// path.
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return self.snap.executable_file_name();
   }
}

//////////////////////////////
// METHODS - ModuleSnapshot //
//////////////////////////////

impl ModuleSnapshot {
   /// Gets the address space range
   /// occupied by the module within
   /// the parent process.
   pub fn address_range<'l>(
      &'l self,
   ) -> &'l std::ops::Range<* const c_void> {
      return self.snap.address_range();
   }

   /// Gets the file name of the
   /// module.  This only includes
   /// the file name and extension
   /// without the containing file
   /// path.
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return self.snap.executable_file_name();
   }
}

