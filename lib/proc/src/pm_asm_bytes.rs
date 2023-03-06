// Implementation of the asm_bytes
// function-like macro.
pub fn asm_bytes(
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // Parse input and generate UUID
   let input   = syn::parse_macro_input!(item as AsmBytesInput);
   let uuid    = input.generate_uuid();

   // Build identifiers based on UUID
   const IDENT_PREFIX : &'static str = "__nusion_asm_bytes";
   let ident   = AsmBytesIdentifier{
      asm_label_start   : quote::format_ident!(
         "{IDENT_PREFIX}_{:X}_asm_start", uuid,
      ),
      asm_label_end     : quote::format_ident!(
         "{IDENT_PREFIX}_{:X}_asm_end",   uuid,
      ),
      module            : quote::format_ident!(
         "{IDENT_PREFIX}_{:X}_module",    uuid,
      ),
   };

   // Parse the assembly template
   let asm_template = input.parse_asm_template(&ident);

   // Unpack variables for use in quote block
   let asm_ident_start  = &ident.asm_label_start;
   let asm_ident_end    = &ident.asm_label_end;
   let module_ident     = &ident.module;

   return proc_macro::TokenStream::from(quote::quote!{
      // Create scope to define ASM
      {
         // Use the same module trick from
         // hook!() to define the ASM
         mod #module_ident {
            // Import items from environment
            use super::*;

            // Assembly bytes code gen
            core::arch::global_asm!(#asm_template);

            // Declarations of pointers
            #[no_mangle]
            #[allow(non_snake_case)]
            extern "C" {
               pub fn #asm_ident_start();
               pub fn #asm_ident_end();
            }
         }

         // Construct the byte slice from the
         // created pointers.  This is the part
         // which fucks up on older version of
         // std.
         unsafe{std::slice::from_raw_parts(
            #module_ident::#asm_ident_start as * const u8,
            (#module_ident::#asm_ident_end as * const u8).offset_from(
               #module_ident::#asm_ident_start as * const u8,
            ) as usize,
         )}
      }
   });
}

struct AsmBytesInput {
   pub asm_template  : syn::LitStr,
}

struct AsmBytesIdentifier {
   pub asm_label_start  : syn::Ident,
   pub asm_label_end    : syn::Ident,
   pub module           : syn::Ident,
}

impl AsmBytesInput {
   pub fn generate_uuid(
      & self,
   ) -> u64 {
      use core::hash::{Hash, Hasher};

      let mut uuid_hasher = hashers::fnv::FNV1aHasher64::default();

      // Takes into account the literal
      // string itself and the position
      // in the file (span) to minimize
      // chance of generating duplicate
      // UUIDs.
      self.asm_template                .hash(& mut uuid_hasher);
      self.asm_template.span().start() .hash(& mut uuid_hasher);
      self.asm_template.span().end()   .hash(& mut uuid_hasher);

      return uuid_hasher.finish();
   }

   pub fn parse_asm_template(
      & self,
      identifiers : & AsmBytesIdentifier,
   ) -> syn::LitStr {
      // All this basically does it append
      // labels and rodata section
      let asm  = self.asm_template.value();
      let span = self.asm_template.span();

      return syn::LitStr::new(&format!(
         "
         .section .rodata
         {}:
         {}
         {}:
         ",
         identifiers.asm_label_start,
         asm,
         identifiers.asm_label_end
      ), span);
   }
}

impl syn::parse::Parse for AsmBytesInput {
   fn parse(
      input : syn::parse::ParseStream<'_>,
   ) -> syn::parse::Result<Self> {
      // Required - String literal containing the ASM
      let asm_template = input.parse::<syn::LitStr>()?;

      // Optional - Trailing comma
      input.parse::<Option<syn::Token![,]>>()?;

      // Create the input and return
      return Ok(Self{
         asm_template   : asm_template
      });
   }
}

