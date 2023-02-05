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
      let ty = if let syn::Type::Path(p) = *ty {
         p.path.segments
      } else {proc_macro_error::abort_call_site!(
         "Return type is not a Result",
      )};

      // Check to make sure it's Result
      let ty = ty.first().unwrap();
      if ty.ident != "Result" {proc_macro_error::abort_call_site!(
         "Return type is not a Result",
      )};

      // Get generic arguments
      let ty = match &ty.arguments {
         syn::PathArguments::AngleBracketed(a)  => a.clone(),
         _ => return user_func,
      };
      let ok   = match ty.args.iter().nth(0).unwrap() {
         syn::GenericArgument::Type(t) => t,
         _ => return user_func,
      };
      let err  = match ty.args.iter().nth(1).unwrap() {
         syn::GenericArgument::Type(t) => t,
         _ => return user_func,
      };

      // Validate ok variant
      let ok = if let syn::Type::Tuple(t) = ok {
         t
      } else {proc_macro_error::abort_call_site!(
         "Ok variant is not the unit type",
      )};
      if ok.elems.empty_or_trailing() == false {proc_macro_error::abort_call_site!(
         "Ok variant is not the unit type",
      )};

      // Validate err variant
      let err = if let syn::Type::Path(p) = err {
         p
      } else {proc_macro_error::abort_call_site!(
         "Err variant is not a static nor dynamic trait object",
      )};
      let err = err.path.segments.first().unwrap();

      // Detect if static or dynamic
      if err.ident == "Box" {
         "result_dy"
      } else {
         "result_st"
      }
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

