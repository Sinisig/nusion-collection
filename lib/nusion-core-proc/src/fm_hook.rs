/// Implementation of the hook
/// function-like macro.
pub fn hook(
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // Parse input item as a string literal and closure
   let input = syn::parse_macro_input!(item as HookInput);

   // Generate input UUID
   let uuid = input.generate_uuid(); 

   // Generate identifiers for the private
   // module, ASM trampoline, and closure
   const IDENT_PREFIX : &'static str = "__nusion_core_hook";
   let ident = HookIdentifier{
      module      : quote::format_ident!(
         "{IDENT_PREFIX}_{:X}_module",       uuid,
      ),
      trampoline  : quote::format_ident!(
         "{IDENT_PREFIX}_{:X}_trampoline",   uuid,
      ),
      closure     : quote::format_ident!(
         "{IDENT_PREFIX}_{:X}_closure",      uuid,
      ),
   };

   // Parse the assembly template
   let asm_template = input.parse_asm_template(&ident);
  
   // Unpack various variables for use in the quote invocation
   let module_ident        = &ident.module;
   let asm_template_ident  = &ident.trampoline;
   let closure_ident       = &ident.closure;
   let closure_input       = &input.closure.inputs;
   let closure_output      = &input.closure.output;
   let closure_body        = &input.closure.body;

   // Finally, generate the Rust code for the hook
   return proc_macro::TokenStream::from(quote::quote!{
      // Create scope for functions
      {
         // Create a module to store all our
         // functions in.  This is an easy way
         // to fudge our way around issues with
         // global_asm!() being used in a statement
         mod #module_ident {
            // Import items from environment
            use super::*;

            // Assembly trampoline code gen
            core::arch::global_asm!(#asm_template);
   
            // Declaration of the assembly function
            #[allow(non_snake_case)]
            extern "C" {
               pub fn #asm_template_ident();
            }

            // Construct a function from the closure
            #[no_mangle]
            #[allow(non_snake_case)]
            pub extern "C" fn #closure_ident(
               #closure_input
            ) #closure_output {
               #closure_body
            }
         }

         // Finally, we return the asm template pointer
         #module_ident::#asm_template_ident
      }
   });
}

struct HookIdentifier {
   pub module     : syn::Ident,
   pub trampoline : syn::Ident,
   pub closure    : syn::Ident,
}

struct HookInput {
   pub asm_template  : syn::LitStr,
   pub closure       : syn::ExprClosure,
}

impl HookInput {
   pub fn generate_uuid(
      & self
   ) -> u64 {
      use core::hash::{Hash, Hasher};

      let mut uuid_hasher = hashers::fnv::FNV1aHasher64::default();

      // In order to have the lowest possible
      // chance of generating duplicate hashes,
      // we take into account the ASM string
      // literal, closure content, file position
      // of literal, and file position of closure.
      self.asm_template                      .hash(& mut uuid_hasher);
      self.closure                           .hash(& mut uuid_hasher);
      self.asm_template.span().start()       .hash(& mut uuid_hasher);
      self.asm_template.span().end()         .hash(& mut uuid_hasher);
      self.closure.or1_token.spans[0].start().hash(& mut uuid_hasher);
      self.closure.or2_token.spans[0].start().hash(& mut uuid_hasher);

      return uuid_hasher.finish();
   }

   pub fn parse_asm_template(
      & self,
      identifiers : & HookIdentifier,
   ) -> syn::LitStr {
      lazy_static::lazy_static!{
         static ref ARG_SEARCHER : regex::Regex = regex::Regex::new(
            r"\{[^\{\}]*?\}"
         ).expect("Failed to parse Regex! This is a bug in the macro!");
      };

      // Substitute template arguments
      let output = ARG_SEARCHER.replace(
         &self.asm_template.value(),
         HookSubstitutor::new(identifiers, self.asm_template.span()),
      ).into_owned();

      // Create the fully-constructed assembly template
      let label_trampoline = &identifiers.trampoline;
      let output = format!("
         {label_trampoline}:  // Start label for the trampoline
         {output}             // Previously parsed ASM
      ");

      // Re-construct LitStr and return
      return syn::LitStr::new(&output, self.asm_template.span());
   }
}

impl syn::parse::Parse for HookInput {
   fn parse(
      input : syn::parse::ParseStream<'_>,
   ) -> syn::parse::Result<Self> {
      // Required - String literal containing the ASM template
      let asm_template = input.parse::<syn::LitStr>()?;

      // Required - Comma separating the next argument
      input.parse::<syn::Token![,]>()?;

      // Required - Closure which will be called
      let closure = input.parse::<syn::ExprClosure>()?;

      // Optional - Trailing comma after the last argument
      input.parse::<Option<syn::Token![,]>>()?;

      // Verify every argument for the closure
      // contains a concrete type
      for pat in &closure.inputs {
         if let syn::Pat::Ident(id) = pat {
            let id   = &id.ident;
            let span = id.span();
            proc_macro_error::abort!(span,
               "closure argument \"{}\" must have a concrete type", id,
            );
         }

         if let syn::Pat::Type(ty) = pat {
            let ty = &*ty.ty;

            if let syn::Type::Infer(ty) = ty {
               let span = ty.underscore_token.span;
               proc_macro_error::abort!(span,
                  "closure arguments may not infer their type",
               );
            }
         }
      }

      // Verify the return type is either
      // nothing, in which case assume void
      // return, or it is a concrete type.
      if let syn::ReturnType::Type(_, ty) = &closure.output {
         let ty = &**ty;
         if let syn::Type::Infer(ty) = ty {
            let span = ty.underscore_token.span;
            proc_macro_error::abort!(span,
               "closure return type may not be inferred",
            );
         }
      }

      // Verify there the move keyword wasn't used
      if let Some(mv) = &closure.capture {
         let span = mv.span;
         proc_macro_error::abort!(span,
            "closure may not take ownership of environment variables",
         );
      }

      // Verify the async keyword wasn't used
      if let Some(ay) = &closure.asyncness {
         let span = ay.span;
         proc_macro_error::abort!(span,
            "closure may not be async",
         );
      }

      // Let quote deal with any more mess,
      // we've done our job.
      return Ok(Self{
         asm_template   : asm_template,
         closure        : closure,
      });
   }
}

enum HookArgument {
   IdentifierTrampoline,
   IdentifierClosure,
}

enum HookArgumentError {
   UnknownArgument,
   UnexpectedParameter,
}

impl std::str::FromStr for HookArgument {
   type Err = HookArgumentError;

   fn from_str(
      s : & str,
   ) -> Result<Self, Self::Err> {
      use std::collections::HashMap;
      lazy_static::lazy_static! {
         static ref ARG_MAP : HashMap<&'static str, HookArgument> = {
            let mut map = HashMap::with_capacity(ARG_COUNT);

            // Add custom arguments here
            const ARG_COUNT : usize = 2;
            map.insert("self",   HookArgument::IdentifierTrampoline);
            map.insert("target", HookArgument::IdentifierClosure);

            map
         };
      };

      // Isolate the argument and parameter
      let (
         arg,
         param,
      ) = s.trim().split_once(char::is_whitespace).unwrap_or((s, ""));
      let arg     = arg.trim();
      let param   = param.trim();

      // Parse into an argument enum
      let arg = ARG_MAP.get(arg).ok_or(HookArgumentError::UnknownArgument)?;

      // Parse the parameter depending on the argument type
      return match arg {
         HookArgument::IdentifierTrampoline  => {
            if param.is_empty() == false {
               Err(HookArgumentError::UnexpectedParameter)
            } else {
               Ok(HookArgument::IdentifierTrampoline)
            }
         },
         HookArgument::IdentifierClosure     => {
            if param.is_empty() == false {
               Err(HookArgumentError::UnexpectedParameter)
            } else {
               Ok(HookArgument::IdentifierClosure)
            }
         },
      };
   }
}

struct HookSubstitutor<'s> {
   ident : &'s HookIdentifier,
   span  : proc_macro2::Span,
}

impl<'s> HookSubstitutor<'s> {
   pub fn new(
      ident : &'s HookIdentifier,
      span  : proc_macro2::Span,
   ) -> Self {
      return Self{
         ident : ident,
         span  : span,
      };
   }
}

impl<'s> regex::Replacer for HookSubstitutor<'s> {
   fn replace_append(
      & mut self,
      caps  : & regex::Captures<'_>,
      dst   : & mut String,
   ) {
      for cap in caps.iter() {
         let cap = match cap {
            Some(cap)   => cap,
            None        => break,
         };

         // Get capture as a string slice
         let cap = cap.as_str();

         // Strip out surrounding curly brackets
         let cap = &cap[1..cap.len()-1];

         // Attempt to parse argument text
         let arg = match cap.parse::<HookArgument>() {
            Ok(arg)  => arg,
            Err(e)   => {match e {
               HookArgumentError::UnknownArgument
                  => proc_macro_error::abort!(self.span,
                     "assembly template contains unknown argument type \"{}\"", cap,
                  ),
               HookArgumentError::UnexpectedParameter
                  => proc_macro_error::abort!(self.span,
                     "assembly template argument \"{}\" has unexpected parameters", cap,
                  ),
            }},
         };

         // Substitute the argument for its real value
         let arg = match arg {
            HookArgument::IdentifierTrampoline
               => format!("{}", &self.ident.trampoline),
            HookArgument::IdentifierClosure
               => format!("{}", &self.ident.closure),
         };

         // Append the generated text to the buffer
         dst.push_str(&arg);
      }

      return;
   }
}

