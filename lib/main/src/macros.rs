//! Various convenience macro definitions.

/// Shorthand for nusion::environment::Environment::get().
#[macro_export]
macro_rules! env {
   () => {
      $crate::environment::Environment::get()
   };
}

/// Shorthand for nusion::environment::Environment::get_mut().
#[macro_export]
macro_rules! env_mut {
   () => {
      $crate::environment::Environment::get_mut()
   };
}

/// Shorthand for nusion::environment::Environment::try_get().
#[macro_export]
macro_rules! try_env {
   () => {
      $crate::environment::Environment::try_get()
   };
}

/// Shorthand for nusion::environment::Environment::try_get_mut().
#[macro_export]
macro_rules! try_env_mut {
   () => {
      $crate::environment::Environment::try_get_mut()
   };
}

/// Internal macro, do not use this!
/// Use the entry attribute macro instead!
#[macro_export]
macro_rules! __build_entry {
   ($entry:ident, void)             => {
      $crate::macros::__sys_build_entry!(
         $crate::environment::Environment::__start_main_void,
         $entry,
         $crate::__osapi
      );
   };
   ($entry:ident, result_static)    => {
      $crate::macros::__sys_build_entry!(
         $crate::environment::Environment::__start_main_result_static,
         $entry,
         $crate::__osapi
      );
   };
   ($entry:ident, result_dynamic)   => {
      $crate::macros::__sys_build_entry!(
         $crate::environment::Environment::__start_main_result_dynamic,
         $entry,
         $crate::__osapi
      );
   };
}

/// Internal macro, do not use this!
/// Use the entry attribute macro instead!
pub use crate::sys::build_entry as __sys_build_entry;

