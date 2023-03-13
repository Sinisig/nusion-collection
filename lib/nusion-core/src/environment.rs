//! Access and manage the local process
//! modules and other tid-bits.

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

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
      std::thread::sleep(std::time::Duration::from_secs(15));
   }
}

/////////////////////////////////
// ERROR REPORTING AND LOGGING //
/////////////////////////////////

/// Prints an error report to the
/// console and writes it to disk
fn output_error_report(
   error_report   : & str,
   file_name      : & str,
   file_extension : & str,
) {
   // Get the time since the Unix Epoch Time
   // for creating a time stamp for the error
   // log file.
   let unix_epoch_elapsed = std::time::SystemTime::now()
      .duration_since(std::time::SystemTime::UNIX_EPOCH)
      .unwrap_or(std::time::Duration::from_secs(0))
      .as_secs();

   // Get the current working directory to
   // start enumerating the full file path
   // for the error log.  This is done instead
   // of using a relative path because since
   // we may be panicking from the injected
   // process, it will output the error log
   // to the game's executable folder, not
   // the injected library's folder.  This
   // can lead to lots of confusion.
   let mut file_path = std::env::current_dir().unwrap_or(
      std::path::PathBuf::new(),
   );

   // Append file name, time, and extension
   file_path.push(std::path::Path::new("temp.bin"));
   file_path.set_file_name(std::path::Path::new(&format!(
      "{file_name}-{unix_epoch_elapsed}",
   )));
   file_path.set_extension(std::path::Path::new(file_extension));

   // Display the error message in the console
   eprint!("{error_report}");

   // Display the output path for the error report
   println!(
      "Writing error log to \"{}\"...\n",
      file_path.to_str().unwrap_or("(invalid text)"),
   );

   // Attempt to write the error log
   std::fs::write(&file_path, error_report).unwrap_or_else(|e| {
      eprintln!("Failed to write the error report! {e}");
      eprintln!("Grumble...grumble...");
   });

   return;
}

/// Panic handler hook for printing
/// the call stack and source code
/// unwrap location
fn panic_handler(panic_info : & std::panic::PanicInfo<'_>) {
   // Error log file output name and extension
   const ERROR_REPORT_FILE_NAME  : &'static str
      = "nusion-panic-report";
   const ERROR_REPORT_FILE_EXT   : &'static str
      = "txt";
   
   // Error log formatting buffer
   let mut err_buffer = String::new();

   // Initial panic message
   err_buffer += "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n";
   err_buffer += "!!!       NUSION PANICKED       !!!\n";
   err_buffer += "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\n";

   // Format the location in the source code
   if let Some(location) = panic_info.location() {
      let file = location.file();
      let line = location.line();
      let colm = location.column();

      err_buffer += &format!("Panicked in {file} at {line},{colm}: ");
   } else {
      err_buffer += "(source file information unavaliable): ";
   }

   // Format the attached payload message
   if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
      err_buffer += &format!("{msg}\n\n");
   } else {
      err_buffer += "(unable to format error message)\n\n";
   }

   // Format the call stack from most to least recent function
   err_buffer += "----------- Call stack ------------\n";
   for frame in backtrace::Backtrace::new().frames().iter() {
      // Zero-fill character count for the address
      const ADDR_CHARCOUNT : usize
         = std::mem::size_of::<usize>() * 2 + 2;

      // Formats a memory address into a string
      let format_address = |address| {format!(
         "{addr:#0fill$x}",
         addr = address as usize,
         fill = ADDR_CHARCOUNT,
      )};

      // Buffer for the current stack frame
      let mut frame_buffer = String::new();
            
      // If there are no symbols, append a note
      if frame.symbols().is_empty() == true {
         frame_buffer += "(no symbol information for this frame)\n";
      }

      // Iterate for every symbol in the frame
      for sym in frame.symbols() {
         // Symbol address in memory
         if let Some(addr) = sym.addr() {
            frame_buffer += &format!("{}: ", format_address(addr));
         } else {
            frame_buffer += &format!("{}: ", "?".repeat(ADDR_CHARCOUNT));
         }

         // Symbol's name
         if let Some(name) = sym.name() {
            frame_buffer += &format!("{name} ");
         } else {
            frame_buffer += "(no symbol name)";
         }

         // File containing the symbol
         if let Some(file) = sym.filename() {
            let file = file.to_str().unwrap_or("(bad file path)");
            frame_buffer += &format!("{file} ");
         }

         // Code line containing the symbol
         if let Some(line) = sym.lineno() {
            frame_buffer += &format!("{line},");
         }

         // Code column containing the symbol
         if let Some(colm) = sym.colno() {
            frame_buffer += &format!("{colm}");
         }

         // Enumerate the next symbol
         frame_buffer += "\n";
      }

      // Print the instruction pointer after the call
      frame_buffer += &format!(
         "   Instruction pointer address: {}\n",
         format_address(frame.ip()),
      );

      // Write the frame buffer to the error log
      err_buffer += &frame_buffer;
      err_buffer += "\n";
   }
   err_buffer += "-----------------------------------\n\n";

   // Output the error report
   output_error_report(
      &err_buffer,
      ERROR_REPORT_FILE_NAME,
      ERROR_REPORT_FILE_EXT,
   );

   // Sleep in debug builds to give time to
   // analyze the panic
   debug_sleep!();

   return;
}

/// Reports an error to the console
/// and logs to a file.
pub fn report_error(err : & str) {
   // Error log file output name and extension
   const ERROR_REPORT_FILE_NAME  : &'static str
      = "nusion-error-report";
   const ERROR_REPORT_FILE_EXT   : &'static str
      = "txt";
   
   // Error log formatting buffer
   let mut err_buffer = String::new();

   // Initial error message
   err_buffer += "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n";
   err_buffer += "!!!       NUSION ERRORED       !!!\n";
   err_buffer += "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\n";

   // Format the error string 
   err_buffer += &format!("{err}\n\n");

   // Output the error report
   output_error_report(
      &err_buffer,
      ERROR_REPORT_FILE_NAME,
      ERROR_REPORT_FILE_EXT,
   );

   // Sleep in debug builds to give time to
   // analyze the error
   debug_sleep!();

   return;
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

/// <code>Result</code> type with error
/// variant <code>EnvironmentError</code>
pub type Result<T> = std::result::Result<T, EnvironmentError>;

/// Struct for storing and managing
/// environment information.  In
/// debug builds, a separate console
/// window is created for debugging
/// purposes.  If <code>main</code>
/// fails to start, <code>main </code>
/// returns an error, or at any point
/// the program panics, an error log
/// is written inside the game executable's
/// directory.  In addition, in debug builds
/// the environment will wait for a brief
/// period of time before exiting to
/// give developers a chance to notice
/// the error or panic and see the output
/// file path.
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

////////////////////////////////
// GLOBAL STATE - Environment //
////////////////////////////////

static mut ENVIRONMENT_GLOBAL_STATE
   : Option<Environment>
   = None;

lazy_static::lazy_static!{
static ref ENVIRONMENT_GLOBAL_STATE_LOCK
   : RwLock<&'static mut Environment>
   = RwLock::new(unsafe{ENVIRONMENT_GLOBAL_STATE.as_mut().expect(
      "Accessed environment before initialization, this is a bug!",
   )});
}

impl Environment {
   fn global_state_lock_mut<'l>(
   ) -> Result<RwLockWriteGuard<'l, &'static mut Self>> {
      return Ok(ENVIRONMENT_GLOBAL_STATE_LOCK.write()?);
   }

   fn global_state_lock<'l>(
   ) -> Result<RwLockReadGuard<'l, &'static Self>> {
      let lock = ENVIRONMENT_GLOBAL_STATE_LOCK.read()?;

      // This is fine because we are converting
      // a mutable reference to an immutable
      // reference.  The other way around is
      // never OK.
      let lock = unsafe{std::mem::transmute::<
         RwLockReadGuard<'l, &'static mut Self>,
         RwLockReadGuard<'l, &'static     Self>,
      >(lock)};

      return Ok(lock);
   }

   fn global_state_init(self) {
      // Lack of synchronization is fine
      // since we only call init once at
      // the beginning before other threads
      // can cause monkey business

      if unsafe{ENVIRONMENT_GLOBAL_STATE.is_some()} {
         panic!("Attempted to initialize environment after it was already initialized, this is a bug!");
      }

      unsafe{ENVIRONMENT_GLOBAL_STATE = Some(self)};
      return;
   }

   fn global_state_free() -> Result<Self> {
      // Obtain the lock to ensure thread safety
      let _write_lock = Self::global_state_lock_mut()?;

      let env = unsafe{ENVIRONMENT_GLOBAL_STATE.take()}.expect(
         "Attempted to free environment after it was already freed, this is a bug!",
      );

      return Ok(env);
   }
}

////////////////////////////////////
// INTERNAL METHODS - Environemnt //
////////////////////////////////////

impl Environment {
   /// Creates a new instance of an
   /// environment
   fn new() -> Result<Self> {
      // Register our panic hook before all
      // else so we get proper panic behavior
      // if any of the below panics.
      std::panic::set_hook(Box::new(panic_handler));

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
// PUBLIC METHODS - Environment //
//////////////////////////////////

impl Environment {
   /// Obtains a lock to the environment
   /// mutex.
   ///
   /// <h2 id=  environment_get_panics>
   /// <a href=#environment_get_panics>
   /// Panics
   /// </a></h2>
   ///
   /// If the function is unable to access
   /// the environment, the program will
   /// panic.  For a non-panicking version,
   /// use <code>try_get</code>.
   pub fn get<'l>(
   ) -> RwLockReadGuard<'l, &'static Self> {
      return Self::try_get().expect(
         "Failed to access environment",
      );
   }

   /// Obtains a mutable lock to the
   /// environment mutex.
   ///
   /// <h2 id=  environment_get_mut_panics>
   /// <a href=#environment_get_mut_panics>
   /// Panics
   /// </a></h2>
   ///
   /// If the function is unable to access
   /// the environment, the program will
   /// panic.  For a non-panicking version,
   /// use <code>try_get_mut</code>.
   pub fn get_mut<'l>(
   ) -> RwLockWriteGuard<'l, &'static mut Self> {
      return Self::try_get_mut().expect(
         "Failed to access mutable environment",
      );
   }

   /// Tries to obtain a lock to the
   /// environment mutex.
   pub fn try_get<'l>(
   ) -> Result<RwLockReadGuard<'l, &'static Self>> {
      return Self::global_state_lock();
   }

   /// Tries to obtain a mutable lock
   /// to the environment mutex.
   pub fn try_get_mut<'l>(
   ) -> Result<RwLockWriteGuard<'l, &'static mut Self>> {
      return Self::global_state_lock_mut();
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
   pub fn modules_refresh(
      & mut self,
   ) -> Result<& mut Self> {
      let modules = crate::process::ModuleSnapshotList::all(
         crate::process::ProcessSnapshot::local()?,
      )?;

      self.modules = modules;
      return Ok(self);
   }
}

////////////////////////////////
// MAIN STARTER HELPER MACROS //
////////////////////////////////

/// Creates a new environment and
/// initializes the global context
/// with it, returning from the caller
/// with OSReturn::FAILURE upon failure.
/// In debug mode, it will sleep for a
/// brief period of time before exiting.
macro_rules! environment_init {
   () => {
      match Environment::new() {
         Ok(env)  => env.global_state_init(),
         Err(e)   => {
            report_error(&format!("Failed to initialize environment: {e}"));
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
macro_rules! environment_free {
   () => {
      std::mem::drop(match Environment::global_state_free() {
         Ok(_)    => (),
         Err(e)   => {
            report_error(&format!("Failed to free environment: {e}"));
            return crate::sys::environment::OSReturn::FAILURE;
         },
      })
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
               report_error(&format!("Failed to obtain local process: {e}"));
               environment_free!();
               return crate::sys::environment::OSReturn::FAILURE;
            },
         };
         let proc = &proc.executable_file_name();

         // Find the process name in the list,
         // erroring if not found
         if $whitelist.iter().find(|cur| {
            cur.eq(&proc)
         }).is_none() == true {
            report_error(&format!("Entrypoint does not allow binding to \"{proc}\""));
            environment_free!();
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
         report_error(&format!("Main returned an error: {err}"));
         environment_free!();
         return crate::sys::environment::OSReturn::FAILURE;
      }
   };
}

///////////////////
// MAIN STARTERS //
///////////////////

/// Internal module, do not use this!
pub mod __start_main {
   use super::*;

   pub fn void<F>(
      entrypoint        : F,
      process_whitelist : &[&str],
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce(),
   {
      environment_init! ();
      check_whitelist!  (process_whitelist);
      execute_main_void!(entrypoint);
      environment_free! ();

      return crate::sys::environment::OSReturn::SUCCESS;
   }

   pub fn result_static<F, E>(
      entrypoint        : F,
      process_whitelist : &[&str],
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce() -> std::result::Result<(), E>,
         E: std::error::Error,
   {
      environment_init!    ();
      check_whitelist!     (process_whitelist);
      execute_main_result! (entrypoint);
      environment_free!    ();

      return crate::sys::environment::OSReturn::SUCCESS;
   }

   pub fn result_dynamic<F>(
      entrypoint        : F,
      process_whitelist : &[&str],
   ) -> crate::sys::environment::OSReturn
   where F: FnOnce() -> std::result::Result<(), Box<dyn std::error::Error>>,
   {
      environment_init!    ();
      check_whitelist!     (process_whitelist);
      execute_main_result! (entrypoint);
      environment_free!    ();

      return crate::sys::environment::OSReturn::SUCCESS;
   }
}

