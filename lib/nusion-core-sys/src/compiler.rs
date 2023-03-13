//! Code compilation functions.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

#[derive(Debug)]
pub enum CompilationError {
   ImpossibleEncoding,
   BufferTooSmall{
      instruction_length   : usize,
      buffer_length        : usize,
   },
}

/// <code>Result</code> type with error
/// variant <code>CompilationError</code>.
pub type Result<T> = std::result::Result<T, CompilationError>;

/// Type which stores a pointer to a hook function
/// for use in <code>hook_fill</code>.
pub type HookTarget = unsafe extern "C" fn();

//////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - CompilationError //
//////////////////////////////////////////////

impl std::fmt::Display for CompilationError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return match self {
         Self::ImpossibleEncoding
            => write!(stream, "Impossible instruction encoding"),
         Self::BufferTooSmall {instruction_length, buffer_length}
            => write!(stream, "Buffer is too small for instruction encoding: Requires at least {instruction_length}, found {buffer_length}"),
      };
   }
}

impl std::error::Error for CompilationError {
}

///////////////
// FUNCTIONS //
///////////////

/// Fills the given memory buffer
/// with architecture-dependent
/// no-operation (NOP) instructions.
pub fn nop_fill(
   memory_buffer : & mut [u8],
) -> Result<()> {
   return crate::cpu::compiler::nop_fill(
      memory_buffer,
   );
}

/// Compiles a call to a function
/// inside a memory buffer.  The
/// rest of the buffer is filled
/// with architecture-dependent
/// no-operation (NOP) instructions.
///
/// <h2 id=  hook_fill_note>
/// <a href=#hook_fill_note>
/// Note
/// </a></h2>
/// The compiled code expects to
/// never be moved to a new memory
/// location.  Copying the memory
/// buffer slice to a new region
/// will lead to invalid code.
/// If you want to clone a compiled,
/// hook, it must be re-compiled
/// in the new memory buffer.
pub fn hook_fill(
   memory_buffer  : & mut [u8],
   hook           : HookTarget,
) -> Result<()> {
   return crate::cpu::compiler::hook_fill(
      memory_buffer, hook,
   );
}

