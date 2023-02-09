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
      = Self(crate::os::environment::EXIT_SUCCESS);

   /// Value when execution failed.
   pub const FAILURE  : Self
      = Self(crate::os::environment::EXIT_FAILURE);
}

////////////////////////
// METHODS - OSReturn //
////////////////////////

impl OSReturn {
   /// Gets the stored value in the OSReturn
   /// instance.
   pub fn get(
      & self,
   ) -> crate::os::environment::OSReturn {
      return self.0.clone();
   }
}

