//! Crate root for nusion-proc-macros, a collection
//! of procedural macros to be incorporated into
//! nusion.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

////////////////////////////////////////////
// ATTRIBUTE MACRO IMPLEMENTATION - entry //
////////////////////////////////////////////

struct EntrypointInfo {
   identifier  : syn::Ident,
   return_type : EntrypointReturnType,
}

enum EntrypointReturnType {
   Void,    // -> ()
   Static,  // -> Result<(), E: std::error::Error>
   Dynamic, // -> Result<(), Box<dyn std::error::Error>>
}

// Internal helper for parsing a function
// signature into its return type.  The
// return type is an error string to be
// parsed into a compile error.
fn entry_parse_signature(
   signature   : syn::Signature,
) -> Result<EntrypointInfo, String> {
   // Verify argument list is empty
   if signature.inputs.is_empty() == false {
      return Err(format!(
         "Entrypoint has non-zero argument count",
      ));
   }

   // Store function identifier and return type
   let identifier    = signature.ident;
   let return_type   = signature.output;

   // Parse the return type if it's () or Result<(), E>
   let return_type = match return_type {
      syn::ReturnType::Default
         => return Ok(EntrypointInfo{
            identifier  : identifier,
            return_type : EntrypointReturnType::Void,
         }),
      syn::ReturnType::Type(_, ty)
         => *ty,
   };

   // Make sure it's an item path
   let return_type = match return_type {
      syn::Type::Path(path) => path.path,
      _ => return Err(format!(
         "Entrypoint doesn't return a Result type",
      )),
   };

   // Get the first and only path segment
   let return_type = return_type.segments.first().unwrap();

   // Verify it's a Result or std::result::Result
   if return_type.ident != "Result" &&
      return_type.ident != "std::result::Result"
   {
      return Err(format!(
         "Entrypoint doesn't return a Result type",
      ));
   }

   // Get generic argument list
   let return_type = match &return_type.arguments {
      syn::PathArguments::AngleBracketed(args) => args.args.clone(),
      _ => return Err(format!(
         "Entrypoint returns invalid Result type",
      )),
   };

   // Make sure there are exactly two type generic paramenters
   if return_type.len() != 2 {
      return Err(format!(
         "Entrypoint returns invalid Result type",
      ));
   }

   // Break up into Ok type and Err type
   let return_ok  = match return_type.iter().nth(0).unwrap() {
      syn::GenericArgument::Type(ty) => ty,
      _ => return Err(format!(
         "Entrypoint returns invalid Result type",
      )),
   };
   let return_err = match return_type.iter().nth(1).unwrap() {
      syn::GenericArgument::Type(ty) => ty,
      _ => return Err(format!(
         "Entrypoint returns invalid Result type",
      )),
   };

   // Verify the Ok variant is the unit type
   let return_ok = match return_ok {
      syn::Type::Tuple(ty) => ty,
      _ => return Err(format!(
         "Entrypoint Result Ok variant is not the unit type '()'",
      )),
   };
   if return_ok.elems.is_empty() == false {
      return Err(format!(
         "Entrypoint Result Ok variant is not the unit type '()'",
      ));
   }

   // Get path item for Err variant
   let return_err = match return_err {
      syn::Type::Path(path)   => path,
      _ => return Err(format!(
         "Entrypoint Result Err variant is not an item",
      )),
   };

   // Isolate the path segment for the item
   let return_err = return_err.path.segments.first().unwrap();

   // Check if the Err type is a static type or trait object
   if return_err.ident != "Box" &&
      return_err.ident != "std::boxed::Box"
   {
      return Ok(EntrypointInfo{
         identifier  : identifier,
         return_type : EntrypointReturnType::Static,
      });
   }

   // We now know that we have a Box<> type, let
   // the compiler deal with any more mess
   return Ok(EntrypointInfo{
      identifier  : identifier,
      return_type : EntrypointReturnType::Dynamic,
   });
}

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
   let signature = syn::parse_macro_input!(item as syn::ItemFn).sig;

   // Parse function signature
   let signature = match entry_parse_signature(signature) {
      Ok(sig)  => sig,
      Err(err) => {
         proc_macro_error::emit_call_site_error!(err);
         return user_func;
      },
   };
   
   // Extract function identifier and return type as a macro arg
   let identifier    = signature.identifier;
   let return_type   = match signature.return_type {
      EntrypointReturnType::Void    => "void",
      EntrypointReturnType::Static  => "result_static",
      EntrypointReturnType::Dynamic => "result_dynamic",
   };

   // Format entry point constrution
   let slib_entry : proc_macro::TokenStream = format!(r"
      nusion::__build_entry!({identifier}, {return_type});
   ").parse().unwrap();

   // Prepend entrypoint to user entrypoint function and return
   let mut output = proc_macro::TokenStream::new();
   output.extend(slib_entry);
   output.extend(user_func);
   return output;
}

///////////////////////////////////////
// ATTRIBUTE MACRO DEFINITION - hook //
///////////////////////////////////////

struct HookAttributes {
   asm_template   : syn::LitStr,
   fn_ptr_ident   : syn::Ident,
}

impl syn::parse::Parse for HookAttributes {
   fn parse(
      input : syn::parse::ParseStream<'_>,
   ) -> syn::parse::Result<Self> {
      let asm_template  = input.parse()?;       // ASM Template
      input.parse::<syn::Token![,]>()?;         // Required comma separator
      let fn_ptr_ident  = input.parse()?;       // ASM Fn Ptr Identifier
      input.parse::<Option<syn::Token![,]>>()?; // Optional trailing comma

      return Ok(Self{
         asm_template   : asm_template,
         fn_ptr_ident   : fn_ptr_ident,
      });
   }
}

/// Constructs an associated assembly
/// intermediate subroutine to call
/// the attached function from a hook
/// patch.
///
/// The first attribute contains the
/// identifier to be used for a constant
/// containing a void pointer to the
/// assembly subroutine.  This constant
/// is what should be passed as the
/// hook target to hook patch functions.
///
/// The second argument is an assembly
/// template used to create the assembly
/// intermediary which is called by the
/// hook patch.  This assembly template
/// largely follows the same usage as the
/// <code><a href=
/// https://doc.rust-lang.org/stable/core/arch/macro.global_asm.html
/// >global_asm!()</a></code> macro, but
/// with a custom template argument format
/// which intentionally doesn't allow
/// custom options nor operand inputs.
///
/// <h2 id=  template_argument_format>
/// <a href=#template_argument_format>
/// Template Argument Format
/// </a></h2>
/// A template argument is represented
/// by some argument surrounded by
/// curly brackets.  This is simmilar
/// to the way template arguments are
/// used in the <code>format!()</code>
/// macro.
///
/// <h2 id=  template_argument_types>
/// <a href=#template_argument_types>
/// Template Argument Types
/// </a></h2>
/// <ul>
/// <li>
/// <code>hook</code>: Substitutes the
/// assembly-compatiable identifier for
/// the target high-level rust hook
/// function.  Use this to call the
/// high-level rust hook function from
/// your assembly intermediary.
/// </li>
/// </ul>
///
/// <h2 id=  safety>
/// <a href=#safety>
/// Safety
/// </a></h2>
/// It is assumed the provided assembly
/// hook has no undefined behavior, calls
/// the high-level hook with valid parameters,
/// and aligns and restores the stack.
/// Not obeying these rules will lead
/// to catastrophic results that will be
/// near impossible to debug.
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn hook(
   attr  : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // Parse attributes as arguments
   let args = syn::parse_macro_input!(attr as HookAttributes);

   todo!()
}

