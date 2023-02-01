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
///
/// <h2 id=  patch_note>
/// <a href=#patch_note>
/// Note
/// </a></h2>
///
/// Since the patch uses the Drop trait
/// to automatically clean up memory, a
/// Patch instance must have a real variable
/// binding to prevent going out of scope
/// and calling Drop too early.  This can
/// be accomplished by assigning a Patch
/// to a named variable.
///
/// <h2 id=  patch_safety>
/// <a href=#patch_safety>
/// Safety
/// </a></h2>
///
/// Every function to create a patch
/// requires quite a bit of care and
/// attention to prevent catastrophic
/// memory safety errors and crashes
/// from regularly occurring due to the
/// nature of overwriting arbitrary
/// memory locations with unrelated
/// byte data.  First, all safety concerns
/// from nusion_sys::mem::MemoryEditor::data_mut()
/// apply to every function to create a
/// patch.  Second, the patch data must
/// be valid for the context.  For example,
/// you should never overwrite machine code
/// with unrelated data.  It should only
/// be overwrote with machine code which
/// is valid for the surrounding code.
/// Some function variants are safer
/// (with an 'R') than others, such as
/// those which take a checksum to compare
/// against.  While they are safer, they
/// are still wildly unsafe.  This is the
/// Mariana Trench of undefined behavior,
/// so make sure to use this module when
/// sober and well-rested.  This library
/// doesn't come with any warranty of any
/// kind, so don't hold me accountable!
pub struct Patch {
   address_range  : std::ops::Range<* const c_void>,
   old_bytes      : Vec<u8>,
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
   /// Creates a patch using a user-defined
   /// closure to write new byte values to
   /// the memory region.  The closure will
   /// only be executed after the memory region
   /// has been successfully opened for reading
   /// and writing and a backup of the pre-patch
   /// bytes has been made.
   /// <h2 id=  patch_patch_with_safety>
   /// <a href=#patch_patch_with_safety>
   /// Safety
   /// </a></h2>
   /// See <a href=#patch_safety>Self</a>
   /// for safety concerns.
   pub unsafe fn with<F>(
      address_range  : std::ops::Range<* const c_void>,
      build_patch    : F,
   ) -> Result<Self>
   where F: FnOnce(& mut [u8]) -> Result<()> {
      let mut editor = sys::mem::MemoryEditor::open_read_write(
         address_range.clone(),
      )?;

      let old_bytes = editor.bytes().to_vec();

      build_patch(editor.bytes_mut())?;

      return Ok(Self{
         address_range  : address_range,
         old_bytes      : old_bytes,
      });
   }
}

///////////////////////////////////
// TRAIT IMPLEMENTATIONS - Patch //
///////////////////////////////////

impl Drop for Patch {
   fn drop(
      & mut self,
   ) {
      unsafe{sys::mem::MemoryEditor::open_read_write(
         self.address_range.clone(),
      ).expect(
         "Failed to restore patched bytes",
      ).bytes_mut()}.copy_from_slice(&self.old_bytes);

      return;
   }
}

