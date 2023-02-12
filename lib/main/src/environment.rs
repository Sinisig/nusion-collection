//! Environment initialization and main
//! thread entrypoint creation.

use std::sync::{Mutex, MutexGuard};

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error relating to the environment.
#[derive(Debug)]
pub enum EnvironmentError {
   ConsoleError{
      err : crate::console::ConsoleError,
   },
   PoisonedContext,
}

/// Result type with Err variant
/// EnvironmentError.
pub type Result<T> = std::result::Result<T, EnvironmentError>;

/// Struct for keeping track of
/// environment information.
pub struct Environment {
   console  : crate::console::Console,
}

//////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - EnvironmentError //
//////////////////////////////////////////////

impl std::fmt::Display for EnvironmentError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return match self {
         Self::ConsoleError{err}
            => write!(stream, "Console error: {err}"),
         Self::PoisonedContext
            => write!(stream, "Environment context is poisoned"),
      };
   }
}

impl std::error::Error for EnvironmentError {
}

impl From<crate::console::ConsoleError> for EnvironmentError {
   fn from(
      item : crate::console::ConsoleError,
   ) -> Self {
      return Self::ConsoleError{
         err : item,
      };
   }
}

impl<T> From<std::sync::PoisonError<T>> for EnvironmentError {
   fn from(
      _ : std::sync::PoisonError<T>,
   ) -> Self {
      return Self::PoisonedContext;
   }
}

////////////////////////////////////
// INTERNAL METHODS - Environment //
////////////////////////////////////

// Rust compiler: Noooo! You can't
// create uninitialized mutable
// global variables!  It's not
// thread safe and violates
// encapsulation!
//
// Me: Haha, unsafe{} go brrrr
// Segmentation fault (core dumped)
// 
// ...
//
// Please make sure to initialize
// this variable :)
static mut ENVIRONMENT_GLOBAL_STATE
   : Environment
   = unsafe{std::mem::MaybeUninit::uninit().assume_init()};

lazy_static::lazy_static!{
static ref ENVIRONMENT_GLOBAL_STATE_GUARD
   : Mutex<&'static mut Environment>
   = Mutex::new(unsafe{&mut ENVIRONMENT_GLOBAL_STATE});
}

impl Environment {
   /// For the love of god, call this
   /// function before EVER using the
   /// global context.  Also never call
   /// this more than once without a
   /// global_state_free() call.
   unsafe fn global_state_init(self) {
      // Done to prevent compiler from calling
      // Drop on the uninitialized state which
      // will almost certaintly cause a crash.
      std::mem::forget(std::mem::replace(
         &mut ENVIRONMENT_GLOBAL_STATE, self,
      ));

      return;
   }

   /// Clears the global state, freeing
   /// all items in it.  Don't even think
   /// about calling this function then
   /// using the global state.  Fails if
   /// the mutex guard dies in transit.
   /// Calling twice in a row without
   /// initializing again is undefined
   /// behavior.
   unsafe fn global_state_free() -> Result<()> {
      // Done like this to block until every thread
      // is done accessing the environment.
      let _guard = ENVIRONMENT_GLOBAL_STATE_GUARD.lock()?;
      ENVIRONMENT_GLOBAL_STATE = std::mem::MaybeUninit::uninit().assume_init();
      return Ok(());
   }

   /// The only safe part of any of this
   /// global state nonsense.
   fn global_state_guard<'l>(
   ) -> Result<MutexGuard<'l, &'static mut Self>> {
      return Ok(ENVIRONMENT_GLOBAL_STATE_GUARD.lock()?);
   }

   /// Forcibly casts to a const reference
   /// Why yes, I program in C
   fn global_state_ref<'l>(
   ) -> Result<MutexGuard<'l, &'static Self>> {
      let guard = Self::global_state_guard()?;

      // Yikes!
      let guard = unsafe{std::mem::transmute::<
         MutexGuard<'l, &'static mut   Self>,
         MutexGuard<'l, &'static       Self>,
      >(guard)};

      return Ok(guard);
   }

   fn new() -> Result<Self> {
      let console = crate::console::Console::new()?;

      return Ok(Self{
         console  : console,
      });
   }
}

///////////////////////////
// METHODS - Environment //
///////////////////////////

impl Environment {
   /// Gets a handle to the program's
   /// environment.
   ///
   /// <h2 id=  environment_get_panics>
   /// <a href=#environment_get_panics>
   /// Panics
   /// </a></h2>
   ///
   /// If the function is unable to access
   /// the environment, the program will
   /// panic.  For a non-panicking version,
   /// use Environment::try_get().
   pub fn get<'l>(
   ) -> MutexGuard<'l, &'static Self> {
      return Self::try_get().expect(
         "Failed to access environment",
      );
   }

   /// Gets a mutable handle to the
   /// program's environment.
   ///
   /// <h2 id=  environment_get_mut_panics>
   /// <a href=#environment_get_mut_panics>
   /// Panics
   /// </a></h2>
   ///
   /// If the function is unable to access
   /// the environment, the program will
   /// panic.  For a non-panicking version,
   /// use Environment::try_get_mut().
   pub fn get_mut<'l>(
   ) -> MutexGuard<'l, &'static mut Self> {
      return Self::try_get_mut().expect(
         "Failed to access mutable environment",
      );
   }

   /// Tries to get a handle to the
   /// program's environment, returning
   /// an error upon failure.
   pub fn try_get<'l>(
   ) -> Result<MutexGuard<'l, &'static Self>> {
      return Self::global_state_ref();
   }

   /// Tries to get a mutable handle to
   /// the program's environment, returning
   /// an error upon failure.
   pub fn try_get_mut<'l>(
   ) -> Result<MutexGuard<'l, &'static mut Self>> {
      return Self::global_state_guard();
   } 

   /// Gets a reference to the stored
   /// console.
   pub fn console<'l>(
      &'l self,
   ) -> &'l crate::console::Console {
      return &self.console;
   }

   /// Gets a mutable reference to the
   /// stored console.
   pub fn console_mut<'l>(
      &'l mut self,
   ) -> &'l mut crate::console::Console {
      return & mut self.console;
   }
}

//////////////////////////////////
// MAIN EXECUTORS - Environment //
//////////////////////////////////

#[cfg(debug_assertions)]
const DEBUG_SLEEP_ON_ERROR_DURATION
   : std::time::Duration
   = std::time::Duration::from_secs(5);

/// Creates a new environment and
/// initializes the global context
/// with it, returning from the caller
/// with OSReturn::FAILURE upon failure.
/// In debug mode, it will sleep for a
/// brief period of time before exiting.
macro_rules! init_environment {
   () => {
      match Environment::new() {
         Ok(env)  => unsafe{env.global_state_init()},
         Err(e)   => {
            eprintln!("Error: Failed to initialize environment: {e}");

            #[cfg(debug_assertions)]
            std::thread::sleep(DEBUG_SLEEP_ON_ERROR_DURATION);

            return crate::sys::environment::OSReturn::FAILURE;
         },
      }
   };
}

/// Frees the global environment context
/// and drops it, returning from the caller
/// with OSReturn::FAILURE upon failure.
/// In debug mode, it will sleep for a
/// brief period of time before exiting.
macro_rules! free_environment {
   () => {
      match unsafe{Environment::global_state_free()} {
         Ok(_)    => (),
         Err(e)   => {
            eprintln!("Error: Failed to free environment: {e}");

            #[cfg(debug_assertions)]
            std::thread::sleep(DEBUG_SLEEP_ON_ERROR_DURATION);

            return crate::sys::environment::OSReturn::FAILURE;
         },
      }
   };
}

/// Executes a main-like function
/// which has no return type.
macro_rules! execute_main_void {
   ($identifier:ident) => {
      $identifier();
   };
}

/// Executes a main-like function
/// which returns a Result value.
/// If an Err is returned, the
/// global environment context will
/// be freed andthe caller will return
/// OSReturn::FAILURE to the system.
/// In debug mode, it will sleep
/// for a brief period of time before
/// exiting.
macro_rules! execute_main_result {
   ($identifier:ident) => {
      if let Err(err) = $identifier() {
         eprintln!("Error: {err}");
         
         #[cfg(debug_assertions)]
         std::thread::sleep(DEBUG_SLEEP_ON_ERROR_DURATION);

         free_environment!();
         return crate::sys::environment::OSReturn::FAILURE;
      }
   };
}

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
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce(),
   {
      init_environment!();
      execute_main_void!(entrypoint);
      free_environment!();

      return crate::sys::environment::OSReturn::SUCCESS;
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
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce() -> std::result::Result<(), E>,
         E: std::error::Error,
   {
      init_environment!();
      execute_main_result!(entrypoint);
      free_environment!();

      return crate::sys::environment::OSReturn::SUCCESS;
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
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce() -> std::result::Result<(), Box<dyn std::error::Error>>,
   {
      init_environment!();
      execute_main_result!(entrypoint);
      free_environment!();

      return crate::sys::environment::OSReturn::SUCCESS;
   }
}

