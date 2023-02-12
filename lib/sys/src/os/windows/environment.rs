//! crate::os::environment implementations
//! for Windows.

use winapi::{
   shared::{
      minwindef::{
         DWORD,
      },
   },
};

#[derive(Copy, Clone)]
pub struct OSReturn(DWORD);

impl OSReturn {
   pub const SUCCESS : Self
      = Self(0);

   pub const FAILURE : Self
      = Self(1);
}

impl std::ops::Deref for OSReturn {
   type Target = DWORD;

   fn deref(
      & self,
   ) -> & Self::Target {
      return &self.0;
   }
}

