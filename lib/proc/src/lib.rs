//! Crate root for nusion-proc-macros, a collection
//! of procedural macros to be incorporated into
//! nusion.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

extern crate proc_macro;

/// Attribute which constructs
/// a dynamic library entrypoint.
/// This is syntactic sugar for
/// nusion::framework::entry!(<name>, <return type>).
#[proc_macro_attribute]
pub fn entry(
   _     : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   todo!();

   // Get the function identifier and signature
   let identifier    = "DUMMY";
   let return_type   = "DUMMY";

   // Insert the framework macro call
   // and return the formatted code
   return format!(r"
      nusion::build_slib_entrypoint!({identifier}, {return_type});

      {item}
   ").parse().unwrap();
}

// Unit tests
#[cfg(tests)]
mod tests;

