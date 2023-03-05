//! Code compilation functions.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

#[derive(Debug)]
pub enum CompilationError {
   ImpossibleEncoding,
   BufferTooSmall{
      inst_len : usize,
      buff_len : usize,
   },
}

pub type Result<T> = std::result::Result<T, CompilationError>;

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
         Self::BufferTooSmall {inst_len, buff_len}
            => write!(stream, "Buffer is too small for instruction encoding: Requires at least {inst_len}, found {buff_len}"),
      };
   }
}

impl std::error::Error for CompilationError {
}

///////////////
// FUNCTIONS //
///////////////

/// Fills the given slice with
/// no-operation instructions.
pub fn nop_fill(
   memory_region  : & mut [u8],
) -> Result<& mut [u8]> {
   return crate::cpu::compiler::nop_fill(memory_region);
}

/// Builds a function hook within
/// the given slice and fills the
/// remaining space with no-operation
/// instructions.
///
/// <h2 id=  hook_fill_safety>
/// <a href=#hook_fill_safety>
/// Safety
/// </a></h2>
///
/// It is assumed the slice will
/// never be copied or moved.  This
/// is because relative memory offsets
/// are used when assembling the call
/// instruction.  The compiled code
/// is only valid for the input slice
/// and not any copies of it.
pub unsafe fn hook_fill(
   memory_region  : & mut [u8],
   target_hook    : unsafe extern "C" fn(),
) -> Result<& mut [u8]> {
   return crate::cpu::compiler::hook_fill(memory_region, target_hook);
}

