//! crate::memory OS implementations
//! for Windows.

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

pub struct MemoryPermissions {
   permissions : DWORD
}

impl MemoryPermissions {
   pub const READ                : Self
      = Self{permissions : PAGE_READONLY           };

   pub const READ_WRITE          : Self
      = Self{permissions : PAGE_READWRITE          };

   pub const READ_EXECUTE        : Self
      = Self{permissions : PAGE_EXECUTE_READ       };

   pub const READ_WRITE_EXECUTE  : Self
      = Self{permissions : PAGE_EXECUTE_READWRITE  };

   pub const ALL : Self
      = Self::READ_WRITE_EXECUTE;
}

impl MemoryPermissions {
   pub fn set(
      address_range  : & std::ops::Range<usize>,
      permissions    : & Self,
   ) -> crate::memory::Result<Self> {
      // Get base address and byte count
      let base    = address_range.start;
      let bytes   = address_range.end - address_range.start;

      // Attempt to set page permissions
      let mut old_permissions = 0;
      if unsafe{VirtualProtect(
         base  as LPVOID,
         bytes as SIZE_T,
         permissions.permissions,
         & mut old_permissions,
      )} == TRUE {
         return Ok(Self{permissions : old_permissions});
      }

      // Parse error number into MemoryErrorKind
      use crate::memory::MemoryErrorKind::*;
      let errkind = match unsafe{GetLastError()} {
         _ => Unknown,
      };

      // Create the MemoryError and return
      return Err(crate::memory::MemoryError::new(
         errkind, address_range.clone(),
      ));
   }
}

