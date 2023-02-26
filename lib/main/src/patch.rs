//! Module containing memory patching
//! utilities.

use core::ffi::c_void;
use std::ops::RangeBounds;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error type containing the reason
/// behind a patch creation failing.
#[derive(Debug)]
pub enum PatchError {
   MemoryError{
      sys_error   : crate::sys::memory::MemoryError
   },
   LengthMismatch{
      found       : usize,
      expected    : usize,
   },
   ResidualBytes{
      left        : usize,
      right       : usize,
   },
   CompilationError{
      sys_error   : crate::sys::compiler::CompilationError,
   },
   ChecksumMismatch{
      found       : Checksum,
      expected    : Checksum,
   },
}

/// A result type returned by patch
/// functions.
pub type Result<T> = std::result::Result<T, PatchError>;

/// Enum for representing alignment
/// of data within a section of memory.
/// This is useful for specifying where
/// a byte slice should be positioned
/// within a larger section of memory.
#[derive(Debug)]
pub enum Alignment {
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
   CenterByte,
}

/// Struct for storing and verifying
/// stored byte data for a patch.
#[derive(Debug, Eq, PartialEq)]
pub struct Checksum {
   checksum : u32,
}

////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - PatchError //
////////////////////////////////////////

impl std::fmt::Display for PatchError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return match self {
         Self::MemoryError       {sys_error,       }
            => write!(stream, "Memory error: {sys_error}",                          ),
         Self::LengthMismatch    {found, expected, }
            => write!(stream, "Length mismatch: Found {found}, expected {expected}",),
         Self::ResidualBytes     {left, right,     }
            => write!(stream, "Residual bytes: {left} on left, {right} on right"),
         Self::CompilationError  {sys_error,       }
            => write!(stream, "Compilation error: {sys_error}"),
         Self::ChecksumMismatch  {found, expected, }
            => write!(stream, "Checksum mismatch: Found {found}, expected {expected}"),
      };
   }
}

impl std::error::Error for PatchError {
}

impl From<crate::sys::memory::MemoryError> for PatchError {
   fn from(
      value : crate::sys::memory::MemoryError,
   ) -> Self {
      return Self::MemoryError{
         sys_error : value,
      };
   }
}

impl From<crate::sys::compiler::CompilationError> for PatchError {
   fn from(
      value : crate::sys::compiler::CompilationError,
   ) -> Self {
      return Self::CompilationError{
         sys_error : value,
      };
   }
}

/////////////////////////
// METHODS - Alignment //
/////////////////////////

impl Alignment {
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
         return Err(PatchError::LengthMismatch{
            found    : insert_byte_count,
            expected : buffer_byte_count,
         });
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
         Self::CenterByte
            => byte_pad_count / 2,
      };
      let bytes_pad_right  = byte_pad_count - bytes_pad_left;

      let bytes_residual_left    = bytes_pad_left  % element_size;
      let bytes_residual_right   = bytes_pad_right % element_size;
      if bytes_residual_left != 0 || bytes_residual_right != 0 {
         return Err(PatchError::ResidualBytes{
            left  : bytes_residual_left,
            right : bytes_residual_right,
         });
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

///////////////////////////////////////
// TRAIT IMPLEMENTATIONS - Alignment //
///////////////////////////////////////

impl Default for Alignment {
   fn default() -> Self {
      return Self::Center;
   }
}

////////////////////////
// METHODS - Checksum //
////////////////////////

impl Checksum {
   /// Creates a new Checksum from
   /// the provided byte data.
   pub fn new(
      data  : & [u8],
   ) -> Self {
      // TODO: Better checksum algorithm
      let checksum = data.iter().map(|b| *b as u32).sum();

      return Self{
         checksum : checksum,
      };
   }

   /// Creates a Checksum from an
   /// existing checksum value.
   pub fn from(
      checksum : u32,
   ) -> Self {
      return Self{
         checksum : checksum,
      };
   }
}

//////////////////////////////////////
// TRAIT IMPLEMENTATIONS - Checksum //
//////////////////////////////////////

impl std::fmt::Display for Checksum {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream,
         "{}",
         self.checksum,
      );
   }
}

//////////////////////////////
// INTERNAL HELPERS - Patch //
//////////////////////////////

unsafe fn patch_buffer_item<T>(
   buffer   : & mut [u8],
   item     : T,
) -> Result<()> {
   let item_size = std::mem::size_of::<T>();

   if buffer.len() != item_size {
      return Err(PatchError::LengthMismatch{
         found    : buffer.len(),
         expected : item_size,
      });
   }

   let destination = buffer.as_mut_ptr() as * mut T;

   *destination = item;

   return Ok(());
}

unsafe fn patch_buffer_item_fill<T>(
   buffer   : & mut [u8],
   item     : T,
) -> Result<()>
where T: Clone,
{
   let residual = buffer.len() % std::mem::size_of::<T>();

   if residual != 0 {
      return Err(PatchError::ResidualBytes{
         left  : 0,
         right : residual,
      });
   }

   let bytes = std::slice::from_raw_parts_mut(
      buffer.as_mut_ptr() as * mut T,
      buffer.len() / std::mem::size_of::<T>(),
   );

   bytes.fill(item);

   return Ok(());
}

unsafe fn patch_buffer_slice<T>(
   buffer   : & mut [u8],
   items    : & [T],
) -> Result<()>
where T: Clone,
{
   let items = std::slice::from_raw_parts(
      items.as_ptr() as * const u8,
      items.len() * std::mem::size_of::<T>(),
   );

   if buffer.len() != items.len() {
      return Err(PatchError::LengthMismatch{
         found    : items.len(),
         expected : buffer.len(),
      });
   }

   buffer.clone_from_slice(items);

   return Ok(());
}

unsafe fn patch_buffer_slice_padded<T, U>(
   buffer      : & mut [u8],
   items       : & [T],
   alignment   : Alignment,
   padding     : U,
) -> Result<()>
where T: Clone,
      U: Clone,
{
   alignment.clone_from_slice_with_padding(
      buffer,
      items,
      padding,
   )?;

   return Ok(());
}

unsafe fn patch_buffer_nop(
   buffer   : & mut [u8],
) -> Result<()> {
   crate::sys::compiler::nop_fill(buffer)?;
   return Ok(());
}

unsafe fn patch_buffer_hook(
   buffer               : & mut [u8],
   target_code_location : * const c_void,
) -> Result<()> {
   crate::sys::compiler::hook_fill(buffer, target_code_location)?;
   return Ok(());
}

//////////////////////////////
// TRAIT DEFINITION - Patch //
//////////////////////////////

/// Implements various memory patching
/// functions for a given type.
///
/// Currently, there are the following
/// patch types, each with respective
/// write/create and checked/unchecked
/// versions:
/// <ul>
/// <li>Test</li>
/// </ul>
///
/// <h2 id=  patch_safety>
/// <a href=#patch_safety>
/// Safety
/// </a></h2>
///
/// This is by far the most unsafe
/// part of this library.  To put
/// into perspective, this is about
/// as unsafe as
/// <a href=https://doc.rust-lang.org/std/mem/fn.transmute.html>
/// std::mem::transmute()
/// </a>, and in many ways even more
/// unsafe.  In addition to all the
/// memory safety concerns from transmute,
/// any of the following will lead
/// to undefined behavior (usually a
/// memory access violation crash):
///
/// <ul>
/// <li>
/// The overwritten memory location
/// is currently being accessed (race
/// condition).
/// </li>
///
/// <li>
/// The overwritten memory location
/// is not a valid place to overwrite
/// with new data.
/// </li>
///
/// <li>
/// The data used to overwrite the
/// memory location is not valid for
/// its purpose (ex: overwriting code
/// with non-code).
/// </li>
///
/// <li>
/// Any reference to code or data
/// in the patch data goes out of
/// scope, either by being dropped
/// by the compiler or by unloading
/// the module containing the code
/// or data.
/// </li>
/// </ul>
pub trait Patch {
   /// The container used to store the
   /// patch metadata.  It is recommended
   /// to make this container store the
   /// overwritten byte data and then
   /// implement the Drop trait to then
   /// restore the overwritten bytes.
   type Container;

   /// Reads the bytes stored in the
   /// memory range as a single value.
   unsafe fn patch_read_item<R, T>(
      memory_range   : R,
   ) -> Result<T>
   where R: RangeBounds<usize>,
         T: Clone;

   /// Reads the bytes stored in the
   /// memory range as a slice of values.
   unsafe fn patch_read_slice<R, T>(
      memory_range   : R,
   ) -> Result<Vec<T>>
   where R: RangeBounds<usize>,
         T: Clone;

   /// Writes values to a memory range
   /// using a predicate, checking
   /// against a checksum.
   unsafe fn patch_write_with<R, P>(
      memory_range   : R,
      checksum       : Checksum,
      predicate      : P,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         P: FnOnce(& mut [u8]) -> Result<()>;

   /// Writes values to a memory range
   /// using a predicate without checking
   /// against a checksum.
   unsafe fn patch_write_unchecked_with<R, P>(
      memory_range   : R,
      predicate      : P,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         P: FnOnce(& mut [u8]) -> Result<()>;

   /// Writes values to a memory range
   /// using a predicate, checking
   /// against a checksum and storing
   /// the old bytes in Self::Container.
   unsafe fn patch_create_with<R, P>(
      memory_range   : R,
      checksum       : Checksum,
      predicate      : P,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         P: FnOnce(& mut [u8]) -> Result<()>;

   /// Writes values to a memory range
   /// using a predicate, storing the
   /// old bytes in Self::Container
   /// without checking against a
   /// checksum.
   unsafe fn patch_create_unchecked_with<R, P>(
      memory_range   : R,
      predicate      : P,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         P: FnOnce(& mut [u8]) -> Result<()>;
}

