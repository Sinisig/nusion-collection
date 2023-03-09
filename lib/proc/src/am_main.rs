/// Implementation of the main
/// attribute macro.
pub fn main(
   attr  : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // Parse attached item into entrypoint info
   let info = syn::parse_macro_input!(item as EntrypointInfo);

   // Parse the process filter list
   let allow_list = syn::parse_macro_input!(
      attr as EntrypointProcessAllowList
   ).list;

   // Miscellaneous variables used to construct
   // the code for main.
   let func    = &info.func;
   let ident   = &func.sig.ident;

   // Construct the syntax for the call
   // to the entrypoint
   return proc_macro::TokenStream::from(match info.variant {
      EntrypointReturnType::Void    => quote::quote! {
         nusion::__build_entry!(#ident, void, #(#allow_list),*);
         #func
      },
      EntrypointReturnType::Static  => quote::quote! {
         nusion::__build_entry!(#ident, result_static, #(#allow_list),*);
         #func
      },
      EntrypointReturnType::Dynamic => quote::quote! {
         nusion::__build_entry!(#ident, result_dynamic, #(#allow_list),*);
         #func
      },
   });
}

struct EntrypointInfo {
   func     : syn::ItemFn,
   variant  : EntrypointReturnType,
}

enum EntrypointReturnType {
   Void,    // -> () or no return type
   Static,  // -> Result<(), E: std::error::Error>
   Dynamic, // -> Result<(), Box<dyn std::error::Error>>
}

/// Gets the span for a visibility
/// enum
fn span_vis(
   vis : & syn::Visibility
) -> proc_macro2::Span {
   use syn::Visibility::*;

   return match vis {
      Public      (tok)
         => tok.pub_token.span,
      Crate       (tok)
         => tok.crate_token.span,
      Restricted  (tok)
         => tok.paren_token.span,
      Inherited
         => proc_macro2::Span::call_site(),
   };
}

/// Gets the span for a type enum
fn span_type(
   ty : & syn::Type,
) -> proc_macro2::Span {
   use syn::Type::*;

   return match ty {
      Array       (ar)
         => ar.bracket_token.span,

      BareFn      (bf)
         => bf.fn_token.span,

      Group       (gp)
         => gp.group_token.span,

      ImplTrait   (it)
         => it.impl_token.span,

      Infer       (ud)
         => ud.underscore_token.span,

      Macro       (mc)
         => mc.mac.bang_token.spans[0],

      Never       (nv)
         => nv.bang_token.span,

      Paren       (pn)
         => pn.paren_token.span,

      Path        (pa)
         => pa.path.segments.first().unwrap().ident.span(),

      Ptr         (pt)
         => pt.star_token.span,

      Reference   (rf)
         => rf.and_token.span,

      Slice       (sc)
         => sc.bracket_token.span,

      TraitObject (to)
         => match to.dyn_token {
            Some(dy) => dy.span,
            None     => proc_macro2::Span::call_site(),
         },

      Tuple       (tp)
         => tp.paren_token.span,

      _
         => proc_macro2::Span::call_site(),
   };
}

/// Gets the span for a generic argument
fn span_generic_argument(
   ga : & syn::GenericArgument,
) -> proc_macro2::Span {
   use syn::GenericArgument::*;

   return match ga {
      Lifetime    (lt)
         => lt.apostrophe,

      Type        (ty)
         => span_type(&ty),

      Const       (_) 
         => proc_macro2::Span::call_site(),

      Binding     (bd)
         => bd.eq_token.span,

      Constraint  (ct)
         => ct.colon_token.span,
   };
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
         syn::Visibility::Inherited => (),
         
         _ => proc_macro_error::emit_error!(
            span_vis(&func.vis), "visibility should be private",
         ),         
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
      let (_, output) = match &func.sig.output {
         syn::ReturnType::Default => {
            return Ok(Self{
               func     : func,
               variant  : EntrypointReturnType::Void,
            });
         },
         syn::ReturnType::Type(ar, ty) => (ar, ty),
      };

      // Make sure the type is a type path
      let output = if let syn::Type::Path(p) = &**output {
         &p.path
      } else {
         proc_macro_error::abort!(
            span_type(&**output),
            "{}",
            OUTPUT_ERROR_MSG,
         );
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
      let output_arg_ok = output_args.args.first().unwrap();
      let output_arg_ok = if let syn::GenericArgument::Type(ty) = output_arg_ok {
         ty
      } else {
         proc_macro_error::abort!(
            span_generic_argument(output_arg_ok),
            "{}",
            OUTPUT_ERROR_MSG,
         );
      };
      
      // Verify the first generic argument
      // is a tuple type
      let output_arg_ok = if let syn::Type::Tuple(tp) = output_arg_ok {
         tp
      } else {
         proc_macro_error::abort!(
            span_type(output_arg_ok),
            "{}",
            OUTPUT_ERROR_MSG,
         );
      };

      // Verify the tuple argument
      // is empty (unit type)
      if output_arg_ok.elems.is_empty() == false {
         let span = output_arg_ok.paren_token.span;
         proc_macro_error::abort!(span, "{}", OUTPUT_ERROR_MSG);
      }

      // Verify the second generic
      // argument is a type
      let output_arg_err = output_args.args.last().unwrap();
      let output_arg_err = if let syn::GenericArgument::Type(ty) = output_arg_err {
         ty
      } else {
         proc_macro_error::abort!(
            span_generic_argument(output_arg_err),
            "{}",
            OUTPUT_ERROR_MSG,
         );
      };

      // Verify the type is some
      // kind of path
      let output_arg_err = if let syn::Type::Path(p) = output_arg_err {
         &p.path
      } else {
         proc_macro_error::abort!(
            span_type(output_arg_err),
            "{}",
            OUTPUT_ERROR_MSG,
         );
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
      let output_arg_err = output_arg_err.args.last().unwrap();
      let output_arg_err = if let syn::GenericArgument::Type(ty) = output_arg_err {
         ty
      } else {
         proc_macro_error::abort!(
            span_generic_argument(output_arg_err),
            "{}",
            OUTPUT_ERROR_MSG,
         );
      };

      // Verify the type is a trait object
      let output_arg_err = if let syn::Type::TraitObject(to) = output_arg_err {
         to
      } else {
         proc_macro_error::abort!(
            span_type(output_arg_err),
            "{}",
            OUTPUT_ERROR_MSG,
         );
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

struct EntrypointProcessAllowList {
   list  : Vec<syn::LitStr>,
}

impl syn::parse::Parse for EntrypointProcessAllowList {
   fn parse(
      input : syn::parse::ParseStream<'_>,
   ) -> syn::parse::Result<Self> {
      let mut output = Vec::new();

      while input.is_empty() == false {
         // Required - String literal for the process name
         let proc = input.parse::<syn::LitStr>()?;

         // Required if not last element - comma separator
         if let Err(e) = input.parse::<syn::Token![,]>() {
            if input.is_empty() == false {
               return Err(e);
            }
         } 

         output.push(proc);
      }

      return Ok(Self{
         list : output
      });
   }
}

