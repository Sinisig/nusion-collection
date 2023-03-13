//! crate::os::environment implementations
//! for Windows.

use winapi::{
   shared::{
      minwindef::{
         DWORD,
      },
   },
};

pub struct OSReturn {
   pub code : DWORD,
}

impl OSReturn {
   pub const SUCCESS : Self
      = Self{code : 0};

   pub const FAILURE : Self
      = Self{code : 1};
}

