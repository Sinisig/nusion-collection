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
      proc_macro_error::emit_call_site_error!(
         "Entrypoint function has non-zero argument count";

         help = "Remove all input arguments";
      );
   } 

   // Store function identifier
   let identifier = signature.ident;

   // Choose initialization type based on return type
   let init_type = if let syn::ReturnType::Type(_, ty) = signature.output {
      // TODO: Verify return type
      "result_dy"
   } else {
      "default"
   };

   // Format entry point constrution
   let slib_entry : proc_macro::TokenStream = format!(r"
      nusion::sys::build_slib_entry!(
         {identifier},
         {init_type}
      );
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

