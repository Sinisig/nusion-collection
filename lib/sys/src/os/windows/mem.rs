//! crate::mem OS implementations for Windows.

use core::ffi::c_void;
use winapi::{
   shared::{
      basetsd::{
         SIZE_T,
      },
      minwindef::{
         DWORD,
         LPVOID,
         TRUE,
      },
   },
   um::{
      errhandlingapi::{
         GetLastError,
      },
      memoryapi::{
         VirtualProtect,
      },
      winnt::{
         PAGE_READONLY,
         PAGE_READWRITE,
         PAGE_EXECUTE_READ,
         PAGE_EXECUTE_READWRITE,
      },
   },
};

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Type used for storing memory
/// permission flags.
pub struct MemoryPermissions(DWORD);

///////////////////////////////////
// CONSTANTS - MemoryPermissions //
///////////////////////////////////

impl MemoryPermissions {
   pub const READ                : Self
      = Self(PAGE_READONLY);

   pub const READ_WRITE          : Self
      = Self(PAGE_READWRITE);

   pub const READ_EXECUTE        : Self
      = Self(PAGE_EXECUTE_READ);

   pub const READ_WRITE_EXECUTE  : Self
      = Self(PAGE_EXECUTE_READWRITE);

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
      // Get base address and byte count
      let base    = address_range.start;
      let bytes   = unsafe{address_range.end.offset_from(address_range.start)};

      // Attempt to set page permissions
      let mut old_permissions = 0;
      if unsafe{VirtualProtect(
         base  as LPVOID,
         bytes as SIZE_T,
         permissions.0,
         & mut old_permissions,
      )} == TRUE {
         return Ok(Self(old_permissions));
      }

      // Parse error number into MemoryErrorKind
      use crate::mem::MemoryErrorKind::*;
      let errkind = match unsafe{GetLastError()} {
         // TODO: Error code parsing
         _           => Unknown,
      };
      
      // Create the MemoryError and return
      return Err(crate::mem::MemoryError::new(
         errkind, address_range.clone(),
      ));
   }
}

