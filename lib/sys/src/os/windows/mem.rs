//! crate::mem OS implementations for Windows.

use core::ffi::c_void;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Type used for storing memory
/// permission flags.
pub struct MemoryPermissions(u32);

///////////////////////////////////
// CONSTANTS - MemoryPermissions //
///////////////////////////////////

impl MemoryPermissions {
   pub const READ                : Self
      = Self(0);

   pub const READ_WRITE          : Self
      = Self(0);

   pub const READ_EXECUTE        : Self
      = Self(0);

   pub const READ_WRITE_EXECUTE  : Self
      = Self(0);

   pub const ALL : Self
      = Self::READ_WRITE_EXECUTE;
}

/////////////////////////////////
// METHODS - MemoryPermissions //
/////////////////////////////////

impl MemoryPermissions {
   pub fn set(
      address_range  : & std::ops::Range<* const c_void>,
      permissions    : & Self,
   ) -> crate::mem::Result<Self> {
      todo!();
   }
}

