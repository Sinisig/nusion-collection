//! Module containing memory patching
//! utilities.

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
   MemoryError{
      sys_error   : crate::sys::mem::MemoryError
   },
   LengthMismatch{
      found       : usize,
      expected    : usize,
   },
   ResidualBytes{
      left        : usize,
      right       : usize,
   },
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

/// Enum for representing alignment
/// of data within a section of memory.
/// This is useful for specifying where
/// a byte slice should be positioned
/// within a larger section of memory.
#[derive(Debug)]
pub enum PatchAlignment {
   Left,
   LeftOffset{
      elements : usize,
   },
   LeftByteOffset{
      bytes    : usize,
   },
   Right,
   RightOffset{
      elements : usize,
   },
   RightByteOffset{
      bytes    : usize,
   },
   Center,
}

//////////////////////////
// METHODS - PatchError //
//////////////////////////

impl PatchError {
   /// Creates a new PatchError from
   /// a given PatchErrorKind.
   pub fn new(
      kind : PatchErrorKind,
   ) -> Self {
      return Self{
         kind : kind,
      };
   }

   /// Returns a reference to the
   /// stored PatchErrorKind.
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
         MemoryError    {sys_error,       }
            => write!(stream, "Memory error: {sys_error}",                          ),
         LengthMismatch {found, expected, }
            => write!(stream, "Length mismatch: Found {found}, expected {expected}",),
         ResidualBytes  {left, right,     }
            => write!(stream, "Residual bytes: {left} on left, {right} on right"),
      };
   }
}

impl std::error::Error for PatchError {
}

impl From<crate::sys::mem::MemoryError> for PatchError {
   fn from(
      value : crate::sys::mem::MemoryError
   ) -> Self {
      return Self::new(PatchErrorKind::MemoryError{
         sys_error : value,
      });
   }
}

//////////////////////////////
// METHODS - PatchAlignment //
//////////////////////////////

impl PatchAlignment {
   /// Returns the amount of left
   /// and right padding to insert
   /// given a buffer byte count
   /// and insert data byte count.
   /// The returned tuple is the
   /// amount of <b>elements</b>
   /// to be inserted before and
   /// after the source respectively.
   /// If there are an uneven number
   /// of bytes on either side or
   /// a byte offset count too large
   /// is passed in, an error is
   /// returned.
   pub fn padding_count<T>(
      & self,
      buffer_byte_count : usize,
      insert_byte_count : usize,
   ) -> Result<(usize, usize)> {
      if buffer_byte_count < insert_byte_count {
         return Err(PatchError::new(PatchErrorKind::LengthMismatch{
            found    : insert_byte_count,
            expected : buffer_byte_count,
         }));
      }

      let byte_pad_count   = buffer_byte_count - insert_byte_count;
      let element_size     = std::mem::size_of::<T>();

      let bytes_pad_left   = match self {
         Self::Left
            => 0,
         Self::LeftOffset        {elements}
            => *elements * element_size,
         Self::LeftByteOffset    {bytes   }
            => *bytes,
         Self::Right
            => byte_pad_count,
         Self::RightOffset       {elements}
            => byte_pad_count - *elements * element_size,
         Self::RightByteOffset   {bytes   }
            => byte_pad_count - *bytes,
         Self::Center
            => element_size * ((byte_pad_count / 2) / element_size),
      };
      let bytes_pad_right  = byte_pad_count - bytes_pad_left;

      let bytes_residual_left    = bytes_pad_left  % element_size;
      let bytes_residual_right   = bytes_pad_right % element_size;
      if bytes_residual_left != 0 || bytes_residual_right != 0 {
         return Err(PatchError::new(PatchErrorKind::ResidualBytes{
            left  : bytes_residual_left,
            right : bytes_residual_right,
         }));
      }

      let elements_left    = bytes_pad_left  / element_size;
      let elements_right   = bytes_pad_right / element_size;

      return Ok((elements_left, elements_right));
   }

   /// Fills a byte array with a
   /// slice type surrounded by
   /// padding values using the
   /// given alignment.
   pub fn clone_from_slice_with_padding<T, U>(
      & self,
      buffer   : & mut [u8],
      slice    : & [U],
      value    : T,
   ) -> Result<& Self>
   where T: Clone,
         U: Clone,
   {
      let (
         pad_count_left,
         pad_count_right,
      ) = self.padding_count::<T>(
         buffer.len(),
         slice.len(),
      )?;

      let size_of_t        = std::mem::size_of::<T>();
      let size_of_u        = std::mem::size_of::<U>();
      let byte_end_left    = pad_count_left * size_of_t;
      let byte_end_slice   = byte_end_left + (slice.len() * size_of_u);

      // Fill left padding
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            0..byte_end_left
         ].as_ptr() as * mut T,
         pad_count_left,
      )}.fill(value.clone());

      // Copy slice
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            byte_end_left..byte_end_slice
         ].as_ptr() as * mut U,
         slice.len(),
      )}.clone_from_slice(slice);

      // Fill right padding
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            byte_end_slice..
         ].as_ptr() as * mut T,
         pad_count_right,
      )}.fill(value.clone());

      return Ok(self);
   }
}

////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - PatchAlignment //
////////////////////////////////////////////

impl Default for PatchAlignment {
   fn default() -> Self {
      return Self::Center;
   }
}

/////////////////////
// METHODS - Patch //
/////////////////////

impl Patch {
   /// Creates a patch using a user-defined
   /// closure to write new byte values to
   /// the memory region.  The closure parameter
   /// is a mutable byte slice for the memory
   /// region of the patch.  The closure will
   /// only be executed after the memory region
   /// has been successfully opened for reading
   /// and writing and a backup of the pre-patch
   /// bytes has been made.
   /// <h2 id=  patch_new_safety>
   /// <a href=#patch_new_safety>
   /// Safety
   /// </a></h2>
   /// See <a href=#patch_safety>Self</a>
   /// for safety concerns.
   pub unsafe fn new<F>(
      address_range  : std::ops::Range<* const c_void>,
      build_patch    : F,
   ) -> Result<Self>
   where F: FnOnce(& mut [u8]) -> Result<()> {
      let mut editor = crate::sys::mem::MemoryEditor::open_read_write(
         address_range.clone(),
      )?;

      let patch = Self{
         address_range  : address_range,
         old_bytes      : editor.bytes().to_vec(),
      };

      build_patch(editor.bytes_mut())?;

      return Ok(patch);
   }

   /// Creates a patch by writing a
   /// slice of arbitrary elements
   /// using a byte alignment and
   /// padding out unfilled data
   /// with a specified padding value.
   /// If the slice data is too long
   /// or there are residual bytes
   /// left over in the padding,
   /// an error will be returned.
   ///
   /// <h2 id=  patch_patch_safety>
   /// <a href=#patch_patch_safety>
   /// Safety
   /// </a></h2>
   /// See <a href=#patch_safety>Self</a>
   /// for safety concerns.
   pub unsafe fn fill_with_padding<T, U>(
      address_range  : std::ops::Range<* const c_void>,
      slice          : & [T],
      padding        : U,
      alignment      : PatchAlignment,
   ) -> Result<Self>
   where T: Clone,
         U: Clone, 
   {
      return Self::new(address_range, |target| {
         alignment.clone_from_slice_with_padding(
            target, slice, padding,
         )?;
         Ok(())
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
      unsafe{crate::sys::mem::MemoryEditor::open_read_write(
         self.address_range.clone(),
      ).expect(
         "Failed to restore patched bytes",
      ).bytes_mut()}.copy_from_slice(&self.old_bytes);

      return;
   }
}

