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
   ($entry:ident, void)          => {
      nusion::sys::os_build_slib_entry!(
         $entry,
         start_main_void
      );
   };
   ($entry:ident, result_static) => {
      nusion::sys::os_build_slib_entry!(
         $entry,
         start_main_result_static
      );
   };
   ($entry:ident, result_dynamic) => {
      nusion::sys::os_build_slib_entry!(
         $entry,
         start_main_result_dynamic
      );
   };
}

