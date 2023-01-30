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
   UnmappedAddress,
   Unknown,
}

/// Result type returned by falliable
/// functions.
pub type Result<T> = std::result::Result<T, MemoryError>;

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
         Self::UnmappedAddress
            => "Address not mapped",
         Self::Unknown
            => "Unknown",
      });
   }
}

