//! Crate root for nusion-proc-macros, a collection
//! of procedural macros to be incorporated into
//! nusion.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

/// Builds a shared library entrypoint
/// using the attached function item.
/// The function signature should be
/// the same as main's signature.
/// This should only ever be used on
/// a single function inside a dynamic
/// library crate.
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn entry(
   _     : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // Store the user function for later
   let user_func = item.clone();

   // Parse item into function signature
   let signature = match syn::parse_macro_input!(item as syn::Item) {
      syn::Item::Fn(func) => func.sig,
      _  => proc_macro_error::abort_call_site!(
         "Entrypoint item is not a function";

         help = "Change item type to a main-like function";
      ),
   };

   // Verify function arguments
   if signature.inputs.empty_or_trailing() == false {
      proc_macro_error::emit_error!(
         proc_macro_error::SpanRange::call_site(), // TODO: Better span
         "Entrypoint function has non-zero argument count";

         help = "Remove all input arguments";
      );
   }

   // TODO: Verify function return type (output) in here
   // instead of evaluating in the sys macro for better
   // error messages.

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

