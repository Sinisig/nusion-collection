//! Memory patching traits and
//! implementations.

use std::ops::RangeBounds;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error relating to a patch function.
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
      residual    : usize,
   },
   ResidualBytesDouble{
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
   OutOfRange{
      maximum     : usize,
      provided    : usize,
   },
   EndOffsetBeforeStartOffset,
   ZeroLengthType,
}

/// <code>Result</code> type with error
/// variant <code>PatchError</code>
pub type Result<T> = std::result::Result<T, PatchError>;

/// Enum for representing alignment
/// of data within a section of memory.
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checksum {
   checksum : u32,
}

/// Type which stores a pointer to
/// a hook function.  The associated
/// function should be generated with
/// the <code>hook!</code> macro instead
/// of manually defining a function.
type HookTarget = unsafe extern "C" fn();

/// Collection of provided structs
/// which implement the <code>Reader</code>
/// trait for use in <code>Patch</code>
/// structs.
pub mod reader {
   use super::*;

   /// Reads a single item which
   /// implements the <code>Copy</code>
   /// trait.
   #[derive(Debug)]
   pub struct Item<
      R: RangeBounds<usize>,
      T: Copy,
   > {
      pub marker              : std::marker::PhantomData<* const T>,
      pub memory_offset_range : R,
   }

   /// Reads a slice of items which
   /// implement the <code>Copy</code>
   /// trait.
   #[derive(Debug)]
   pub struct Slice<
      R: RangeBounds<usize>,
      T: Copy,
   > {
      pub marker              : std::marker::PhantomData<* const T>,
      pub memory_offset_range : R,
      pub element_count       : usize,
   }
}

/// Collection of provided structs
/// which implement the <code>Writer</code>
/// trait for use in <code>Patch</code>
/// structs.
pub mod writer {
   use super::*;

   /// Clones a single item.
   #[derive(Debug)]
   pub struct Item<
      's,
      R: RangeBounds<usize>,
      T: Clone,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub item                : &'s T,
   }

   /// Repeatedly clones a single item
   /// to fill the memory buffer.
   #[derive(Debug)]
   pub struct ItemFill<
      's,
      R: RangeBounds<usize>,
      T: Clone,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub item                : &'s T,
   }

   /// Positions and clones a single
   /// element according to the alignment
   /// and fills the surrounding bytes
   /// with the cloned padding value.
   #[derive(Debug)]
   pub struct ItemPadded<
      's,
      R: RangeBounds<usize>,
      T: Clone,
      U: Clone,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub alignment           : Alignment,
      pub item                : &'s T,
      pub padding             : &'s U,
   }

   /// Clones a single slice.
   #[derive(Debug)]
   pub struct Slice<
      's,
      R: RangeBounds<usize>,
      T: Clone,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub slice               : &'s [T],
   }

   /// Repeatedly clones a single slice
   /// to fill the memory buffer.
   #[derive(Debug)]
   pub struct SliceFill<
      's,
      R: RangeBounds<usize>,
      T: Clone,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub slice               : &'s [T],
   }

   /// Positions and clones a single
   /// slice according to the alignment
   /// and fills the surrounding bytes
   /// with the cloned padding value.
   #[derive(Debug)]
   pub struct SlicePadded<
      's,
      R: RangeBounds<usize>,
      T: Clone,
      U: Clone,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub alignment           : Alignment,
      pub slice               : &'s [T],
      pub padding             : &'s U,
   }

   /// Compiles a block of architecture-dependent
   /// no-operation (nop) machine-code
   /// instructions.
   #[derive(Debug)]
   pub struct Nop<
      R: RangeBounds<usize>,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
   }

   /// Compiles a call to a given assembly
   /// subroutine, filling the rest of the
   /// bytes with architecture-dependent
   /// no-operation (nop) instructions.
   /// It is recommended to use the <code>hook!</code>
   /// macro to generate your hook.
   #[derive(Debug)]
   pub struct Hook<
      R: RangeBounds<usize>,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub hook                : HookTarget,
   }

   /// Copies a byte buffer containing
   /// assembly instructions into the
   /// memory offset range according
   /// to the alignment.  Any unfilled
   /// bytes are overwritten with
   /// architecture-dependent no-operation
   /// (nop) instructions.  It is recommended
   /// to use the <code>asm_bytes!</code>
   /// macro to generate the byte slice.
   #[derive(Debug)]
   pub struct Asm<
      R: RangeBounds<usize>,
   > {
      pub memory_offset_range : R,
      pub checksum            : Checksum,
      pub alignment           : Alignment,
      pub asm_bytes           : &'static [u8],
   }
}

///////////////////////
// TRAIT DEFINITIONS //
///////////////////////

/// Opens a region of memory for
/// patching and applies a patch
/// using some type implementing
/// either the <code>Reader</code>
/// or <code>Writer</code> trait,
/// depending on whether a reader
/// or writer function is used.
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
/// <code><a href=
/// https://doc.rust-lang.org/std/mem/fn.transmute.html>std::mem::transmute
/// </a></code>, and in many ways even more
/// unsafe.  In addition to all the
/// memory safety concerns from transmute,
/// any of the following will lead
/// to <b>undefined behavior</b> (usually
/// a memory access violation crash):
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
   /// implement the <code>Drop</code>
   /// trait to then restore the overwritten
   /// bytes.
   type Container;

   /// Reads the bytes stored in the
   /// memory range and converts them
   /// to their appropriate type using
   /// a reader.
   unsafe fn patch_read<Rd, Mr>(
      & self,
      reader : & Rd,
   ) -> Result<Rd::Item>
   where Rd: Reader<Mr>,
         Mr: RangeBounds<usize>;

   /// Writes a patch using a patcher
   /// without saving the overwritten
   /// bytes, checking against a checksum.
   unsafe fn patch_write<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> Result<()>
   where Wt: Writer<Mr>,
         Mr: RangeBounds<usize>;

   /// Writes a patch using a writer
   /// without saving the overwritten
   /// bytes.
   unsafe fn patch_write_unchecked<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> Result<()>
   where Wt: Writer<Mr>,
         Mr: RangeBounds<usize>;

   /// Creates a patch using a writer,
   /// storing the overwritten bytes in
   /// the specified container.
   unsafe fn patch_create<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> Result<Self::Container>
   where Wt: Writer<Mr>,
         Mr: RangeBounds<usize>;

   /// Creates a patch using a writer,
   /// storing the overwritten bytes in
   /// the specified container.
   unsafe fn patch_create_unchecked<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> Result<Self::Container>
   where Wt: Writer<Mr>,
         Mr: RangeBounds<usize>;
}

/// Trait for reading byte data from
/// a memory buffer and transforming
/// the byte data into some useful type.
/// This is the trait which reads bytes
/// from memory.
pub trait Reader<R: RangeBounds<usize>> {
   /// The item type which is returned
   /// by <code>read_item</code>.
   type Item;

   /// Returns the stored memory offset
   /// range in the reader.
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R;

   /// Reads the byte slice and converts
   /// it to a valid value of type
   /// <code>Self::Container</code>.
   fn read_item(
      & self,
      memory_buffer  : & [u8],
   ) -> Result<Self::Item>;
}

/// Trait for storing patch metadata
/// and later applying the patch to
/// some memory buffer.  This is the
/// trait which writes bytes to memory.
pub trait Writer<R: RangeBounds<usize>> {
   /// Returns the stored memory offset
   /// range in the writer.
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R;

   /// Returns the stored checksum
   /// for the writer.
   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum;

   /// Builds the patch and writes it
   /// to the memory buffer.  The input
   /// memory buffer should be a slice
   /// to the actual memory location.
   /// Copying the slice can break
   /// many patch implementations.
   fn build_patch(
      & self,
      memory_buffer  : & mut [u8],
   ) -> Result<()>;
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
         Self::MemoryError                {sys_error,       }
            => write!(stream, "Memory error: {sys_error}",                          ),
         Self::LengthMismatch             {found, expected, }
            => write!(stream, "Length mismatch: Found {found}, expected {expected}",),
         Self::ResidualBytes              {residual,        }
            => write!(stream, "{residual} leftover residual bytes"),
         Self::ResidualBytesDouble        {left, right,     }
            => write!(stream, "Residual bytes: {left} on left, {right} on right"),
         Self::CompilationError           {sys_error,       }
            => write!(stream, "Compilation error: {sys_error}"),
         Self::ChecksumMismatch           {found, expected, }
            => write!(stream, "Checksum mismatch: Found {found}, expected {expected}"),
         Self::OutOfRange                 {maximum, provided}
            => write!(stream, "Out of range: Maximum of {maximum} bytes, provided {provided} bytes"),
         Self::EndOffsetBeforeStartOffset
            => write!(stream, "End offset is before start offset"),
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
            => {
               0
            },
         Self::LeftOffset        {elements}
            => {
               let bytes = *elements * element_size;
               if bytes > byte_pad_count {
                  return Err(PatchError::OutOfRange{
                     maximum  : byte_pad_count,
                     provided : bytes,
                  });
               }

               bytes
            },
         Self::LeftByteOffset    {bytes   }
            => {
               let bytes = *bytes;
               if bytes > byte_pad_count {
                  return Err(PatchError::OutOfRange{
                     maximum  : byte_pad_count,
                     provided : bytes,
                  });
               }

               bytes            
            },
         Self::Right
            => {
               byte_pad_count
            },
         Self::RightOffset       {elements}
            => {
               let bytes = *elements * element_size;
               if bytes > byte_pad_count {
                  return Err(PatchError::OutOfRange{
                     maximum  : byte_pad_count,
                     provided : bytes,
                  });
               }

               byte_pad_count - bytes
            },
         Self::RightByteOffset   {bytes   }
            => {
               let bytes = *bytes;
               if bytes > byte_pad_count {
                  return Err(PatchError::OutOfRange{
                     maximum  : byte_pad_count,
                     provided : bytes,
                  });
               }

               byte_pad_count - bytes
            },
         Self::Center
            => {
               element_size * ((byte_pad_count / 2) / element_size)
            },
         Self::CenterByte
            => {
               byte_pad_count / 2
            },
      };
      let bytes_pad_right  = byte_pad_count - bytes_pad_left;

      let bytes_residual_left    = bytes_pad_left  % element_size;
      let bytes_residual_right   = bytes_pad_right % element_size;
      if bytes_residual_left != 0 || bytes_residual_right != 0 {
         return Err(PatchError::ResidualBytesDouble{
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
         size_of_t,
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

      // Copy item
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
   /// Creates a new checksum from
   /// the provided byte data.
   pub fn new(
      data  : & [u8],
   ) -> Self {
      let checksum = crc::Crc::<u32>::new(
         &crc::CRC_32_CKSUM,
      ).checksum(data);

      return Self{
         checksum : checksum,
      };
   }

   /// Creates a checksum from an
   /// existing checksum value.
   pub const fn from(
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

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - reader::Item //
//////////////////////////////////////////

impl<
   R: RangeBounds<usize>,
   T: Copy,
> Reader<R> for reader::Item<R, T> {
   type Item = T;

   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn read_item(
      & self,
      memory_buffer  : & [u8],
   ) -> Result<Self::Item> {
      let item_size = std::mem::size_of::<T>();

      if memory_buffer.len() != item_size {
         return Err(PatchError::LengthMismatch{
            found    : memory_buffer.len(),
            expected : item_size,
         })
      }

      // This looks sketchy, but since we have
      // the Copy trait bound and checked the
      // length with the above code, this will
      // always be valid given the memory buffer
      // is also valid.
      let item_ptr   = memory_buffer.as_ptr() as * const T;
      let item       = unsafe{*item_ptr};

      return Ok(item);
   }
}

///////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - reader::Slice //
///////////////////////////////////////////

impl<
   R: RangeBounds<usize>,
   T: Copy,
> Reader<R> for reader::Slice<R, T> {
   type Item = Vec<T>;

   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn read_item(
      & self,
      memory_buffer  : & [u8],
   ) -> Result<Self::Item> {
      let item_size  = std::mem::size_of::<T>();
      let byte_count = self.element_count * item_size;

      if memory_buffer.len() < byte_count {
         return Err(PatchError::LengthMismatch{
            found    : memory_buffer.len(),
            expected : byte_count,
         });
      }

      if item_size == 0 {
         return Ok(Vec::new());
      }

      let bytes_residual = memory_buffer.len() % item_size;
      if bytes_residual != 0 {
         return Err(PatchError::ResidualBytes{
            residual : bytes_residual,
         });
      }

      // Again, looks sketchy but the above code
      // verifies this is sound
      let item_slice = unsafe{std::slice::from_raw_parts(
         memory_buffer.as_ptr() as * const T,
         self.element_count,
      )};
      let item_vec   = item_slice.to_vec();

      return Ok(item_vec);
   }
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::Item //
//////////////////////////////////////////

impl<
   's,
   R: RangeBounds<usize>,
   T: Clone,
> Writer<R> for writer::Item<'s, R, T> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   } 

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      let item_size = std::mem::size_of::<T>();

      if memory_buffer.len() != item_size {
         return Err(PatchError::LengthMismatch{
            found    : memory_buffer.len(),
            expected : item_size,
         });
      }

      let destination = memory_buffer.as_mut_ptr() as * mut T;

      unsafe{*destination = self.item.clone()};

      return Ok(());
   }
}

//////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::ItemFill //
//////////////////////////////////////////////

impl<
   's,
   R: RangeBounds<usize>,
   T: Clone,
> Writer<R> for writer::ItemFill<'s, R, T> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      let residual = memory_buffer.len() % std::mem::size_of::<T>();

      if residual != 0 {
         return Err(PatchError::ResidualBytes{
            residual : residual,
         });
      }

      let bytes = unsafe{std::slice::from_raw_parts_mut(
         memory_buffer.as_mut_ptr() as * mut T,
         memory_buffer.len() / std::mem::size_of::<T>(),
      )};

      bytes.fill(self.item.clone());

      return Ok(());
   }
}

////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::ItemPadded //
////////////////////////////////////////////////

impl<
   's,
   R: RangeBounds<usize>,
   T: Clone,
   U: Clone,
> Writer<R> for writer::ItemPadded<'s, R, T, U> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      self.alignment.clone_from_item_with_padding(
         memory_buffer,
         self.item.clone(),
         self.padding.clone(),
      )?;

      return Ok(());
   }
}

///////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::Slice //
///////////////////////////////////////////

impl<
   's,
   R: RangeBounds<usize>,
   T: Clone,
> Writer<R> for writer::Slice<'s, R, T> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      let slice = unsafe{std::slice::from_raw_parts(
         self.slice.as_ptr() as * const u8,
         self.slice.len() * std::mem::size_of::<T>(),
      )};

      if memory_buffer.len() != slice.len() {
         return Err(PatchError::LengthMismatch{
            found    : slice.len(),
            expected : memory_buffer.len(),
         });
      }

      memory_buffer.clone_from_slice(slice);

      return Ok(());
   }
}

///////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::SliceFill //
///////////////////////////////////////////////

impl<
   's,
   R: RangeBounds<usize>,
   T: Clone,
> Writer<R> for writer::SliceFill<'s, R, T> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      if memory_buffer.len() == 0 {
         return Ok(());
      }

      if self.slice.len() == 0 {
         return Err(PatchError::ZeroLengthType);
      }

      let slice_len_bytes = self.slice.len() * std::mem::size_of::<T>();

      if memory_buffer.len() % slice_len_bytes != 0 {
         return Err(PatchError::ResidualBytes{
            residual : memory_buffer.len() % slice_len_bytes,
         });
      }

      // This is how the sausage is made
      // Have to create Vec copies so we
      // call clone() and can still access
      // the raw bytes.  Before you ask,
      // std::slice::clone_from_slice()
      // doesn't work for this use case.
      let mut memory_buffer_view = & mut memory_buffer[..];
      while memory_buffer_view.len() != 0 {
         // Clone slice elements and convert to byte slice
         let slice_clone = self.slice.to_vec();
         let slice_clone = unsafe{std::slice::from_raw_parts(
            slice_clone.as_ptr() as * const u8,
            slice_len_bytes,
         )};

         // Copy to the beginning of the buffer view
         memory_buffer_view[..slice_clone.len()].copy_from_slice(slice_clone);

         // Isolate the buffer view to cut off the written bytes
         memory_buffer_view = & mut memory_buffer_view[slice_clone.len()..];
      }

      return Ok(());
   }
}

/////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::SlicePadded //
/////////////////////////////////////////////////

impl<
   's,
   R: RangeBounds<usize>,
   T: Clone,
   U: Clone,
> Writer<R> for writer::SlicePadded<'s, R, T, U> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      self.alignment.clone_from_slice_with_padding(
         memory_buffer,
         self.slice,
         self.padding.clone(),
      )?;

      return Ok(());
   }
}

/////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::Nop //
/////////////////////////////////////////

impl<
   R: RangeBounds<usize>,
> Writer<R> for writer::Nop<R> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      crate::sys::compiler::nop_fill(
         memory_buffer,
      )?;
      return Ok(());
   }
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::Hook //
//////////////////////////////////////////

impl<
   R: RangeBounds<usize>,
> Writer<R> for writer::Hook<R> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      crate::sys::compiler::hook_fill(
         memory_buffer,
         self.hook,
      )?;
      return Ok(());
   }
}

/////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - writer::Asm //
/////////////////////////////////////////

impl<
   R: RangeBounds<usize>,
> Writer<R> for writer::Asm<R> {
   fn memory_offset_range<'l>(
      &'l self,
   ) -> &'l R {
      return & self.memory_offset_range;
   }

   fn checksum<'l>(
      &'l self,
   ) -> &'l Checksum {
      return & self.checksum;
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      // Verify the ASM will fit into the buffer
      if memory_buffer.len() < self.asm_bytes.len() {
         return Err(PatchError::LengthMismatch{
            found    : self.asm_bytes.len(),
            expected : memory_buffer.len(),
         });
      }

      // Byte padding count
      let padding_bytes_left = self.alignment.padding_count::<u8>(
         memory_buffer.len(),
         self.asm_bytes.len(),
      )?.0;

      // Copy the ASM bytes
      memory_buffer[
         padding_bytes_left..padding_bytes_left+self.asm_bytes.len()
      ].copy_from_slice(self.asm_bytes);

      // Build the padding instructions
      crate::sys::compiler::nop_fill(& mut memory_buffer[
         ..padding_bytes_left
      ])?;
      crate::sys::compiler::nop_fill(& mut memory_buffer[
         padding_bytes_left+self.asm_bytes.len()..
      ])?;

      return Ok(());
   }
}

