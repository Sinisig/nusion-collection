//! Runtime initialization for custom
//! entrypoints.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Error type for when the runtime
/// fails to initialize.
#[derive(Debug)]
pub struct RuntimeInitError{
   kind  : RuntimeInitErrorKind,
}

/// Error kind for RuntimeInitError
#[derive(Debug)]
pub enum RuntimeInitErrorKind{
   ConsoleAllocFailure{
      err : crate::os::console::ConsoleError,
   },
}

/// Struct for keeping track of
/// runtime information.
pub struct Runtime {
   console  : crate::os::console::Console,
}

////////////////////////////////
// METHODS - RuntimeInitError //
////////////////////////////////

impl RuntimeInitError {
   /// Creates a new RuntimeInitError from
   /// a RuntimeInitErrorKind.
   pub fn new(
      kind : RuntimeInitErrorKind,
   ) -> Self {
      return Self{
         kind : kind,
      };
   }

   /// Gets a reference to the stored
   /// RuntimeInitErrorKind.
   pub fn kind<'l>(
      &'l self,
   ) -> &'l RuntimeInitErrorKind {
      return &self.kind;
   }
}

//////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - RuntimeInitError //
//////////////////////////////////////////////

impl std::fmt::Display for RuntimeInitError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", self.kind());
   }
}

impl std::error::Error for RuntimeInitError {
}

//////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - RuntimeInitErrorKind //
//////////////////////////////////////////////////

impl std::fmt::Display for RuntimeInitErrorKind {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return match self {
         Self::ConsoleAllocFailure{err}
            => write!(stream, "Console allocation failure: {err}"),
      };
   }
}

////////////////////////////////
// INTERNAL METHODS - Runtime //
////////////////////////////////

impl Runtime {
   fn init() -> Result<Self, RuntimeInitError> {
      // Create a console with the title "Nusion Console"
      let console = crate::os::console::Console::new(
         "Nusion Console",
      ).map_err(|err| {
         RuntimeInitError::new(RuntimeInitErrorKind::ConsoleAllocFailure{
            err : err,
         })
      })?;

      return Ok(Self{
         console  : console,
      });
   }
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
      let _runtime = Self::init().expect("Failed to initialize runtime");

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
      let _runtime = Self::init().expect("Failed to initialize runtime");

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
      let _runtime = Self::init().expect("Failed to initialize runtime");

      if let Err(err) = entrypoint() {
         eprintln!("Error: {err}");
         return crate::os::runtime::EXIT_FAILURE;
      }

      return crate::os::runtime::EXIT_SUCCESS;
   }

   /// Gets a reference to the stored
   /// console.
   pub fn console<'l>(
      &'l self,
   ) -> &'l crate::os::console::Console {
      return &self.console;
   }

   /// Gets a mutable reference to the
   /// stored console.
   pub fn console_mut<'l>(
      &'l mut self,
   ) -> &'l mut crate::os::console::Console {
      return & mut self.console;
   }
}

/////////////////////////////////////
// TRAIT IMPLEMENTATIONS - Runtime //
/////////////////////////////////////

impl Drop for Runtime {
   fn drop(
      & mut self
   ) {
      return;
   }
}

