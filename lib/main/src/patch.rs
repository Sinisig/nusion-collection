//! Module containing memory patching
//! utilities.

use nusion_sys as sys;
use core::ffi::c_void;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error type containing the reason
/// behind a patch creation failing.
#[derive(Debug)]
pub struct PatchError {
   kind  : PatchErrorKind,
}

/// An error enum containing the reason
/// behind a PatchError.
#[derive(Debug)]
pub enum PatchErrorKind {
   MemoryError(sys::mem::MemoryError),
}

/// A result type returned by patch
/// functions.
pub type Result<T> = std::result::Result<T, PatchError>;

/// Struct for creating and storing
/// various different memory patch
/// types.  Patched bytes will be
/// restored to their original values
/// automatically when going out of
/// scope via the
/// <a href=https://doc.rust-lang.org/std/ops/trait.Drop.html>
/// Drop
/// </a>
/// trait.
pub struct Patch {
   location    : std::ops::Range<* const c_void>,
   old_bytes   : Vec<u8>,
}

//////////////////////////
// METHODS - PatchError //
//////////////////////////

impl PatchError {
   pub fn kind<'l>(
      &'l self,
   ) -> &'l PatchErrorKind {
      return &self.kind;
   }
}

////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - PatchError //
////////////////////////////////////////

impl std::fmt::Display for PatchError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      use PatchErrorKind::*;
      return match self.kind() {
         MemoryError(err)
            => write!(stream, "Memory error: {err}"),
      };
   }
}

impl std::error::Error for PatchError {
}

impl From<sys::mem::MemoryError> for PatchError {
   fn from(
      value : sys::mem::MemoryError
   ) -> Self {
      return Self{
         kind: PatchErrorKind::MemoryError(value),
      };
   }
}

/////////////////////
// METHODS - Patch //
/////////////////////

impl Patch {

}

///////////////////////////////////
// TRAIT IMPLEMENTATIONS - Patch //
///////////////////////////////////

impl Drop for Patch {
   fn drop(
      & mut self,
   ) {
      unsafe{sys::mem::MemoryEditor::open_read_write(
         self.location.clone(),
      ).expect(
         "Failed to restore patched bytes",
      ).bytes_mut()}.clone_from_slice(&self.old_bytes);

      return;
   }
}

