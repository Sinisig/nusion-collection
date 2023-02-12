//! Various metadata and other information
//! for creating and maintaining a runtime
//! environment.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Return type for returning to the OS.
pub struct OSReturn(crate::os::environment::OSReturn);

//////////////////////////
// CONSTANTS - OSReturn //
//////////////////////////

impl OSReturn {
   /// Value when execution was successful.
   pub const SUCCESS  : Self
      = Self(crate::os::environment::OSReturn::SUCCESS);

   /// Value when execution failed.
   pub const FAILURE  : Self
      = Self(crate::os::environment::OSReturn::FAILURE);
}

//////////////////////////////////////
// TRAIT IMPLEMENTATIONS - OSReturn //
//////////////////////////////////////

impl std::ops::Deref for OSReturn {
   type Target = crate::os::environment::OSReturn;

   fn deref(
      & self,
   ) -> & Self::Target {
      return &self.0;
   }
}

