//! Crate root for nusion-sys-proc-macros, a collection
//! of procedural macros to be incorporated into
//! nusion-sys.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

#[proc_macro]
#[proc_macro_error::proc_macro_error]
pub fn entry(item : proc_macro::TokenStream) -> proc_macro::TokenStream {
   return item;
}

