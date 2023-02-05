//! Environment initialization and main
//! thread entrypoint creation.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Error type for when the environment
/// fails to initialize.
#[derive(Debug)]
pub struct EnvironmentInitError{
   kind  : EnvironmentInitErrorKind,
}

/// Error kind for EnvironmentInitError
#[derive(Debug)]
pub enum EnvironmentInitErrorKind{
   ConsoleAllocFailure{
      err : crate::sys::console::ConsoleError,
   },
}

/// Struct for keeping track of
/// environment information.
pub struct Environment {
   console  : crate::sys::console::Console,
}

///////////////////////////////
// INTERNAL GLOBAL VARIABLES //
///////////////////////////////

static mut MAIN_THREAD_ENVIRONMENT : Option<std::sync::Mutex<Environment>>
   = None;

///////////////
// FUNCTIONS //
///////////////

/// Gets a reference to the main thread's
/// environment information.
pub fn environment() -> &'static Environment {
   return environment_mut();
}

/// Gets a mutable reference to the main thread's
/// environment information.
pub fn environment_mut() -> &'static mut Environment {
   return unsafe{MAIN_THREAD_ENVIRONMENT.as_mut().expect(
      "Environment is uninitialized",
   )}.get_mut().expect(
      "Failed to unlock environment mutex",
   );
}

////////////////////////////////////
// METHODS - EnvironmentInitError //
////////////////////////////////////

impl EnvironmentInitError {
   /// Creates a new EnvironmentInitError from
   /// a EnvironmentInitErrorKind.
   pub fn new(
      kind : EnvironmentInitErrorKind,
   ) -> Self {
      return Self{
         kind : kind,
      };
   }

   /// Gets a reference to the stored
   /// EnvironmentInitErrorKind.
   pub fn kind<'l>(
      &'l self,
   ) -> &'l EnvironmentInitErrorKind {
      return &self.kind;
   }
}

//////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - EnvironmentInitError //
//////////////////////////////////////////////////

impl std::fmt::Display for EnvironmentInitError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", self.kind());
   }
}

impl std::error::Error for EnvironmentInitError {
}

//////////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - EnvironmentInitErrorKind //
//////////////////////////////////////////////////////

impl std::fmt::Display for EnvironmentInitErrorKind {
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

////////////////////////////////////
// INTERNAL METHODS - Environment //
////////////////////////////////////

impl Environment {
   fn init() -> Result<Self, EnvironmentInitError> {
      let console = crate::sys::console::Console::new().map_err(|err| {
         EnvironmentInitError::new(EnvironmentInitErrorKind::ConsoleAllocFailure{
            err : err,
         })
      })?;

      return Ok(Self{
         console  : console,
      });
   }
}

///////////////////////////
// METHODS - Environment //
///////////////////////////

impl Environment {
   /// Initializes the thread environment
   /// and executes an entrypoint with no
   /// return type.
   ///
   /// <h2   id=note_environment_start_main_result_static>
   /// <a href=#note_environment_start_main_result_static>
   /// Note
   /// </a></h2>
   /// This function should never be called directly.
   /// Instead use the nusion::entry attribute macro
   /// to register a function as the designated entrypoint.
   pub fn start_main_void<F>(
      entrypoint  : F,
   ) -> crate::sys::env::OSReturn
   where F: FnOnce(),
   {
      let environment = Self::init().expect(
         "Failed to initialize environment",
      );

      if unsafe{MAIN_THREAD_ENVIRONMENT.is_none()} == false {
         panic!("Attempted to re-initialize environment");
      } else {
         unsafe{MAIN_THREAD_ENVIRONMENT = Some(
            std::sync::Mutex::new(environment)
         )};
      }

      entrypoint();

      unsafe{MAIN_THREAD_ENVIRONMENT = None};
      return crate::sys::env::OSReturn::SUCCESS;
   }

   /// Initializes the thread environment
   /// and executes an entrypoint with a
   /// Result<(), E> return type where E
   /// implements std::error::Error statically.
   ///
   /// <h2   id=note_environment_start_main_result_static>
   /// <a href=#note_environment_start_main_result_static>
   /// Note
   /// </a></h2>
   /// This function should never be called directly.
   /// Instead use the nusion::entry attribute macro
   /// to register a function as the designated entrypoint.
   pub fn start_main_result_static<F, E>(
      entrypoint  : F,
   ) -> crate::sys::env::OSReturn
   where F: FnOnce() -> Result<(), E>,
         E: std::error::Error,
   {
      let environment = Self::init().expect(
         "Failed to initialize environment",
      );

      if unsafe{MAIN_THREAD_ENVIRONMENT.is_none()} == false {
         panic!("Attempted to re-initialize environment");
      } else {
         unsafe{MAIN_THREAD_ENVIRONMENT = Some(
            std::sync::Mutex::new(environment)
         )};
      }

      if let Err(err) = entrypoint() {
         eprintln!("Error: {err}");
         return crate::sys::env::OSReturn::FAILURE;
      }

      unsafe{MAIN_THREAD_ENVIRONMENT = None};
      return crate::sys::env::OSReturn::SUCCESS;
   }

   /// Initializes the thread environment
   /// and executes an entrypoint with a
   /// Result<(), Box<dyn std::error::Error>
   /// return type.
   /// 
   /// <h2   id=note_environment_start_main_result_static>
   /// <a href=#note_environment_start_main_result_static>
   /// Note
   /// </a></h2>
   /// This function should never be called directly.
   /// Instead use the nusion::entry attribute macro
   /// to register a function as the designated entrypoint.
   pub fn start_main_result_dynamic<F>(
      entrypoint  : F,
   ) -> crate::sys::env::OSReturn
   where F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
   {
      let environment = Self::init().expect(
         "Failed to initialize environment",
      );

      if unsafe{MAIN_THREAD_ENVIRONMENT.is_none()} == false {
         panic!("Attempted to re-initialize environment");
      } else {
         unsafe{MAIN_THREAD_ENVIRONMENT = Some(
            std::sync::Mutex::new(environment)
         )};
      } 

      if let Err(err) = entrypoint() {
         eprintln!("Error: {err}");
         return crate::sys::env::OSReturn::FAILURE;
      }

      unsafe{MAIN_THREAD_ENVIRONMENT = None};
      return crate::sys::env::OSReturn::SUCCESS;
   }

   /// Gets a reference to the stored
   /// console.
   pub fn console<'l>(
      &'l self,
   ) -> &'l crate::sys::console::Console {
      return &self.console;
   }

   /// Gets a mutable reference to the
   /// stored console.
   pub fn console_mut<'l>(
      &'l mut self,
   ) -> &'l mut crate::sys::console::Console {
      return & mut self.console;
   }
}

