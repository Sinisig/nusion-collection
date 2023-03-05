//! Module containing memory patching
//! utilities.

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checksum {
   checksum : u32,
}

/// Collection of structs which implement
/// the Patcher trait.
pub mod method {
   use super::*;

   #[derive(Clone, Debug)]
   pub struct Nop {
      pub memory_offset_range : std::ops::Range<usize>,
      pub checksum            : Checksum,
   }

   #[derive(Clone, Debug)]
   pub struct Hook {
      pub memory_offset_range : std::ops::Range<usize>,
      pub checksum            : Checksum,
      pub target_hook         : * const core::ffi::c_void,
   }
}

///////////////////////
// TRAIT DEFINITIONS //
///////////////////////

/// Opens a region of memory for
/// patching and applies a patch
/// using some type implementing
/// the Patcher trait.  This is
/// the trait which opens up a
/// section of memory for patching.
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
   unsafe fn patch_read_item<T>(
      & self,
      memory_range : std::ops::Range<usize>,
   ) -> Result<T>
   where T: Copy;

   /// Reads the bytes stored in the
   /// memory range as a slice of values.
   unsafe fn patch_read_slice<T>(
      & self,
      memory_range : std::ops::Range<usize>,
   ) -> Result<Vec<T>>
   where T: Copy;

   /// Writes a patch using a patcher
   /// without saving the overwritten
   /// bytes, checking against a checksum.
   unsafe fn patch_write<P>(
      & mut self,
      patcher : & P,
   ) -> Result<()>
   where P: Patcher;

   /// Writes a patch using a patcher
   /// without saving the overwritten
   /// bytes.
   unsafe fn patch_write_unchecked<P>(
      & mut self,
      patcher : & P,
   ) -> Result<()>
   where P: Patcher;

   /// Creates a patch using a patcher,
   /// storing the overwritten bytes in
   /// the specified container.
   unsafe fn patch_create<P>(
      & mut self,
      patcher : & P,
   ) -> Result<Self::Container>
   where P: Patcher;

   /// Creates a patch using a patcher,
   /// storing the overwritten bytes in
   /// the specified container.
   unsafe fn patch_create_unchecked<P>(
      & mut self,
      patcher : & P,
   ) -> Result<Self::Container>
   where P: Patcher;
}

/// Trait for storing patch metadata
/// and later applying the patch to
/// some type implementing the Patch
/// trait.  This is the trait which
/// writes bytes to memory.
pub trait Patcher {
   /// Returns the stored memory offset
   /// range in the patch.
   fn memory_offset_range(
      & self,
   ) -> std::ops::Range<usize>;

   /// Returns the stored checksum
   /// for the patch.
   fn checksum(
      & self,
   ) -> Checksum;

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
         Self::MemoryError          {sys_error,       }
            => write!(stream, "Memory error: {sys_error}",                          ),
         Self::LengthMismatch       {found, expected, }
            => write!(stream, "Length mismatch: Found {found}, expected {expected}",),
         Self::ResidualBytes        {residual,        }
            => write!(stream, "{residual} leftover residual bytes"),
         Self::ResidualBytesDouble  {left, right,     }
            => write!(stream, "Residual bytes: {left} on left, {right} on right"),
         Self::CompilationError     {sys_error,       }
            => write!(stream, "Compilation error: {sys_error}"),
         Self::ChecksumMismatch     {found, expected, }
            => write!(stream, "Checksum mismatch: Found {found}, expected {expected}"),
         Self::OutOfRange           {maximum, provided}
            => write!(stream, "Out of range: Maximum of {maximum} bytes, provided {provided} bytes"),
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
   /// Creates a new Checksum from
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

/////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - method::Nop //
/////////////////////////////////////////

impl Patcher for method::Nop {
   fn memory_offset_range(
      & self,
   ) -> std::ops::Range<usize> {
      return self.memory_offset_range.clone();
   }

   fn checksum(
      & self,
   ) -> Checksum {
      return self.checksum.clone();
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
// TRAIT IMPLEMENTATIONS - method::Hook //
//////////////////////////////////////////

impl Patcher for method::Hook {
   fn memory_offset_range(
      & self,
   ) -> std::ops::Range<usize> {
      return self.memory_offset_range.clone();
   }

   fn checksum(
      & self,
   ) -> Checksum {
      return self.checksum.clone();
   }  

   fn build_patch(
      & self,
      memory_buffer : & mut [u8],
   ) -> Result<()> {
      unsafe{crate::sys::compiler::hook_fill(
         memory_buffer,
         self.target_hook,
      )}?;
      return Ok(());
   }
}








/*
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
         residual : residual,
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
         residual : buffer.len() % slice_len_bytes,
      });
   }

   // This is how the sausage is made
   // Have to create Vec copies so we
   // call clone() and can still access
   // the raw bytes.  Before you ask,
   // std::slice::clone_from_slice()
   // doesn't work for this use case.
   let mut buffer_view = & mut buffer[..];
   while buffer_view.len() != 0 {
      // Clone slice elements and convert to byte slice
      let slice_clone = slice.to_vec();
      let slice_clone = std::slice::from_raw_parts(
         slice_clone.as_ptr() as * const u8,
         slice_len_bytes,
      );

      // Copy to the beginning of the buffer view
      buffer_view[..slice_clone.len()].copy_from_slice(slice_clone);

      // Isolate the buffer view to cut off the written bytes
      buffer_view = & mut buffer_view[slice_clone.len()..];
   }

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
*/

