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

mod am_main;
mod fm_hook;
mod fm_asm_bytes;

//////////////////////////////////
// PROCEDURAL MACRO DEFINITIONS //
//////////////////////////////////

/// Builds a shared library entrypoint
/// using the attached function.  The
/// attached function is semantically
/// equivalent to a binary crate's
/// <code>main</code>.
/// 
/// <h2 id=  main_syntax>
/// <a href=#main_syntax>
/// Syntax
/// </a></h2>
/// The macro should only be attached to
/// a single function which takes the form
/// of a binary crate's <code>main()</code>.
/// The following are valid forms:
/// <ul>
/// <li><code>
/// fn main()
/// </code></li>
/// <li>
/// <code>
/// fn main() -> Result&lt;(), E&gt;
/// </code>
/// where <code>E</code> is some
/// type which implements the trait
/// <code>std::error::Error</code>.
/// </li>
/// <li><code>
/// fn main() -> Result&lt;(), Box&lt;dyn std::error::Error&gt;&gt;
/// </code></li>
/// </ul>
///
/// The attribute input for the macro
/// may also take a list of process names
/// which are allowed to be attached to.
/// If the library is loaded into a process
/// which doesn't match the name of any
/// process in this list, it will exit
/// before executing the main function.
/// This process name list is a comma-separated
/// list of string literals.
///
/// <h2 id=  main_example>
/// <a href=#main_example>
/// Examples
/// </a></h2>
///
/// <h6 id=  main_examples_basic>
/// <a href=#main_examples_basic>
/// Basic entrypoint
/// </a></h6>
///
/// ```
/// #[nusion_lib::main]
/// fn main() {
///    println!("Hello, World!");
/// }
/// ```
///
/// <h6 id=  main_examples_static>
/// <a href=#main_examples_static>
/// Falliable main with a static error type
/// </a></h6>
///
/// ```
/// #[derive(Debug)]
/// struct MainError;
///
/// impl std::fmt::Display for MainError {
///    fn fmt(& self, stream : & mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///       return write!(stream, "Oopsy poopsy! Main returned an error!");
///    }
/// }
///
/// impl std::error::Error for MainError {
/// }
///
/// #[nusion_lib::main]
/// fn main() -> Result<(), MainError> {
///    println!("Hello, World!");
///    return Ok(());
/// }
/// ```
///
/// <h6 id=  main_examples_dynamic_whitelist>
/// <a href=#main_examples_dynamic_whitelist>
/// Falliable main with a dynamic error type and process whitelist
/// </a></h6>
///
/// ```
/// #[nusion_lib::main("calculator.exe", "paint.exe")]
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///    println!("Hello, World!");
///    return Ok(());
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn main(
   attr  : proc_macro::TokenStream,
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   return am_main::main(attr, item);
}

/// Generates an ASM trampoline and
/// Rust function pair, returning the
/// function pointer to the ASM
/// trampoline.  It is recommended
/// to use this macro to initialize
/// the <code>hook</code> field of the
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
/// >asm!</a></code> macro, but
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
/// >Fn</a></code> trait.
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
/// >format!</a></code>'s argument style,
/// as inputs cannot be specified outside
/// the ASM template string.  They must be
/// specified within the curly brackets
/// inside the ASM template.
///
/// The following is a complete list of
/// valid template arguments:
/// <ul>
/// <li>
/// <code>self</code> - The ASM-compatiable
/// label for the ASM trampoline.
/// </li>
/// <li>
/// <code>target</code> - The ASM-compatiable
/// label for the Rust closure.  Use this argument
/// to call your closure from your ASM trampoline.
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
///
/// Here are the main things to be
/// aware of when writing your ASM
/// trampoline:
/// <ul>
/// <li>
/// The hook obeys the platform's C
/// ABI / Calling Convention.
/// </li>
/// <li>
/// All input arguments follow the correct
/// format for Rust arguments and are all
/// valid for their context.
/// </li>
/// <li>
/// All instructions overwritten by
/// the hook are executed exactly as
/// they were (stolen instructions)
/// </li>
/// <li>
/// Any stolen instructions which use
/// stack-relative offsets are adjusted
/// due to the call to the ASM trampoline
/// to point to the same memory address
/// </li>
/// <li>
/// All volatile registers in use by the
/// interrupted function are restored to
/// their intended values after the ASM
/// trampoline returns
/// </li>
/// </ul>
///
/// <h2 id=  hook_examples>
/// <a href=#hook_examples>
/// Examples
/// </a></h2>
///
/// <h6 id=  hook_examples_basic>
/// <a href=#hook_examples_basic>
/// Basic hook which does effectively nothing
/// </a></h6>
///
/// ```
/// const HOOK_NOTHING : nusion_lib::patch::writer::Hook = nusion_lib::patch::writer::Hook{
///    memory_offset_range  : 0x7FFF0000..0x7FFF00D,
///    checksum             : nusion_lib::patch::Checksum::from(0),
///    hook                 : nusion_lib::hook!("
///       // Even though we aren't doing anything
///       // with this hook, we still need to compensate
///       // for the instructions we overwrote creating
///       // the hook and re-write them here
///       xor   eax,eax
///       mov   ecx,[rsp-0x10] // With stack-relative offsets, we need
///       sub   edx,ecx        // to remember the function call pushed
///       mov   [rsp-0x08],edx // the return value onto the stack and
///                            // subsequently changed the value of the
///                            // stack pointer.  Therefore, any stolen
///                            // instructions need to account for this
///                            // offset by adding 0x08 to all stack-relative
///                            // instructions for 64-bit Intel/AMD.  The
///                            // actual amount depends on your architecture.
///       
///       // After we execute the stolen bytes/instructions,
///       // we can do whatever we want.  If we call a function,
///       // we must adhere to our platform's ABI rules for calling
///       // a function.  In addition, since we interrupted a
///       // function mid-execution, we also mut preserve all volatile
///       // registers which were in-use where we hooked the function.
///
///       // Since we aren't doing anything, we can
///       // safely return to the hooked function.
///       ret
///    ", || {}),
/// }
/// ```
///
/// <h6 id=  hook_examples_realistic>
/// <a href=#hook_examples_realistic>
/// More realistic hook likely to be used
/// </a></h6>
///
/// ```
/// const HOOK_GME_2020 : nusion_lib::patch::writer::Hook = nusion_lib::patch::writer::Hook{
///    memory_offset_range  : 0x7FFF1337..0x7FFF133C,
///    checksum             : nusion_lib::patch::Checksum::from(0xDEADBEEF),
///    hook                 : nusion_lib::hook!("
///       // Stolen bytes
///       xor   edi,rax
///       sub   edi,[rcx+0x100]
///       mov   [rcx+0x104],edi
///
///       // Align stack and store volatiles
///       push  rcx
///
///       // Call our closure
///       lea   rdi,[rcx+0x104]
///       call  {target}
///
///       // Restore stack and important volatiles
///       pop   rcx
///
///       // Return to the hooked code
///       ret
///    ", |money_counter : & mut i32| {
///       println!("Pwned! We rich af boys!!!");
///
///       *money_counter = i32::MAX;
///
///       return;
///    }),
/// }
/// ```
///
/// <h6 id=  hook_examples_incorrect>
/// <a href=#hook_examples_incorrect>
/// Common <b>incorrect</b> usage
/// </a></h6>
///
/// ```
/// pub fn aimbot_enable(fov : f64) {
///    let hook_aimbot = nusion_lib::patch::writer::Hook{
///       memory_offset_range  : 0x80001347..0x8000139F,
///       checksum             : nusion_lib::patch::Checksum::from(0xBAADF00D),
///       hook                 : nusion_lib::hook!("
///          push  rax
///          call  {target}
///          pop   rax
///          ret
///       ", || {
///          // !!! This will fail to compile !!!
///          // Even though this looks like a closure,
///          // we are not allowed to access our environment
///          // in any way, even if we are allowed to take
///          // ownership of variables in our environment.
///          // For all intents and purposes, the closure
///          // is an anonymous function with vertical pipes
///          // instead of curly brackets.  The only variables
///          // we can access from a hook closure are its direct
///          // input arguments and global variables (const,
///          // static, static mut).
///          let fov_calculated = fov / 2.0 + 1.5;
///
///          return;
///       });
///    };
///
///    return;
/// }
/// ```
#[proc_macro]
#[proc_macro_error::proc_macro_error]
pub fn hook(
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   return fm_hook::hook(item);
}

/// Generates a static byte slice
/// containing assembly instructions.
/// The syntax is mostly the same
/// as <code><a href=
/// https://doc.rust-lang.org/stable/core/arch/macro.asm.html
/// >asm!</a></code>, but there
/// are no options nor template arguments.
/// 
/// <h2 id=  asm_bytes_note>
/// <a href=#asm_bytes_note>
/// Note
/// </a></h2>
///
/// Due to issues with constructing
/// the byte slice, the macro output
/// cannot be used to initialize a
/// const variable.  This can be
/// side-stepped by using the
/// <a href=https://docs.rs/lazy_static/1.4.0/lazy_static/
/// >lazy_static</a> crate to
/// initialize at runtime.
///
/// <h2 id=  asm_bytes_safety>
/// <a href=#asm_bytes_safety>
/// Safety
/// </a></h2>
///
/// The input assembly should not only
/// be valid for its intended use case,
/// but should also <b>never</b> use any
/// memory-relative offsets.  Since the raw
/// machine code is stored as a byte slice
/// and then copiped when applied through
/// a patch, memory-relative offsets will
/// remain the same.  <i>This will lead
/// to the formerly valid offsets pointing
/// to now unknown junk data</i>.  The only
/// memory-relative offsets which are valid
/// are ones relative to the instruction
/// pointer / program counter register and
/// are contained within the provided assembly.
/// Any references to code or data outside
/// the provided assembly will lead to undefined
/// behavior.
///
/// <h2 id=  asm_bytes_examples>
/// <a href=#asm_bytes_examples>
/// Examples
/// </a></h2>
///
/// <h6 id=  asm_bytes_examples_correct>
/// <a href=#asm_bytes_examples_correct>
/// Correct usages
/// </a></h6>
///
/// ```
/// let correct_usage_0 = nusion_lib::asm_bytes!("
///    xor   eax,eax  // We aren't accessing memory
///                   // in any way, so there's
///                   // absolutely nothing wrong
///                   // with simple instructions
///                   // like this
/// ");
///
/// let correct_usage_1 = nusion_lib::asm_bytes!("
///    lea   rax,[rdi+0xD0]    // This is allowable because
///    call  rax               // the call target is calculated
///    leave                   // from a pointer stored in a
///    ret                     // register, not a label
/// ");
///
/// let correct_usage_2 = nusion_lib::asm_bytes!("
///    internal_label:         // This is allowable because
///    sub   ebx,1             // the label is within our
///    jnz   internal_label    // ASM code and is code-relative
/// ");
/// ```
///
/// <h6 id=  asm_bytes_examples_incorrect>
/// <a href=#asm_bytes_examples_incorrect>
/// <b>Incorrect</b> usages
/// </a></h6>
///
/// ```
/// let incorrect_usage_0 = nusion_lib::asm_bytes!("
///    sub   ebx,1             // This is not allowed because
///    jnz   external_label    // we are jumping to some outside label
///                            // which will not be in the same
///                            // location if we copy the ASM
/// ");
///
/// let incorrect_usage_1 = nusion_lib::asm_bytes!("
///    call  jesus_take_the_wheel   // Calling to some external
///                                 // function by label is not
///                                 // allowed.  The relative offset
///                                 // will break when copied.
/// ");
/// ```
#[proc_macro]
#[proc_macro_error::proc_macro_error]
pub fn asm_bytes(
   item  : proc_macro::TokenStream,
) -> proc_macro::TokenStream {
   return fm_asm_bytes::asm_bytes(item);
}

