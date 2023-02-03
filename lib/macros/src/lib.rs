//! Crate root for nusion-macros, a collection
//! of macros which are incorporated into nusion.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

extern crate proc_macro;

/// Attribute macro which defines
/// an entry point in a dynamic
/// library.  This entrypoint will
/// be executed by the system's
/// dynamic library loader when
/// the library is loaded into the
/// process.  This macro should only
/// be attached to functions with
/// signatures like main, which is
/// to say any function with no
/// arguments and either a void
/// return type or a Result type
/// with an Ok variant of the unit
/// type and Err variant of either
/// a trait which implements
/// std::error::Error or
/// Box<dyn std::error::Error>.
#[proc_macro_attribute]
pub fn entry(
   _     : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   todo!();

   // Get the identifier for the function
   let identifier = "DUMMY";

   // Insert the framework macro call
   // and return the formatted code
   return format!(r"
      nusion::framework::entry!({identifier})

      {item}
   ").parse().unwrap();
}

// Unit tests
#[cfg(tests)]
mod tests;

