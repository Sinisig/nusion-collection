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
   CenterByte,
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

////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - PatchAlignment //
////////////////////////////////////////////

impl Default for PatchAlignment {
   fn default() -> Self {
      return Self::Center;
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
   alignment   : PatchAlignment,
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
/// memory access violation crash)
///
/// The overwritten memory location
/// is currently being accessed (race
/// condition).
///
/// The overwritten memory location
/// is not a valid place to overwrite
/// with new data.
///
/// The data used to overwrite the
/// memory location is not valid for
/// its purpose (ex: overwriting code
/// with non-code).
///
/// Any reference to code or data
/// in the patch data goes out of
/// scope, either by being dropped
/// by the compiler or by unloading
/// the module containing the code
/// or data.
pub unsafe trait Patch {
   /// The container used to store the
   /// patch metadata.  It is recommended
   /// to make this container store the
   /// overwritten byte data and then
   /// implement the Drop trait to then
   /// restore the overwritten bytes.
   type Container;

   /// Creates a patch at a given
   /// memory location offset using
   /// a predicate to write the bytes
   /// to the memory location.  The
   /// actual memory address written
   /// to depends on the implementation
   /// of the trait.
   unsafe fn patch<R, P>(
      & mut self,
      memory_offset_range  : R,
      predicate            : P,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         P: FnOnce(& mut [u8]) -> Result<()>;

   /// Writes a single item to a
   /// given memory offset range.
   /// If the length of the item
   /// in bytes doesn't match the
   /// memory offset range, an
   /// error will be returned.
   unsafe fn patch_item<R, T>(
      & mut self,
      memory_offset_range  : R,
      item                 : T,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch(self, memory_offset_range, |bytes| {
         patch_buffer_item(bytes, item)?;
         return Ok(());
      });
   }

   /// Fills a region of memory
   /// with a single repeated value.
   /// If there are unfillable bytes
   /// which are unable to fully contain
   /// a copy of the item, an error
   /// is returned.
   unsafe fn patch_item_fill<R, T>(
      & mut self,
      memory_offset_range  : R,
      item                 : T,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch(self, memory_offset_range, |bytes| {
         patch_buffer_item_fill(bytes, item)?;
         return Ok(());
      });
   }

   /// Writes a slice of items to
   /// a given memory offset range.
   /// If the item slice is not the
   /// same size in bytes as the
   /// memory offset range, an
   /// error is returned.
   unsafe fn patch_slice<R, T>(
      & mut self,
      memory_offset_range  : R,
      items                : & [T],
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch(self, memory_offset_range, |bytes| {
         patch_buffer_slice(bytes, items)?;
         return Ok(());
      });
   }

   /// Writes a slice of items to
   /// a given memory offset range.
   /// If the item slice is longer
   /// than the memory offset range
   /// in bytes, an error is returned.
   /// If the item slice is shorter
   /// than the memory offset range
   /// in bytes, a padding alignment
   /// and value is used to fill out
   /// the leftover bytes.  If there
   /// are still leftover bytes which
   /// are too small to fit the padding
   /// value, an error is returned.
   unsafe fn patch_slice_padded<R, T, U>(
      & mut self,
      memory_offset_range  : R,
      items                : & [T],
      alignment            : PatchAlignment,
      padding              : U,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch(self, memory_offset_range, |bytes| {
         patch_buffer_slice_padded(bytes, items, alignment, padding)?;
         return Ok(());
      });
   }

   /// Compiles a block of architecture
   /// no-operation instructions to fill
   /// the memory region.  The function
   /// will return an error if no possible
   /// instruction encodings can fill the
   /// entire memory region.
   unsafe fn patch_nop<R>(
      & mut self,
      memory_offset_range  : R,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>
   {
      return Self::patch(self, memory_offset_range, |bytes| {
         patch_buffer_nop(bytes)?;
         return Ok(());
      });
   }

   /// Compiles a jump to a given
   /// code location, padding the
   /// rest of the memory region with
   /// architecture-dependent no-operation
   /// instructions.  The function will
   /// return an error if no possible
   /// instruction encoding can fit
   /// within the memory region.
   unsafe fn patch_hook<R>(
      & mut self,
      memory_offset_range  : R,
      target_code_location : * const c_void,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch(self, memory_offset_range, |bytes| {
         patch_buffer_hook(bytes, target_code_location)?;
         return Ok(());
      });
   }
}

