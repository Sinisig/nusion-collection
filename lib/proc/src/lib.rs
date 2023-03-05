//! Crate root for nusion-proc-macros, a collection
//! of procedural macros to be incorporated into
//! nusion.
//!
//! It is not recommended to use this crate directly,
//! but instead include nusion as a dependency, as
//! nusion re-exports all macros in this crate.

//////////////////////
// INTERNAL MODULES //
//////////////////////

mod pm_main;
mod pm_hook;

//////////////////////////////////
// PROCEDURAL MACRO DEFINITIONS //
//////////////////////////////////

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
   return pm_main::main(attr, item);
}

/// Generates an ASM trampoline and
/// Rust function pair, returning the
/// function pointer to the ASM
/// trampoline.  It is recommended
/// to use this macro to initialize
/// the <code>target_hook</code> field
/// of the
/// <code>nusion::patch::method::Hook</code>
/// struct.
///
/// <h2 id=  hook_syntax>
/// <a href=#hook_synatx>
/// Syntax
/// </a></h2>
/// The first argument should be a
/// string literal serving as an
/// assembly template similar to the
/// <code><a href=
/// https://doc.rust-lang.org/stable/core/arch/macro.asm.html
/// >asm!()</a></code> macro, but
/// there are no options and template
/// arguments take a new meaning.
///
/// The second argument will be a
/// function which is called by the
/// ASM trampoline.  Syntactically
/// it will look like a closure, but
/// the macro will convert it to a
/// normal function.  This means all
/// input parameters to the "closure"
/// must have concrete types and the
/// return type must also be a concrete
/// type.  Type inference is not allowed.
/// In addition, the "closure" may not
/// modify its environment.  It must
/// be in a form which would implement
/// the
/// <code><a href=
/// https://doc.rust-lang.org/std/ops/trait.Fn.html
/// >Fn()</a></code> trait.
///
/// <h2 id=  hook_asm_template_arguments>
/// <a href=#hook_asm_template_arguments>
/// ASM Template Arguments
/// </a></h2>
/// All template arguments are surrounded
/// by curly brackets.  Inside the curly
/// brackets should be the input for the
/// argument.  This is not to be confused
/// with
/// <code><a href=
/// https://doc.rust-lang.org/std/macro.format.html
/// >format!()</a></code>'s argument style,
/// as inputs cannot be specified outside
/// the ASM template string.  They must be
/// specified within the curly brackets
/// inside the ASM template.
///
/// The following is a complete list of
/// valid template arguments:
/// <ul>
/// <li>
/// <code>closure</code> - Substitutes
/// the argument for the ASM-compatiable
/// generated label for the input closure.
/// Use this to call the closure from the
/// ASM trampoline.
/// </li>
/// </ul>
///
/// <h2 id=  hook_safety>
/// <a href=#hook_safety>
/// Safety
/// </a></h2>
/// It is assumed the ASM template code
/// is valid for its use case and will
/// never lead to any undefined behavior
/// nor memory safety violations.  Failing
/// to do so will lead to catastrophic bugs
/// which will be near impossible to debug.
#[proc_macro]
#[proc_macro_error::proc_macro_error]
pub fn hook(
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   return pm_hook::hook(item);
}

