/// Implementation of the main
/// attribute macro.
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

