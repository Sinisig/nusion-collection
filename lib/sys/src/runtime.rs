//! Runtime initialization for custom
//! entrypoints.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Struct for keeping track of
/// runtime information.
pub struct Runtime {
}

///////////////////////
// METHODS - Runtime //
///////////////////////

impl Runtime {
   /// Initializes the runtime environment
   /// and executes an entrypoint with no
   /// return type.
   pub fn start_main_void<F>(
      entrypoint  : F,
   ) -> crate::os::runtime::OSReturn
   where F: FnOnce(),
   {
      entrypoint();
      return crate::os::runtime::EXIT_SUCCESS;
   }

   /// Initializes the runtime environment
   /// and executes an entrypoint with a
   /// Result<(), E> return type where E
   /// implements std::error::Error statically.
   pub fn start_main_result_static<F, E>(
      entrypoint  : F,
   ) -> crate::os::runtime::OSReturn
   where F: FnOnce() -> Result<(), E>,
         E: std::error::Error,
   {
      if let Err(err) = entrypoint() {
         eprintln!("Error: {err}");
         return crate::os::runtime::EXIT_FAILURE;
      }

      return crate::os::runtime::EXIT_SUCCESS;
   }

   /// Initializes the runtime environment
   /// and executes an entrypoint with a
   /// Result<(), Box<dyn std::error::Error>
   /// return type.
   pub fn start_main_result_dynamic<F>(
      entrypoint  : F,
   ) -> crate::os::runtime::OSReturn
   where F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
   {
      if let Err(err) = entrypoint() {
         eprintln!("Error: {err}");
         return crate::os::runtime::EXIT_FAILURE;
      }

      return crate::os::runtime::EXIT_SUCCESS;
   }
}

