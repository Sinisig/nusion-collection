//! Various convenience macros.

/// Internal macro, do not use this!
/// Use the <code>main</code> attribute macro instead!
#[macro_export]
macro_rules! __build_entry {
   ($entry:ident, void,             $($proc:literal),*)   => {
      $crate::macros::__sys_build_entry!(
         $crate::environment::Environment::__start_main_void,
         $entry,
         $crate::__osapi,
         $($proc),*
      );
   };
   ($entry:ident, result_static,    $($proc:literal),*)   => {
      $crate::macros::__sys_build_entry!(
         $crate::environment::Environment::__start_main_result_static,
         $entry,
         $crate::__osapi,
         $($proc),*
      );
   };
   ($entry:ident, result_dynamic,   $($proc:literal),*)   => {
      $crate::macros::__sys_build_entry!(
         $crate::environment::Environment::__start_main_result_dynamic,
         $entry,
         $crate::__osapi,
         $($proc),*
      );
   };
}

/// Internal macro, do not use this!
/// Use the <code>main</code> attribute macro instead!
pub use crate::sys::build_entry as __sys_build_entry;


/// Shorthand for <code>environment::Environment::get</code>.
#[macro_export]
macro_rules! env {
   () => {
      $crate::environment::Environment::get()
   };
}

/// Shorthand for <code>environment::Environment::get_mut</code>.
#[macro_export]
macro_rules! env_mut {
   () => {
      $crate::environment::Environment::get_mut()
   };
}

/// Shorthand for <code>environment::Environment::try_get</code>.
#[macro_export]
macro_rules! try_env {
   () => {
      $crate::environment::Environment::try_get()
   };
}

/// Shorthand for <code>environment::Environment::try_get_mut</code>.
#[macro_export]
macro_rules! try_env_mut {
   () => {
      $crate::environment::Environment::try_get_mut()
   };
}

