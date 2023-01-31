//! Various functions used for modifying
//! arbitrary memory permissions and values.

use core::ffi::c_void;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Error information returned by a
/// failing memory function.
#[derive(Debug)]
pub struct MemoryError {
   kind           : MemoryErrorKind,
   address_range  : std::ops::Range<* const c_void>,
}

/// Error enum containing the kind
/// of error returned by a failing
/// memory function.
#[derive(Debug)]
pub enum MemoryErrorKind {
   PermissionDenied,
   InvalidAddressRange,
   UnmappedAddress,
   Unknown,
}

/// Result type returned by falliable
/// functions.
pub type Result<T> = std::result::Result<T, MemoryError>;

/// Struct for opening up memory for
/// reading and writing and accessing
/// said memory.  Memory permissions
/// will be restored automatically
/// when the struct goes out of scope
/// via the <a href="https://doc.rust-lang.org/std/ops/trait.Drop.html">Drop</a>
/// trait.
pub struct MemoryEditor {
   address_range     : std::ops::Range<* const c_void>,
   old_permissions   : crate::os::mem::MemoryPermissions,
}

///////////////////////////
// METHODS - MemoryError //
///////////////////////////

impl MemoryError {
   /// Creates a new MemoryError from a kind
   /// enum variant and a memory address range.
   pub fn new(
      kind           : MemoryErrorKind,
      address_range  : std::ops::Range<* const c_void>,
   ) -> Self {
      return Self{
         kind           : kind,
         address_range  : address_range,
      }
   }

   /// Retrieves the error kind variant
   /// belonging to the error.
   pub fn kind<'l>(
      &'l self,
   ) -> &'l MemoryErrorKind {
      return &self.kind;
   }

   /// Gets the address range relating to
   /// the memory error.
   pub fn address_range<'l>(
      &'l self,
   ) -> &'l std::ops::Range<* const c_void> {
      return &self.address_range;
   }
}

/////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - MemoryError //
/////////////////////////////////////////

impl std::fmt::Display for MemoryError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream,
         "{err} {start:#0fill$x} - {end:#0fill$x}",
         err   = self.kind(),
         start = self.address_range().start as usize,
         end   = self.address_range().end   as usize,
         fill  = std::mem::size_of::<usize>() * 2 + 2,
      );
   }
}

impl std::error::Error for MemoryError {
}

/////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - MemoryErrorKind //
/////////////////////////////////////////////

impl std::fmt::Display for MemoryErrorKind {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::PermissionDenied
            => "Permission denied",
         Self::InvalidAddressRange
            => "Invalid address range",
         Self::UnmappedAddress
            => "Address not mapped",
         Self::Unknown
            => "Unknown",
      });
   }
}

/////////////////////////////////////
// INTERNAL HELPERS - MemoryEditor //
/////////////////////////////////////

impl MemoryEditor {
   fn open_with_permissions(
      address_range     : std::ops::Range<* const c_void>,
      new_permissions   : crate::os::mem::MemoryPermissions,
   ) -> Result<Self> {
      let old_permissions = crate::os::mem::MemoryPermissions::set(
         &address_range,
         &new_permissions,
      )?;

      return Ok(Self{
         address_range     : address_range,
         old_permissions   : old_permissions,
      });
   }
}

////////////////////////////
// METHODS - MemoryEditor //
////////////////////////////

impl MemoryEditor {
   /// Attempts to open a range of memory
   /// for reading.
   pub fn open_read(
      address_range  : std::ops::Range<* const c_void>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::mem::MemoryPermissions::READ,
      );
   }

   /// Attempts to open a range of memory
   /// for reading and writing.
   pub fn open_read_write(
      address_range  : std::ops::Range<* const c_void>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::mem::MemoryPermissions::READ_WRITE,
      );
   }

   /// Attempts to open a range of memory
   /// for reading and code execution.
   pub fn open_read_execute(
      address_range  : std::ops::Range<* const c_void>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::mem::MemoryPermissions::READ_EXECUTE,
      );
   }

   /// Attempts to open a range of memory
   /// for reading, writing, and code
   /// execution.
   pub fn open_read_write_execute(
      address_range  : std::ops::Range<* const c_void>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::mem::MemoryPermissions::READ_WRITE_EXECUTE,
      );
   }

   /// Attempts to open a range of memory
   /// with all memory access permissions.
   pub fn open_all(
      address_range  : std::ops::Range<* const c_void>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::mem::MemoryPermissions::ALL,
      );
   }

   /// Creates a slice type referencing
   /// the data in the stored memory location.
   ///
   /// <h2 id =  safety_data>
   /// <a href="#safety_data">
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <a href="https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html">std::slice::from_raw_parts()</a>
   /// apply.
   ///
   /// In addition, since the data was created
   /// from raw pointers, the data may change
   /// in unexpected ways and lead to undefined
   /// behavior.
   pub unsafe fn data<'l, T>(
      &'l self,
   ) -> &'l [T] {
      let start      = self.address_range.start;
      let end        = self.address_range.end;
      let byte_count = end.offset_from(end) as usize;

      return std::slice::from_raw_parts(
         start as * const T,
         byte_count / std::mem::size_of::<T>(),
      );
   }

   /// Creates a mutable slice type referencing
   /// the data in the stored memory location.
   ///
   /// <h2 id =  safety_data_mut>
   /// <a href="#safety_data_mut">
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <a href="#safety_data">Self::data()</a>
   /// apply.
   ///
   /// In addition, trying to call Self::data_mut()
   /// on a MemoryEditor created without write permissions
   /// is undefined behavior and will very likely lead
   /// to a crash when attempting to modify the stored
   /// data.
   pub unsafe fn data_mut<'l, T>(
      &'l mut self,
   ) -> &'l mut [T] {
      let start      = self.address_range.start;
      let end        = self.address_range.end;
      let byte_count = end.offset_from(end) as usize;

      return std::slice::from_raw_parts_mut(
         start as * mut T,
         byte_count / std::mem::size_of::<T>(),
      );
   }

   /// Creates a byte slice type referencing
   /// the bytes in the stored memory location.
   ///
   /// <h2 id =  safety_bytes>
   /// <a href="#safety_bytes">
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <a href="#safety_data">Self::data()</a>
   /// apply.
   pub unsafe fn bytes<'l>(
      &'l self,
   ) -> &'l [u8] {
      return self.data::<u8>();
   }

   /// Creates a mutable byte slice type
   /// referencing the bytes in the stored
   /// memory location.
   ///
   /// <h2 id =  safety_bytes_mut>
   /// <a href="#safety_bytes_mut">
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <a href="#safety_data_mut">Self::data_mut()</a>
   /// apply.
   pub unsafe fn bytes_mut<'l>(
      &'l mut self,
   ) -> &'l mut [u8] {
      return self.data_mut::<u8>();
   }
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - MemoryEditor //
//////////////////////////////////////////

impl Drop for MemoryEditor {
   fn drop(
      & mut self,
   ) { 
      crate::os::mem::MemoryPermissions::set(
         &self.address_range,
         &self.old_permissions,
      ).expect(
         "Failed to restore memory permissions",
      );
      return;
   }
}

