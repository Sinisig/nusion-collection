//! Environment initialization and main
//! thread entrypoint creation.

use std::sync::{Mutex, MutexGuard};

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Error type relating to some
/// issue with the environment.
#[derive(Debug)]
pub struct EnvironmentError{
   kind  : EnvironmentErrorKind,
}

/// Error kind for EnvironmentError
#[derive(Debug)]
pub enum EnvironmentErrorKind{
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

////////////////////////////////
// METHODS - EnvironmentError //
////////////////////////////////

impl EnvironmentError {
   /// Creates a new EnvironmentError from
   /// a EnvironmentErrorKind.
   pub fn new(
      kind : EnvironmentErrorKind,
   ) -> Self {
      return Self{
         kind : kind,
      };
   }

   /// Gets a reference to the stored
   /// EnvironmentErrorKind.
   pub fn kind<'l>(
      &'l self,
   ) -> &'l EnvironmentErrorKind {
      return &self.kind;
   }
}

//////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - EnvironmentError //
//////////////////////////////////////////////

impl std::fmt::Display for EnvironmentError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", self.kind());
   }
}

impl std::error::Error for EnvironmentError {
}

impl From<crate::console::ConsoleError> for EnvironmentError {
   fn from(
      item : crate::console::ConsoleError,
   ) -> Self {
      return Self::new(EnvironmentErrorKind::ConsoleError{
         err : item,
      });
   }
}

impl<T> From<std::sync::PoisonError<T>> for EnvironmentError {
   fn from(
      _ : std::sync::PoisonError<T>,
   ) -> Self {
      return Self::new(EnvironmentErrorKind::PoisonedContext);
   }
}

//////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - EnvironmentErrorKind //
//////////////////////////////////////////////////

impl std::fmt::Display for EnvironmentErrorKind {
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
      unsafe{Self::new().expect(
         "Failed to initialize environment",
      ).global_state_init()};

      entrypoint();

      unsafe{Self::global_state_free().expect("Failed to free environment")};
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
   where F: FnOnce() -> std::result::Result<(), E>,
         E: std::error::Error,
   {
      unsafe{Self::new().expect(
         "Failed to initialize environment",
      ).global_state_init()};

      if let Err(err) = entrypoint() {
         eprintln!("Error: {err}");
         unsafe{Self::global_state_free().expect("Failed to free environment")};
         return crate::sys::env::OSReturn::FAILURE;
      }

      unsafe{Self::global_state_free().expect("Failed to free environment")};
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
   where F: FnOnce() -> std::result::Result<(), Box<dyn std::error::Error>>,
   {
      unsafe{Self::new().expect(
         "Failed to initialize environment",
      ).global_state_init()};

      if let Err(err) = entrypoint() {
         eprintln!("Error: {err}");
         unsafe{Self::global_state_free().expect("Failed to free environment")};
         return crate::sys::env::OSReturn::FAILURE;
      }

      unsafe{Self::global_state_free().expect("Failed to free environment")};
      return crate::sys::env::OSReturn::SUCCESS;
   } 

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

   /// Tries to get a mutable handle to
   /// the program's environment, returning
   /// an error upon failure.
   pub fn try_get_mut<'l>(
   ) -> Result<MutexGuard<'l, &'static mut Self>> {
      return Self::global_state_guard();
   }

   /// Tries to get a handle to the
   /// program's environment, returning
   /// an error upon failure.
   pub fn try_get<'l>(
   ) -> Result<MutexGuard<'l, &'static Self>> {
      return Self::global_state_ref();
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

