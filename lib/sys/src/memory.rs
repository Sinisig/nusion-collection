//! Various functions used for modifying
//! arbitrary memory permissions and values.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Error information returned by a
/// failing memory function.
#[derive(Debug)]
pub struct MemoryError {
   kind           : MemoryErrorKind,
   address_range  : std::ops::Range<usize>,
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

/// Result type with error
/// variant <code>MemoryError</code>
pub type Result<T> = std::result::Result<T, MemoryError>;

/// Struct for opening up memory for
/// reading and writing and accessing
/// said memory.  Memory permissions
/// will be restored automatically
/// when the struct goes out of scope
/// via the <code><a href=
/// "https://doc.rust-lang.org/std/ops/trait.Drop.html">Drop
/// </a></code> trait.
pub struct MemoryEditor {
   address_range     : std::ops::Range<usize>,
   old_permissions   : crate::os::memory::MemoryPermissions,
}

///////////////////////////
// METHODS - MemoryError //
///////////////////////////

impl MemoryError {
   /// Creates a new MemoryError from a kind
   /// enum variant and a memory address range.
   pub fn new(
      kind           : MemoryErrorKind,
      address_range  : std::ops::Range<usize>,
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
   ) -> &'l std::ops::Range<usize> {
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
         start = self.address_range().start,
         end   = self.address_range().end,
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
      address_range     : std::ops::Range<usize>,
      new_permissions   : crate::os::memory::MemoryPermissions,
   ) -> Result<Self> {
      if address_range.end < address_range.start {
         return Err(MemoryError::new(
            MemoryErrorKind::InvalidAddressRange,
            address_range,
         ));
      }

      let old_permissions = crate::os::memory::MemoryPermissions::set(
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
      address_range  : std::ops::Range<usize>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::memory::MemoryPermissions::READ,
      );
   }

   /// Attempts to open a range of memory
   /// for reading and writing.
   pub fn open_read_write(
      address_range  : std::ops::Range<usize>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::memory::MemoryPermissions::READ_WRITE,
      );
   }

   /// Attempts to open a range of memory
   /// for reading and code execution.
   pub fn open_read_execute(
      address_range  : std::ops::Range<usize>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::memory::MemoryPermissions::READ_EXECUTE,
      );
   }

   /// Attempts to open a range of memory
   /// for reading, writing, and code
   /// execution.
   pub fn open_read_write_execute(
      address_range  : std::ops::Range<usize>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::memory::MemoryPermissions::READ_WRITE_EXECUTE,
      );
   }

   /// Attempts to open a range of memory
   /// with all memory access permissions.
   pub fn open_all(
      address_range  : std::ops::Range<usize>,
   ) -> Result<Self> {
      return Self::open_with_permissions(
         address_range,
         crate::os::memory::MemoryPermissions::ALL,
      );
   }

   /// Creates a slice type referencing
   /// the data in the stored memory location.
   ///
   /// <h2 id=  memory_editor_as_slice_safety>
   /// <a href=#memory_editor_as_slice_safety>
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <code><a href=
   /// "https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html">std::slice::from_raw_parts</a></code>
   /// apply.
   ///
   /// In addition, since the data was created
   /// from raw pointers, the data may change
   /// in unexpected ways and lead to undefined
   /// behavior.
   ///
   /// <h2 id=  memory_editor_as_slice_panics>
   /// <a href=#memory_editor_as_slice_panics>
   /// Panics
   /// </a></h2>
   /// If the size of <code>T</code> is zero
   /// or attempting to create the slice leaves
   /// residual bytes which cannot be packed
   /// into <code>T</code>, the thread will
   /// panic.
   pub unsafe fn as_slice<'l, T>(
      &'l self,
   ) -> &'l [T] {
      let start      = self.address_range.start;
      let end        = self.address_range.end;
      let byte_count = end - start;
      let item_size  = std::mem::size_of::<T>();

      if item_size == 0 {
         panic!("Byte size of item is zero");
      }
      if byte_count % item_size != 0 {
         panic!("Residual bytes after last element");
      }

      return std::slice::from_raw_parts(
         start as * const T,
         byte_count / item_size,
      );
   }

   /// Creates a mutable slice type referencing
   /// the data in the stored memory location.
   ///
   /// <h2 id=  memory_editor_as_slice_mut_safety>
   /// <a href=#memory_editor_as_slice_mut_safety>
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <code><a href=
   /// #memory_editor_as_slice_safety>as_slice</a></code>
   /// apply.
   ///
   /// In addition, trying to call <code>as_slice_mut</code>
   /// on a MemoryEditor created without write permissions
   /// is undefined behavior and will very likely lead
   /// to a crash when attempting to modify the stored
   /// data.
   ///
   /// <h2 id=  memory_editor_as_slice_mut_panics>
   /// <a href=#memory_editor_as_slice_mut_panics>
   /// Panics
   /// </a></h2>
   /// This function will panic under the same
   /// conditions as <code><a href=
   /// #memory_editor_as_slice_panics>as_slice</a></code>.
   pub unsafe fn as_slice_mut<'l, T>(
      &'l mut self,
   ) -> &'l mut [T] {
      let start      = self.address_range.start;
      let end        = self.address_range.end;
      let byte_count = end - start;
      let item_size  = std::mem::size_of::<T>();

      if item_size == 0 {
         panic!("Byte size of item is zero");
      }
      if byte_count % item_size != 0 {
         panic!("Residual bytes after last element");
      }

      return std::slice::from_raw_parts_mut(
         start as * mut T,
         byte_count / item_size,
      );
   }

   /// Creates a byte slice type referencing
   /// the bytes in the stored memory location.
   ///
   /// <h2 id=  memory_editor_as_bytes_safety>
   /// <a href=#memory_editor_as_bytes_safety>
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <code><a href=
   /// "#memory_editor_as_slice_safety">MemoryEditor::as_slice</a></code>
   /// apply.
   pub unsafe fn as_bytes<'l>(
      &'l self,
   ) -> &'l [u8] {
      return self.as_slice::<u8>();
   }

   /// Creates a mutable byte slice type
   /// referencing the bytes in the stored
   /// memory location.
   ///
   /// <h2 id =  memory_editor_as_bytes_mut_safety>
   /// <a href="#memory_editor_as_bytes_mut_safety">
   /// Safety
   /// </a></h2>
   /// All safety concerns from
   /// <code><a href=
   /// "#memory_editor_as_slice_mut_safety">MemoryEditor::as_slice_mut</a></code>
   /// apply.
   pub unsafe fn as_bytes_mut<'l>(
      &'l mut self,
   ) -> &'l mut [u8] {
      return self.as_slice_mut::<u8>();
   }
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - MemoryEditor //
//////////////////////////////////////////

impl Drop for MemoryEditor {
   fn drop(
      & mut self,
   ) { 
      crate::os::memory::MemoryPermissions::set(
         &self.address_range,
         &self.old_permissions,
      ).expect(
         "Failed to restore memory permissions",
      );
      return;
   }
}

