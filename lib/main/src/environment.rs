//! Environment initialization and main
//! thread entrypoint creation.

use std::sync::{Mutex, MutexGuard};

//////////////////
// DEBUG MACROS //
//////////////////

/// Blocks the thread for a duration
/// of time in debug builds to give
/// the programmer time to react to
/// error messages.  This does nothing
/// in release builds.
macro_rules! debug_sleep {
   () => {
      #[cfg(debug_assertions)]
      std::thread::sleep(std::time::Duration::from_secs(10));
   }
}

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error relating to the environment.
#[derive(Debug)]
pub enum EnvironmentError {
   PoisonedContext,
   ConsoleError{
      err : crate::console::ConsoleError,
   },
   ProcessError{
      err : crate::process::ProcessError,
   },
}

/// Result type with Err variant
/// EnvironmentError.
pub type Result<T> = std::result::Result<T, EnvironmentError>;

/// Struct for keeping track of
/// environment information.
pub struct Environment {
   console  : crate::console::Console,
   process  : crate::process::ProcessSnapshot,
   modules  : crate::process::ModuleSnapshotList,
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
         Self::PoisonedContext
            => write!(stream, "Environment context is poisoned"),
         Self::ConsoleError{err}
            => write!(stream, "Console error: {err}"),
         Self::ProcessError{err}
            => write!(stream, "Process error: {err}"),
      };
   }
}

impl std::error::Error for EnvironmentError {
}

impl<T> From<std::sync::PoisonError<T>> for EnvironmentError {
   fn from(
      _ : std::sync::PoisonError<T>,
   ) -> Self {
      return Self::PoisonedContext;
   }
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

impl From<crate::process::ProcessError> for EnvironmentError {
   fn from(
      item : crate::process::ProcessError,
   ) -> Self {
      return Self::ProcessError{
         err : item,
      };
   }
}

////////////////////////////////////
// INTERNAL METHODS - Environment //
////////////////////////////////////

static mut ENVIRONMENT_GLOBAL_STATE
   : Option<Environment>
   = None;

lazy_static::lazy_static!{
static ref ENVIRONMENT_GLOBAL_STATE_GUARD
   : Mutex<&'static mut Environment>
   = Mutex::new(unsafe{ENVIRONMENT_GLOBAL_STATE.as_mut().expect(
      "Accessed environment before initialization, this is a bug",
   )});
}

impl Environment {
   // Make sure to initialize before accessing
   // the guard, otherwise the program will
   // panic.
   unsafe fn global_state_init(self) {
      ENVIRONMENT_GLOBAL_STATE = Some(self);
      return;
   }

   // Don't use the guard after freeing, as this
   // will leave the mutex guard with a dangling
   // reference.
   unsafe fn global_state_free() -> Result<()> {
      // Done like this to block until every thread
      // is done accessing the environment.
      let _guard = ENVIRONMENT_GLOBAL_STATE_GUARD.lock()?;
      ENVIRONMENT_GLOBAL_STATE = None;
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

   /// Creates a new instance of an
   /// environment
   fn new() -> Result<Self> {
      // Register our panic hook before all
      // else so we get proper panic behavior
      // if any of the below panics.
      std::panic::set_hook(Box::new(|panic_info| {
         const ERROR_LOG_FILE_NAME  : &'static str
            = "nusion-panic-log";
         const ERROR_LOG_FILE_EXT   : &'static str
            = "txt";

         let mut err_buffer = String::new();

         err_buffer += "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n";
         err_buffer += "!!!       NUSION PANICKED       !!!\n";
         err_buffer += "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\n";

         // Format the location in the source code
         if let Some(location) = panic_info.location() {
            let file = location.file();
            let line = location.line();
            let colm = location.column();

            err_buffer += &format!(
               "Panicked in {file} at {line},{colm}: "
            );
         } else {
            err_buffer += "(source file information unavaliable): ";
         }

         // Format the attached payload message
         if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
            err_buffer += &format!("{msg}\n\n");
         } else {
            err_buffer += "(unable to format error message)\n\n";
         }

         // Format down the entire known call stack
         err_buffer += "----------- Call stack ------------\n";
         err_buffer += "TODO: Implement this\n";
         err_buffer += "-----------------------------------\n\n";

         // Get the current working directory to
         // start enumerating the full file path
         // for the error log.  This is done instead
         // of using a relative path because since
         // we may be panicking from the injected
         // process, it will output the error log
         // to the game's executable folder, not
         // the injected library's folder.  This
         // can lead to lots of confusion.
         let mut err_log_path = std::env::current_dir().unwrap_or(
            std::path::PathBuf::new(),
         );

         // Append file name, time, and extension
         err_log_path.push(std::path::Path::new(
            ERROR_LOG_FILE_NAME,
         ));
         err_log_path.push(std::path::Path::new(&format!(
            "",  
         )));
         err_log_path.push(std::path::Path::new(&format!(
            ".{ERROR_LOG_FILE_EXT}",
         )));

         // Write the output error log path, but don't
         // actually write the file yet
         err_buffer += &format!(
            "Writing error log to \"{}\"...\n",
            err_log_path.to_str().unwrap_or("(invalid text)"),
         );

         // Display the error message
         eprint!("{err_buffer}");

         // Attempt to write the error log
         std::fs::write(&err_log_path, &err_buffer).unwrap_or_else(|e| {
            eprintln!("Failed to write the error log! {e}");
            eprintln!("Grumble...grumble...");
         });

         // Sleep in debug builds to give time to
         // analyze the panic
         debug_sleep!();
      }));

      let console = crate::console::Console::new()?;

      let process = crate::process::ProcessSnapshot::local()?;

      let modules = crate::process::ModuleSnapshotList::all(
         crate::process::ProcessSnapshot::local()?,
      )?;

      return Ok(Self{
         console  : console,
         process  : process,
         modules  : modules,
      });
   }
}

/////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - Environment //
/////////////////////////////////////////

impl std::ops::Drop for Environment {
   fn drop(
      & mut self,
   ) {
      let _ = std::panic::take_hook();
      return;
   }
}

//////////////////////////////////
// MAIN EXECUTORS - Environment //
//////////////////////////////////

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
            eprintln!   ("Error: Failed to initialize environment: {e}");
            debug_sleep!();
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
            eprintln!   ("Error: Failed to free environment: {e}");
            debug_sleep!();
            return crate::sys::environment::OSReturn::FAILURE;
         },
      }
   };
}

/// Checks the given process whitelist
/// and makes sure the process name is
/// contained within the whitelist assuming
/// a non-empty whitelist.
macro_rules! check_whitelist {
   ($whitelist:ident) => {
      // Make sure there's items
      if $whitelist.is_empty() == false {
         // Get the process name
         let proc = match crate::process::ProcessSnapshot::local() {
            Ok(proc) => proc,
            Err(e)   => {
               eprintln!         ("Error: Failed to obtain local process: {e}");
               debug_sleep!      ();
               free_environment! ();
               return crate::sys::environment::OSReturn::FAILURE;
            },
         };
         let proc = &proc.executable_file_name();

         // Find the process name in the list,
         // erroring if not found
         if $whitelist.iter().find(|cur| {
            cur.eq(&proc)
         }).is_none() == true {
            eprintln!         ("Error: Entrypoint does not allow binding to \"{proc}\"");
            debug_sleep!      ();
            free_environment! ();
            return crate::sys::environment::OSReturn::FAILURE;
         }
      }
   }
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
         eprintln!         ("Error: {err}");
         debug_sleep!      ();
         free_environment! ();
         return crate::sys::environment::OSReturn::FAILURE;
      }
   };
}

impl Environment {
   /// Initializes the thread environment
   /// and executes an entrypoint with no
   /// return type.  If the process name
   /// does not match any of those in
   /// process whitelist, an error is returned.
   /// If the process whitelist is empty,
   /// this check is ignored.
   ///
   /// <h2   id=note_environment_start_main_result_static>
   /// <a href=#note_environment_start_main_result_static>
   /// Note
   /// </a></h2>
   /// This function should never be called directly.
   /// Instead use the nusion::entry attribute macro
   /// to register a function as the designated entrypoint.
   pub fn __start_main_void<F>(
      entrypoint        : F,
      process_whitelist : &[&str],
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce(),
   {
      init_environment! ();
      check_whitelist!  (process_whitelist);
      execute_main_void!(entrypoint);
      free_environment! ();

      return crate::sys::environment::OSReturn::SUCCESS;
   }

   /// Initializes the thread environment
   /// and executes an entrypoint with a
   /// Result&lt;(), E&gt; return type where E
   /// implements std::error::Error statically.
   /// If the process name does not match any
   /// of those in process whitelist, an error
   /// is returned. If the process whitelist is
   /// empty, this check is ignored.
   ///
   /// <h2   id=note_environment_start_main_result_static>
   /// <a href=#note_environment_start_main_result_static>
   /// Note
   /// </a></h2>
   /// This function should never be called directly.
   /// Instead use the nusion::entry attribute macro
   /// to register a function as the designated entrypoint.
   pub fn __start_main_result_static<F, E>(
      entrypoint        : F,
      process_whitelist : &[&str],
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce() -> std::result::Result<(), E>,
         E: std::error::Error,
   {
      init_environment!    ();
      check_whitelist!     (process_whitelist);
      execute_main_result! (entrypoint);
      free_environment!    ();

      return crate::sys::environment::OSReturn::SUCCESS;
   }

   /// Initializes the thread environment
   /// and executes an entrypoint with a
   /// Result&lt;(), Box&lt;dyn std::error::Error&gt;&gt;
   /// return type. If the process name
   /// does not match any of those in
   /// process whitelist, an error is
   /// returned. If the process whitelist
   /// is empty, this check is ignored.
   ///
   /// <h2   id=note_environment_start_main_result_static>
   /// <a href=#note_environment_start_main_result_static>
   /// Note
   /// </a></h2>
   /// This function should never be called directly.
   /// Instead use the nusion::entry attribute macro
   /// to register a function as the designated entrypoint.
   pub fn __start_main_result_dynamic<F>(
      entrypoint        : F,
      process_whitelist : &[&str],
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce() -> std::result::Result<(), Box<dyn std::error::Error>>,
   {
      init_environment!    ();
      check_whitelist!     (process_whitelist);
      execute_main_result! (entrypoint);
      free_environment!    ();

      return crate::sys::environment::OSReturn::SUCCESS;
   }
}

//////////////////////////////////
// PUBLIC METHODS - Environment //
//////////////////////////////////

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

   /// Gets a reference to the current
   /// process information.
   pub fn process<'l>(
      &'l self,
   ) -> &'l crate::process::ProcessSnapshot {
      return &self.process;
   }

   /// Gets a reference to the stored
   /// module list for the process.
   pub fn modules<'l>(
      &'l self,
   ) -> &'l crate::process::ModuleSnapshotList {
      return &self.modules;
   }

   /// Gets a mutable reference to the
   /// stored module list for the process.
   pub fn modules_mut<'l>(
      &'l mut self,
   ) -> &'l mut crate::process::ModuleSnapshotList {
      return & mut self.modules;
   }

   /// Refreshes the module list for
   /// the current process in case any
   /// other modules were loaded or
   /// unloaded.  For most use cases,
   /// this function should not be needed
   /// as processes rarely dynamically load
   /// or unload modules after initialization.
   pub fn refresh_modules(
      & mut self,
   ) -> Result<& mut Self> {
      let modules = crate::process::ModuleSnapshotList::all(
         crate::process::ProcessSnapshot::local()?,
      )?;

      self.modules = modules;
      return Ok(self);
   }
}

