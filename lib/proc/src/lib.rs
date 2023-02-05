//! Crate root for nusion-proc-macros, a collection
//! of procedural macros to be incorporated into
//! nusion.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

extern crate proc_macro;

/// Builds a shared library entrypoint
/// using the attached function item.
/// The function signature should be
/// the same as main's signature.
/// This should only ever be used on
/// a single function.
#[proc_macro_attribute]
pub fn entry(
   _     : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // Store the user function for later
   let user_func = item.clone();

   // Format entry point constrution
   let slib_entry : proc_macro::TokenStream = format!(r"

   ").parse().unwrap();

   // Insert entrypoint before the user function and return
   let mut output = proc_macro::TokenStream::new();
   output.extend(slib_entry);
   output.extend(user_func);
   return output;
}

// Unit tests
#[cfg(tests)]
mod tests;

