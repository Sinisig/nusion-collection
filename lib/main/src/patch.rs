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
   ZeroLengthType,
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
         Self::ZeroLengthType
            => write!(stream, "Type has zero length for non-zero range length"),
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

   /// Fills a byte array with an
   /// item surrounded by padding
   /// values using the given
   /// alignment.
   pub fn clone_from_item_with_padding<T, U>(
      & self,
      buffer   : & mut [u8],
      item     : T,
      value    : U,
   ) -> Result<& Self>
   where U: Clone,
   {
      let size_of_t = std::mem::size_of::<T>();
      let size_of_u = std::mem::size_of::<U>();

      let (
         pad_count_left,
         pad_count_right,
      ) = self.padding_count::<U>(
         buffer.len(),
         1,
      )?;
 
      let byte_end_left    = pad_count_left * size_of_u;
      let byte_end_slice   = byte_end_left + size_of_t;

      // Fill left padding
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            0..byte_end_left
         ].as_ptr() as * mut U,
         pad_count_left,
      )}.fill(value.clone());

      // Copy slice
      let dest = buffer[
         byte_end_left..byte_end_slice
      ].as_ptr() as * mut T;

      unsafe{*dest = item};
 
      // Fill right padding
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            byte_end_slice..
         ].as_ptr() as * mut U,
         pad_count_right,
      )}.fill(value.clone());

      return Ok(self);
   }

   /// Fills a byte array with a
   /// slice type surrounded by
   /// padding values using the
   /// given alignment.
   pub fn clone_from_slice_with_padding<T, U>(
      & self,
      buffer   : & mut [u8],
      slice    : & [T],
      value    : U,
   ) -> Result<& Self>
   where T: Clone,
         U: Clone,
   {
      let size_of_t = std::mem::size_of::<T>();
      let size_of_u = std::mem::size_of::<U>();

      let (
         pad_count_left,
         pad_count_right,
      ) = self.padding_count::<U>(
         buffer.len(),
         slice.len(),
      )?;
 
      let byte_end_left    = pad_count_left * size_of_u;
      let byte_end_slice   = byte_end_left + (slice.len() * size_of_t);

      // Fill left padding
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            0..byte_end_left
         ].as_ptr() as * mut U,
         pad_count_left,
      )}.fill(value.clone());

      // Copy slice
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            byte_end_left..byte_end_slice
         ].as_ptr() as * mut T,
         slice.len(),
      )}.clone_from_slice(slice);

      // Fill right padding
      unsafe{std::slice::from_raw_parts_mut(
         buffer[
            byte_end_slice..
         ].as_ptr() as * mut U,
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

unsafe fn patch_buffer_item_padded<T, U>(
   buffer      : & mut [u8],
   item        : T,
   alignment   : Alignment,
   padding     : U,
) -> Result<()>
where T: Clone,
      U: Clone,
{
   alignment.clone_from_item_with_padding(
      buffer,
      item,
      padding,
   )?;

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

unsafe fn patch_buffer_slice_fill<T>(
   buffer   : & mut [u8],
   slice    : & [T],
) -> Result<()>
where T: Clone,
{
   if buffer.len() == 0 {
      return Ok(());
   }

   if slice.len() == 0 {
      return Err(PatchError::ZeroLengthType);
   }

   let slice_len_bytes = slice.len() * std::mem::size_of::<T>();

   if buffer.len() % slice_len_bytes != 0 {
      return Err(PatchError::ResidualBytes{
         left  : 0,
         right : buffer.len() % slice_len_bytes,
      });
   }

   // TODO: Actual copying
   todo!()
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
   buffer         : & mut [u8],
   code_location  : * const c_void,
) -> Result<()> {
   crate::sys::compiler::hook_fill(buffer, code_location)?;
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
/// <li>
/// <b>Item</b> - Writes a single item
/// to the memory region.  If the length
/// of the memory region doesn't match
/// the size of the item in bytes, an
/// error is returned.
/// </li>
///
/// <li>
/// <b>Item Fill</b> - Fill the memory
/// region with a single value.  If the
/// memory region can't be evenly filled,
/// an error is returned.
/// </li>
///
/// <li>
/// <b>Item Padded</b> - Writes a single
/// item to a memory region with the rest
/// of the memory region filled with a
/// padding value.  The position of the
/// item in the memory region is controlled
/// by an alignment argument.
/// </li>
///
/// <li>
/// <b>Slice</b> - Writes a single slice
/// to the memory region.  If the length
/// of the slice in bytes doesn't match
/// the length of the memory region, an
/// error is returned.
/// </li>
///
/// <li>
/// <b>Slice Fill</b> - Fill the memory
/// region with a single slice.  If the
/// memory region can't be evenly filled,
/// an error is returned.
/// </li>
///
/// <li>
/// <b>Slice Padded</b> - Writes a single
/// slice to a memory region with the rest
/// of the memory region filled with a
/// padding value.  The position of the
/// slice in the memory region is controlled
/// by an alignment argument.
/// </li>
///
/// <li>
/// <b>Nop</b> - Fill the memory region with
/// the architecture-dependent no-operation
/// instruction.  If it is impossible to
/// fill every byte of the memory region
/// with no-operation instructions, an error
/// is returned.
/// </li>
/// 
/// <li>
/// <b>Hook</b> - Compiles a call instruction
/// to a given code location in the memory
/// region and a jump instruction to skip
/// over the remaining bytes.  If it is
/// impossible to compile the call instruction
/// into the given memory region, an error
/// is returned.
/// </li>
///
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
   ////////////////////
   // REQUIRED ITEMS //
   ////////////////////

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
      & self,
      memory_range   : R,
   ) -> Result<T>
   where R: RangeBounds<usize>,
         T: Copy;

   /// Reads the bytes stored in the
   /// memory range as a slice of values.
   unsafe fn patch_read_slice<R, T>(
      & self,
      memory_range   : R,
   ) -> Result<Vec<T>>
   where R: RangeBounds<usize>,
         T: Copy;

   /// Writes values to a memory range
   /// using a predicate, checking
   /// against a checksum.
   unsafe fn patch_write_with<R, P>(
      & mut self,
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
      & mut self,
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
      & mut self,
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
      & mut self,
      memory_range   : R,
      predicate      : P,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         P: FnOnce(& mut [u8]) -> Result<()>;

   ////////////////////
   // PROVIDED ITEMS //
   ////////////////////

   // Item ////////////////////////////////////////////////////////////////////

   unsafe fn patch_write_item<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      item           : T,
   ) -> Result<()>
   where R: RangeBounds<usize>,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_item(buffer, item)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_item<R, T>(
      & mut self,
      memory_range   : R,
      item           : T,
   ) -> Result<()>
   where R: RangeBounds<usize>,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_item(buffer, item)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_item<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      item           : T,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_item(buffer, item)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_item<R, T>(
      & mut self,
      memory_range   : R,
      item           : T,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_item(buffer, item)?;
         return Ok(());
      });
   }

   // Item Fill ///////////////////////////////////////////////////////////////

   unsafe fn patch_write_item_fill<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      item           : T,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_item_fill(buffer, item)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_item_fill<R, T>(
      & mut self,
      memory_range   : R,
      item           : T,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_item_fill(buffer, item)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_item_fill<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      item           : T,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_item_fill(buffer, item)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_item_fill<R, T>(
      & mut self,
      memory_range   : R,
      item           : T,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_item_fill(buffer, item)?;
         return Ok(());
      });
   }

   // Item Padded /////////////////////////////////////////////////////////////

   unsafe fn patch_write_item_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      item           : T,
      alignment      : Alignment,
      padding        : U,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_item_padded(buffer, item, alignment, padding)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_item_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      item           : T,
      alignment      : Alignment,
      padding        : U,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_item_padded(buffer, item, alignment, padding)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_item_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      item           : T,
      alignment      : Alignment,
      padding        : U,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_item_padded(buffer, item, alignment, padding)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_item_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      item           : T,
      alignment      : Alignment,
      padding        : U,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_item_padded(buffer, item, alignment, padding)?;
         return Ok(());
      });
   }

   // Slice ///////////////////////////////////////////////////////////////////

   unsafe fn patch_write_slice<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      slice          : & [T],
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_slice(buffer, slice)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_slice<R, T>(
      & mut self,
      memory_range   : R,
      slice          : & [T],
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_slice(buffer, slice)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_slice<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      slice          : & [T],
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_slice(buffer, slice)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_slice<R, T>(
      & mut self,
      memory_range   : R,
      slice          : & [T],
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_slice(buffer, slice)?;
         return Ok(());
      });
   }

   // Slice Fill //////////////////////////////////////////////////////////////

   unsafe fn patch_write_slice_fill<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      slice          : & [T],
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_slice_fill(buffer, slice)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_slice_fill<R, T>(
      & mut self,
      memory_range   : R,
      slice          : & [T],
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_slice_fill(buffer, slice)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_slice_fill<R, T>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      slice          : & [T],
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_slice_fill(buffer, slice)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_slice_fill<R, T>(
      & mut self,
      memory_range   : R,
      slice          : & [T],
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_slice_fill(buffer, slice)?;
         return Ok(());
      });
   }

   // Slice Padded ////////////////////////////////////////////////////////////

   unsafe fn patch_write_slice_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      slice          : & [T],
      alignment      : Alignment,
      padding        : U,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_slice_padded(buffer, slice, alignment, padding)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_slice_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      slice          : & [T],
      alignment      : Alignment,
      padding        : U,
   ) -> Result<()>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_slice_padded(buffer, slice, alignment, padding)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_slice_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      slice          : & [T],
      alignment      : Alignment,
      padding        : U,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_slice_padded(buffer, slice, alignment, padding)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_slice_padded<R, T, U>(
      & mut self,
      memory_range   : R,
      slice          : & [T],
      alignment      : Alignment,
      padding        : U,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
         T: Clone,
         U: Clone,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_slice_padded(buffer, slice, alignment, padding)?;
         return Ok(());
      });
   }

   // Nop /////////////////////////////////////////////////////////////////////

   unsafe fn patch_write_nop<R>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
   ) -> Result<()>
   where R: RangeBounds<usize>,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_nop(buffer)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_nop<R>(
      & mut self,
      memory_range   : R,
   ) -> Result<()>
   where R: RangeBounds<usize>,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_nop(buffer)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_nop<R>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_nop(buffer)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_nop<R>(
      & mut self,
      memory_range   : R,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_nop(buffer)?;
         return Ok(());
      });
   }

   // Hook ////////////////////////////////////////////////////////////////////

   unsafe fn patch_write_hook<R>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      code_location  : * const c_void,
   ) -> Result<()>
   where R: RangeBounds<usize>,
   {
      return Self::patch_write_with(self, memory_range, checksum, |buffer| {
         patch_buffer_hook(buffer, code_location)?;
         return Ok(());
      });
   }

   unsafe fn patch_write_unchecked_hook<R>(
      & mut self,
      memory_range   : R,
      code_location  : * const c_void,
   ) -> Result<()>
   where R: RangeBounds<usize>,
   {
      return Self::patch_write_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_hook(buffer, code_location)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_hook<R>(
      & mut self,
      memory_range   : R,
      checksum       : Checksum,
      code_location  : * const c_void,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch_create_with(self, memory_range, checksum, |buffer| {
         patch_buffer_hook(buffer, code_location)?;
         return Ok(());
      });
   }

   unsafe fn patch_create_unchecked_hook<R>(
      & mut self,
      memory_range   : R,
      code_location  : * const c_void,
   ) -> Result<Self::Container>
   where R: RangeBounds<usize>,
   {
      return Self::patch_create_unchecked_with(self, memory_range, |buffer| {
         patch_buffer_hook(buffer, code_location)?;
         return Ok(());
      });
   }
}

