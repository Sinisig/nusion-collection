// Implementation of the asm_bytes
// function-like macro.
pub fn asm_bytes(
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   // Parse input and generate UUID
   let input   = syn::parse_macro_input!(item as AsmBytesInput);
   let uuid    = input.generate_uuid();

   // Build identifiers based on UUID
   const IDENT_PREFIX : &'static str = "__nusion_core_asm_bytes";
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

            // Declarations of function pointers
            #[allow(non_snake_case)]
            extern "C" {
               fn #asm_ident_start();
               fn #asm_ident_end();
            }

            // Byte pointers to the function pointers.
            // We have to do it this way to force the
            // compiler to lay the labels next to each
            // other in memory.  Otherwise, it can re-order
            // them into memory if they were static variables
            // and break everything.
            pub const ASM_START  : * const u8 = #asm_ident_start  as * const u8;
            pub const ASM_END    : * const u8 = #asm_ident_end    as * const u8;
         }

         // Construct the byte slice from the
         // created pointers.  This is the part
         // which breaks 'const' on older versions
         // of the standard library.
         unsafe{std::slice::from_raw_parts(
            #module_ident::ASM_START,
            usize::try_from(#module_ident::ASM_END.offset_from(
               #module_ident::ASM_START,
            )).expect("ASM end pointer is before start pointer! This is a bug in the macro!"),
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
      let user_assembly = self.asm_template.value();
      let label_start   = &identifiers.asm_label_start;
      let label_end     = &identifiers.asm_label_end;
      let span          = self.asm_template.span();
      

      return syn::LitStr::new(&format!("
         .section .rodata        // Mark as non-executable

         {label_start}:          // Start label
         {user_assembly}         // User's assembly code
         {label_end}:            // End label

         .section .text          // Restore text section
      "), span);
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
         asm_template : asm_template
      });
   }
}

