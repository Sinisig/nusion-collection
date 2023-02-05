//! Platform-specific macro implementations.

/// Macro for creating a shared library
/// entrypoint and attaching a function
/// to act as the "main" for the shared
/// library.  The first argument is an
/// identifier for the function, and the
/// second argument is the crate path to
/// the chosen initialization function.
#[macro_export]
macro_rules! build_slib_entry {
   ($entry:ident, default)    => {
      nusion::sys::build_slib_entry_os!(
         $entry,
         run_main_default
      );
   };
   ($entry:ident, result_st)  => {
      nusion::sys::build_slib_entry_os!(
         $entry,
         run_main_result_static
      );
   };
   ($entry:ident, result_dy)  => {
      nusion::sys::build_slib_entry_os!(
         $entry,
         run_main_result_dynamic
      );
   };
}

