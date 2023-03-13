//! Information for managing the
//! program's runtime environment.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Return type for returning to the OS.
pub struct OSReturn {
   code : crate::os::environment::OSReturn,
}

//////////////////////////
// CONSTANTS - OSReturn //
//////////////////////////

impl OSReturn {
   /// Return code when execution was successful.
   pub const SUCCESS : Self
      = Self{
         code : crate::os::environment::OSReturn::SUCCESS,
      };

   /// Return code when execution failed.
   pub const FAILURE : Self
      = Self{
         code : crate::os::environment::OSReturn::FAILURE,
      };
}

//////////////////////////////////////
// TRAIT IMPLEMENTATIONS - OSReturn //
//////////////////////////////////////

impl std::ops::Deref for OSReturn {
   type Target = crate::os::environment::OSReturn;

   fn deref(
      & self,
   ) -> & Self::Target {
      return &self.code;
   }
}

