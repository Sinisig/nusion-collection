//! Crate root for nusion-proc-macros, a collection
//! of procedural macros to be incorporated into
//! nusion.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

///////////////////////////////////////////
// ATTRIBUTE MACRO IMPLEMENTATION - main //
///////////////////////////////////////////

struct EntrypointInfo {
   func     : syn::ItemFn,
   variant  : EntrypointReturnType,
}

enum EntrypointReturnType {
   Void,    // (no return type)
   Static,  // -> Result<(), E: std::error::Error>
   Dynamic, // -> Result<(), Box<dyn std::error::Error>>
}

impl syn::parse::Parse for EntrypointInfo {
   fn parse(
      input : syn::parse::ParseStream<'_>,
   ) -> syn::parse::Result<Self> {
      const OUTPUT_ERROR_MSG : &'static str
         = "main return type should be nothing, Result<(), E: Error>, or Result<(), Box<dyn std::error::Error>>";

      // First parse the entire function
      let func = input.parse::<syn::ItemFn>()?;

      // Check that the visibility is private
      match &func.vis {
         syn::Visibility::Public    (tok) => {
            let span = tok.pub_token.span.unwrap();
            proc_macro_error::emit_error!(
               span, "visibility should be private",
            );
         },
         syn::Visibility::Crate     (tok) => {
            let span = tok.crate_token.span.unwrap();
            proc_macro_error::emit_error!(
               span, "visibility should be private",
            );
         },
         syn::Visibility::Restricted(tok) => {
            let span = tok.paren_token.span.unwrap();
            proc_macro_error::emit_error!(
               span, "visibility should be private",
            );
         },
         syn::Visibility::Inherited       => (),
      }

      // Check that the identifier is named 'main'
      if func.sig.ident != quote::format_ident!("main") {
         let span = func.sig.ident.span();
         proc_macro_error::emit_error!(
            span, "identifier should be 'main'",
         );
      }

      // Make sure there are no input arguments
      if func.sig.inputs.is_empty() == false {
         let span = func.sig.paren_token.span;
         proc_macro_error::emit_error!(
            span, "main should take 0 arguments",
         );
      }

      // If there is no return type, construct
      // a void return type main function.
      // Otherwise unwrap the stored type
      let (arrow_token, output) = match &func.sig.output {
         syn::ReturnType::Default => {
            return Ok(Self{
               func     : func,
               variant  : EntrypointReturnType::Void,
            });
         },
         syn::ReturnType::Type(ar, ty) => (ar, ty),
      };

      // Make sure the type is a type path
      let output = match &**output {
         syn::Type::Path(path) => &path.path,
         
         syn::Type::Array        (ar) => {
            let span = ar.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::BareFn       (bf) => {
            let span = bf.fn_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         }
         syn::Type::Group        (gp) => {
            let span = gp.group_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::ImplTrait    (it) => {
            let span = it.impl_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Infer        (ud) => {
            let span = ud.underscore_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Macro        (mc) => {
            let span = mc.mac.bang_token.spans[0];
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Never        (nv) => {
            let span = nv.bang_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Paren        (pn) => {
            let span = pn.paren_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Ptr          (pt) => {
            let span = pt.star_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Reference    (rf) => {
            let span = rf.and_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Slice        (sc) => {
            let span = sc.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::TraitObject  (to) => {
            let span = match to.dyn_token {
               Some(dy) => dy.span,
               None     => arrow_token.spans[1],
            };
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Tuple        (tp) => {
            let span = tp.paren_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         _ => {
            proc_macro_error::abort_call_site!("{}", OUTPUT_ERROR_MSG);
         },
      };

      // Look at the last identifier
      // If it is a different Result
      // type to std::result::Result,
      // let quote deal with the mess
      let output = output.segments.last().unwrap();

      // Verify the return type is some kind of Result
      if output.ident != quote::format_ident!("Result") {
         proc_macro_error::abort!(output.ident.span(), "{}", OUTPUT_ERROR_MSG);
      }

      // Unwrap the generic arguments
      let output_args = match &output.arguments {
         syn::PathArguments::AngleBracketed(args) => args,

         syn::PathArguments::Parenthesized(paren) => {
            let span = paren.paren_token.span;
            proc_macro_error::abort!(span, "generic arguments should be surrounded by angle brackets");
         },
         syn::PathArguments::None => {
            let span = output.ident.span();
            proc_macro_error::abort!(span, "Result missing generic arguments");
         },
      };

      // Verify there are exactly 2 generics
      if output_args.args.len() != 2 {
         let span = output_args.lt_token.span;
         proc_macro_error::abort!(span, "Result should have 2 generic arguments");
      }

      // Verify the first generic argument
      // is a type
      let output_arg_ok = match output_args.args.first().unwrap() {
         syn::GenericArgument::Type(ty) => ty,

         syn::GenericArgument::Lifetime   (lt) => {
            let span = lt.apostrophe;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Const      (_)  => {
            let span = output.ident.span();
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Binding    (bd) => {
            let span = bd.eq_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Constraint (ct) => {
            let span = ct.colon_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
      };

      // Verify the first generic argument
      // is a tuple type
      let output_arg_ok = match output_arg_ok {
         syn::Type::Tuple(tp) => tp,
         
         syn::Type::Array        (ar) => {
            let span = ar.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::BareFn       (bf) => {
            let span = bf.fn_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         }
         syn::Type::Group        (gp) => {
            let span = gp.group_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::ImplTrait    (it) => {
            let span = it.impl_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Infer        (ud) => {
            let span = ud.underscore_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Macro        (mc) => {
            let span = mc.mac.bang_token.spans[0];
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Never        (nv) => {
            let span = nv.bang_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Paren        (pn) => {
            let span = pn.paren_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Path         (ph) => {
            let span = ph.path.segments.last().unwrap().ident.span();
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Ptr          (pt) => {
            let span = pt.star_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Reference    (rf) => {
            let span = rf.and_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Slice        (sc) => {
            let span = sc.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::TraitObject  (to) => {
            let span = match to.dyn_token {
               Some(dy) => dy.span,
               None     => arrow_token.spans[1],
            };
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         _ => {
            proc_macro_error::abort_call_site!("{}", OUTPUT_ERROR_MSG);
         },
      };

      // Verify the tuple argument
      // is empty (unit type)
      if output_arg_ok.elems.is_empty() == false {
         let span = output_arg_ok.paren_token.span;
         proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
      }

      // Verify the second generic
      // argument is a type
      let output_arg_err = match output_args.args.last().unwrap() {
         syn::GenericArgument::Type(ty) => ty,

         syn::GenericArgument::Lifetime   (lt) => {
            let span = lt.apostrophe;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Const      (_)  => {
            let span = output.ident.span();
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Binding    (bd) => {
            let span = bd.eq_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Constraint (ct) => {
            let span = ct.colon_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
      };

      // Verify the type is some
      // kind of path
      let output_arg_err = match output_arg_err {
         syn::Type::Path(path) => &path.path,
         
         syn::Type::Array        (ar) => {
            let span = ar.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::BareFn       (bf) => {
            let span = bf.fn_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         }
         syn::Type::Group        (gp) => {
            let span = gp.group_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::ImplTrait    (it) => {
            let span = it.impl_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Infer        (ud) => {
            let span = ud.underscore_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Macro        (mc) => {
            let span = mc.mac.bang_token.spans[0];
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Never        (nv) => {
            let span = nv.bang_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Paren        (pn) => {
            let span = pn.paren_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Ptr          (pt) => {
            let span = pt.star_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Reference    (rf) => {
            let span = rf.and_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Slice        (sc) => {
            let span = sc.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::TraitObject  (to) => {
            let span = match to.dyn_token {
               Some(dy) => dy.span,
               None     => arrow_token.spans[1],
            };
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Tuple        (tp) => {
            let span = tp.paren_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         _ => {
            proc_macro_error::abort_call_site!("{}", OUTPUT_ERROR_MSG);
         },
      };

      // Get the ending path item for the
      // err variant.
      let output_arg_err = output_arg_err.segments.last().unwrap();

      // If the identifier is not 'Box', assume
      // this is some kind of user type implementing
      // the Error trait.
      if output_arg_err.ident != quote::format_ident!("Box") {
         return Ok(Self{
            func     : func,
            variant  : EntrypointReturnType::Static,
         });
      }

      // Verify the Box type has provided
      // generic arguments
      let output_arg_err = match &output_arg_err.arguments {
         syn::PathArguments::AngleBracketed(args) => args,

         syn::PathArguments::Parenthesized(paren) => {
            let span = paren.paren_token.span;
            proc_macro_error::abort!(span, "generic arguments should be surrounded by angle brackets");
         },
         syn::PathArguments::None => {
            let span = output.ident.span();
            proc_macro_error::abort!(span, "Box missing generic arguments");
         },
      };

      // Verify there is exactly one generic argument
      if output_arg_err.args.len() != 1 {
         let span = output_arg_err.lt_token.span;
         proc_macro_error::abort!(span, "Box should have 1 generic argument");
      }

      // Verify the generic argument is a type
      let output_arg_err = match output_arg_err.args.last().unwrap() {
         syn::GenericArgument::Type(ty) => ty,

         syn::GenericArgument::Lifetime   (lt) => {
            let span = lt.apostrophe;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Const      (_)  => {
            let span = output.ident.span();
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Binding    (bd) => {
            let span = bd.eq_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::GenericArgument::Constraint (ct) => {
            let span = ct.colon_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
      };

      // Verify the type is a trait object
      let output_arg_err = match output_arg_err {
         syn::Type::TraitObject(to) => to,

         syn::Type::Array        (ar) => {
            let span = ar.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::BareFn       (bf) => {
            let span = bf.fn_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         }
         syn::Type::Group        (gp) => {
            let span = gp.group_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::ImplTrait    (it) => {
            let span = it.impl_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Infer        (ud) => {
            let span = ud.underscore_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Macro        (mc) => {
            let span = mc.mac.bang_token.spans[0];
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Never        (nv) => {
            let span = nv.bang_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Paren        (pn) => {
            let span = pn.paren_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Path         (ph) => {
            let span = ph.path.segments.last().unwrap().ident.span();
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Ptr          (pt) => {
            let span = pt.star_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Reference    (rf) => {
            let span = rf.and_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Slice        (sc) => {
            let span = sc.bracket_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         syn::Type::Tuple        (tp) => {
            let span = tp.paren_token.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         },
         _ => {
            proc_macro_error::abort_call_site!("{}", OUTPUT_ERROR_MSG);
         },
      };

      // Verify there is only one trait bound
      if output_arg_err.bounds.len() != 1 {
         if let Some(d) = output_arg_err.dyn_token {
            let span = d.span;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         } else {
            proc_macro_error::abort_call_site!("{}", OUTPUT_ERROR_MSG);
         }
      };

      // Verify the trait bound is actually
      // a trait bound
      let output_arg_err = match output_arg_err.bounds.first().unwrap() {
         syn::TypeParamBound::Trait(tr) => tr,
         
         syn::TypeParamBound::Lifetime(lt) => {
            let span = lt.apostrophe;
            proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
         }
      };

      // Make sure the path is not empty
      if output_arg_err.path.segments.is_empty() == true {
         proc_macro_error::abort_call_site!("{}", OUTPUT_ERROR_MSG);
      }

      // Get the last part of the path
      let output_arg_err = output_arg_err.path.segments.last().unwrap();

      // Make sure the ending path identifier is 'Error'
      if output_arg_err.ident != quote::format_ident!("Error") {
         let span = output_arg_err.ident.span();
         proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
      }

      // Let quote deal with any extra
      // corner-case bullshit, we've
      // done enough verification
      return Ok(Self{
         func     : func,
         variant  : EntrypointReturnType::Dynamic,
      });
   }
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
pub fn main(
   attr  : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // attr should not contain anything
   if attr.is_empty() == false {
      proc_macro_error::emit_call_site_error!(
         "macro attributes should be empty"
      );
      return item;
   }

   // Parse attached item into entrypoint info
   let info = syn::parse_macro_input!(item as EntrypointInfo);

   // Miscellaneous variables used to construct
   // the code for main.
   let func    = &info.func;
   let ident   = &func.sig.ident;

   // Construct the syntax for the call
   // to the entrypoint
   return proc_macro::TokenStream::from(match info.variant {
      EntrypointReturnType::Void    => quote::quote! {
         nusion::__build_entry!(#ident, void);
         #func
      },
      EntrypointReturnType::Static  => quote::quote! {
         nusion::__build_entry!(#ident, result_static);
         #func
      },
      EntrypointReturnType::Dynamic => quote::quote! {
         nusion::__build_entry!(#ident, result_dynamic);
         #func
      },
   });
}

///////////////////////////////////////
// ATTRIBUTE MACRO DEFINITION - hook //
///////////////////////////////////////

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
   // Parse item as a function
   let hook = syn::parse_macro_input!(item as syn::ItemFn);

   // Parse attributes as arguments
   let args = syn::parse_macro_input!(attr as HookArguments);

   // Generate a unique identifier for the function
   use core::hash::{Hash, Hasher};
   let mut hasher = HookHasher::new();
   hook.hash(& mut hasher);

   // Store the unique identifier as a u64
   let uuid = hasher.finish();

   // Create identifiers for ASM and intermediary hooks
   const IDENT_PREFIX : &'static str = "__nusion_hook";

   let ident_asm = quote::format_ident!(
      "{IDENT_PREFIX}_asm_{:X}_{}",
      uuid,
      hook.sig.ident,
   );
   let ident_int = quote::format_ident!(
      "{IDENT_PREFIX}_int_{:X}_{}",
      uuid,
      hook.sig.ident,
   );

   // Substitute in template arguments
   let asm = args.substitute_arguments(
      & ident_asm,
      & ident_int,
   );

   // Store input and output for hook
   let input_hook    = &hook.sig.inputs;
   let output_hook   = &hook.sig.output;

   // Convert the hook input to a list of
   // arguments to call the hook
   let mut input_call_hook = syn::punctuated::Punctuated::<
      syn::Ident, syn::token::Comma,
   >::new();
   
   for p in hook.sig.inputs.iter() {
      // Isolate the identifier for the argument
      let p = match p {
         syn::FnArg::Typed(p) => p,
         _                    => {
            proc_macro_error::emit_call_site_error!(
               "hook function argument has no type",
            );
            continue;
         },
      };
      let p = match &*p.pat {
         syn::Pat::Ident(p)   => p,
         _                    => {
            proc_macro_error::emit_call_site_error!(
               "hook function argument has no identifier",
            );
            continue;
         },
      };
      let p = &p.ident;

      // Add the identifier to the argument list
      input_call_hook.push(p.clone());
   }

   // Get miscellaneous references to prepare
   // for construction of the code
   let ident_hook_ptr   = &args.ptr_ident;
   let ident_hook       = &hook.sig.ident;

   // Finally, put everything together
   // and construct the syntax for the
   // ASM hook, intermediary, and hook
   return proc_macro::TokenStream::from(quote::quote!{
      // Assembly hook
      core::arch::global_asm!(#asm);

      // Declaration to the assembly hook
      extern "C" {
         #[no_mangle]
         #[allow(non_snake_case)]
         pub fn #ident_asm();
      }

      // Void pointer to the assembly hook
      const #ident_hook_ptr : * const core::ffi::c_void
         = #ident_asm as * const core::ffi::c_void;

      // Intermediate for calling the user's hook
      #[no_mangle]
      #[allow(non_snake_case)]
      pub extern "C" fn #ident_int(
         #input_hook
      ) #output_hook {
         return #ident_hook(#input_call_hook);
      }

      // User's hook function
      #hook
   });
}

#[derive(Debug)]
struct HookArguments {
   pub asm_template  : syn::LitStr,
   pub ptr_ident     : syn::Ident,
}

impl HookArguments {
   pub fn substitute_arguments(
      & self,
      identifier_asm : & syn::Ident,
      identifier_int : & syn::Ident,
   ) -> String {
      lazy_static::lazy_static! {
         static ref FIND_ARGUMENTS : regex::Regex = regex::Regex::new(
            r"\{[[:alpha:]]*?\}",
         ).unwrap();
      }
      let mut output = String::new();

      // Construct subroutine label
      output.push_str(&format!(
         ".section .text\n{identifier_asm}:\n",
      ));

      // Substitute template arguments
      output.push_str(&FIND_ARGUMENTS.replace(
         &self.asm_template.value(),
         HookAsmParser{
            identifier_int : identifier_int,
         },
      ));

      // Return formatted ASM
      return output;
   }
}

impl syn::parse::Parse for HookArguments {
   fn parse(
      input : syn::parse::ParseStream<'_>,
   ) -> syn::parse::Result<Self> {
      // Required - String literal containing ASM template
      let asm_template = input.parse::<syn::LitStr>()?;

      // Required - Comma separating ASM template and identifier
      input.parse::<syn::Token![,]>()?;

      // Required - Identifier for the ASM pointer constant
      let ptr_ident = input.parse::<syn::Ident>()?;

      // Optional - Trailing comma after the identifier
      input.parse::<Option<syn::Token![,]>>()?;

      // Create Arguments struct and return
      return Ok(Self{
         asm_template   : asm_template,
         ptr_ident      : ptr_ident,
      });
   }
}

#[derive(Debug)]
struct HookAsmParser<'s> {
   pub identifier_int : &'s syn::Ident,
}

impl<'s> regex::Replacer for HookAsmParser<'s> {
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

         // Strip out surrounding curly brackets
         let cap = &cap[1..cap.len()-1];

         // Try to parse as an argument
         let arg = match cap.parse::<HookArgumentType>() {
            Ok(arg)  => arg,
            Err(_)   => panic!("invalid template argument \"{cap}\""),
         };

         // Stringify the actual value for the argument
         let arg = match arg {
            HookArgumentType::IdentifierIntermediate
               => self.identifier_int.to_string(),
         };

         // Append the substituted string to the buffer
         dst.push_str(&arg);
      }
      return;
   }
}

#[derive(Clone, Debug)]
enum HookArgumentType {
   IdentifierIntermediate,
}

impl std::str::FromStr for HookArgumentType {
   type Err = ();

   fn from_str(
      s : & str,
   ) -> Result<Self, Self::Err> {
      use std::collections::hash_map::HashMap;

      // Hash map for argument types
      lazy_static::lazy_static! {
         static ref ARGUMENT_MAP : HashMap<&'static str, HookArgumentType> = [
            ("hook", HookArgumentType::IdentifierIntermediate),
         ].iter().cloned().collect();
      }

      return ARGUMENT_MAP.get(s).ok_or(()).map(|a| a.clone());
   }
}

#[derive(Debug)]
struct HookHasher {
   crc : u64,
}

impl HookHasher {
   pub fn new() -> Self {
      return Self{
         crc : 0,
      };
   }
}

impl core::hash::Hasher for HookHasher {
   fn finish(
      & self,
   ) -> u64 {
      return self.crc;
   }

   fn write(
      & mut self,
      bytes : & [u8],
   ) {
      self.crc = crc::Crc::<u64>::new(
         & crc::CRC_64_WE,
      ).checksum(bytes);
      return;
   }
}

