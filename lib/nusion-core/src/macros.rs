//! Various convenience macros.

/// Internal macro, do not use this!
#[macro_export]
macro_rules! __build_entry {
   ($entry:ident, void,             $($proc:literal),*)   => {
      $crate::__private::sys_build_entry!(
         $crate::__private::start_main::void,
         $entry,
         $crate::__private::osapi,
         $($proc),*
      );
   };
   ($entry:ident, result_static,    $($proc:literal),*)   => {
      $crate::__private::sys_build_entry!(
         $crate::__private::start_main::result_static,
         $entry,
         $crate::__private::osapi,
         $($proc),*
      );
   };
   ($entry:ident, result_dynamic,   $($proc:literal),*)   => {
      $crate::__private::sys_build_entry!(
         $crate::__private::start_main::result_dynamic,
         $entry,
         $crate::__private::osapi,
         $($proc),*
      );
   };
}

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

