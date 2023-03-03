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

#[derive(Clone, Debug)]
enum HookTemplateArgument {
   TargetHookLabel,
}

impl std::str::FromStr for HookTemplateArgument {
   type Err = ();

   fn from_str<'l>(
      s : &'l str,
   ) -> Result<Self, Self::Err> {
      use std::collections::hash_map::HashMap;

      lazy_static::lazy_static! {
         static ref ARGUMENT_MAP : HashMap<&'static str, HookTemplateArgument> = [
            // Add new arguments here!
            ("hook", HookTemplateArgument::TargetHookLabel)
         ].iter().cloned().collect();
      }

      return ARGUMENT_MAP.get(s).ok_or(()).map(|a| a.clone());
   }
}

#[derive(Debug)]
struct HookTemplateReplacer<'s> {
   label_target   : &'s str,
}

impl<'s> regex::Replacer for HookTemplateReplacer<'s> {
   fn replace_append(
      & mut self,
      caps  : & regex::Captures<'_>,
      dst   : & mut String,
   ) {
      for cap in caps.iter() {
         // Get the capture as a string
         let cap = match cap {
            Some(cap)   => cap.as_str(),
            None        => break,
         };
         
         // Strip out the surrounding curly brackets
         let cap = &cap[1..cap.len()-1];

         // Try to parse as an argument
         let arg = match cap.parse::<HookTemplateArgument>() {
            Ok(a)    => a,
            Err(_)   => panic!("invalid template argument \"{cap}\""),
         };

         // Write the new substituted text
         dst.push_str(match arg {
            HookTemplateArgument::TargetHookLabel
               => self.label_target,
         });
      }

      return;
   }
}

/// Internal helper, formats an input
/// ASM template to one ready to be
/// passed to global_asm!()
fn hook_format_asm_template(
   template       : & str,
   label_assembly : & str,
   label_target   : & str,
) -> String {
   // Regex for finding and replacing
   // template arguments
   lazy_static::lazy_static! {
      static ref TEMPLATE_SEARCHER : regex::Regex = regex::Regex::new(r"(?x)
         \{[A-Za-z0-9]*?\}
      ").unwrap();
   }

   // Search for and iterate over every
   // template argument
   let template = TEMPLATE_SEARCHER.replace_all(
      template,
      HookTemplateReplacer{
         label_target   : label_target,
      },
   );

   // Convert to an owned string
   let template = template.into_owned();

   // Append the prefix stuff for
   // the function definition
   let template = format!("
      {}:
      {}
   ", label_assembly, template);

   // Return the formatted ASM template
   return template;
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

   // Make a copy of the hook function
   let hook_func = item.clone();

   // Get the function signature to copy
   // for the intermediary
   let hook_func_signature = syn::parse_macro_input!(item as syn::ItemFn);
   let hook_func_signature = hook_func_signature.sig;

   // Store the identifier and create
   // a list of the arguments without
   // the type information
   let hook_func_ident        = &hook_func_signature.ident;
   let mut hook_func_inputs   = syn::punctuated::Punctuated::<
      syn::Ident, syn::token::Comma,
   >::new();
   for p in &hook_func_signature.inputs {
      // Isolate the identifier for the arg
      let p = match p {
         syn::FnArg::Typed(p) => p,
         _                    => proc_macro_error::abort_call_site!(""),
      };
      let p = match &*p.pat {
         syn::Pat::Ident(p)   => p,
         _                    => proc_macro_error::abort_call_site!(""),
      };
      let p = &p.ident;

      // Add the identifier to the list
      hook_func_inputs.push_value(p.clone());
   }

   // Create a checksum of the entire
   // hook function for appending to
   // labels.  This will help prevent
   // clashes with simmilarly named
   // items.
   let hook_func_chksm = crc::Crc::<u64>::new(
      &crc::CRC_64_WE,
   ).checksum(&hook_func.to_string().bytes().collect::<Vec<u8>>());

   // Generate identifiers for ASM hook
   // and extern "C" intermediate
   let label_hll = quote::format_ident!(
      "__nusion_hook_hll_{}_{}",
      format!("{:#016X}", hook_func_chksm)[2..],
      hook_func_ident,
   );
   let label_asm = quote::format_ident!(
      "__nusion_hook_asm_{}_{}",
      format!("{:#016X}", hook_func_chksm)[2..],
      hook_func_ident,
   );

   // Get the identifier for the constant
   // function pointer for the ASM hook
   let ident_asm_ptr = args.fn_ptr_ident;

   // Convert the stored ASM template to
   // a sanitized string which doesn't
   // include the quotes from the string
   // literal
   let asm_template = args.asm_template.token().to_string();
   let asm_template = &asm_template[1..asm_template.len() - 1];

   // Format the ASM template to the
   // final one passed to global_asm()!
   let asm_template = hook_format_asm_template(
      asm_template,
      &label_asm.to_string(),
      &label_hll.to_string(),
   );

   // Create the function signature for
   // the intermediate
   let mut inter_sig = hook_func_signature.clone();
   inter_sig.ident   = syn::Ident::new(
      &label_hll.to_string(),
      proc_macro2::Span::call_site(),
   );

   // Put everything together and generate
   // the hook syntax
   let hook = proc_macro::TokenStream::from(quote::quote!{
      // Assemble the intermediary hook
      core::arch::global_asm!(#asm_template);

      // Declaration for our ASM hook
      extern "C" {
         pub fn #label_asm();
      }

      // Constant pointer to the ASM hook
      const #ident_asm_ptr
         : * const core::ffi::c_void
         = #label_asm as * const core::ffi::c_void;

      // The intermediate function for calling
      // the Rust code
      #[no_mangle]
      pub extern "C" #inter_sig {
         return #hook_func_ident(#hook_func_inputs);
      }
   });
   
   // Combine the hook function with the
   // generated syntax and return the new
   // syntax
   let mut output = proc_macro::TokenStream::new();
   output.extend(hook);
   output.extend(hook_func);
   return output;
}

