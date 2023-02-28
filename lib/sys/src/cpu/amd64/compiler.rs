//! crate::cpu::compiler implementation for AMD64.

use crate::compiler::CompilationError;

pub fn nop_fill(
   memory_region : & mut [u8],
) -> crate::compiler::Result<& mut [u8]> {
   let mut memory_view = & mut memory_region[..];

   'assemble_loop : loop {
      let instruction_length = match memory_view.len() {
         0  => break 'assemble_loop,
         1  => super::assembler::nop1 (memory_view)?,
         2  => super::assembler::nop2 (memory_view)?,
         3  => super::assembler::nop3 (memory_view)?,
         4  => super::assembler::nop4 (memory_view)?,
         5  => super::assembler::nop5 (memory_view)?,
         6  => super::assembler::nop6 (memory_view)?,
         7  => super::assembler::nop7 (memory_view)?,
         8  => super::assembler::nop8 (memory_view)?,
         9  => super::assembler::nop9 (memory_view)?,
         10 => super::assembler::nop10(memory_view)?,
         _  => super::assembler::nop11(memory_view)?,
      };

      memory_view = & mut memory_view[instruction_length..];
   }

   return Ok(memory_region);
}

pub unsafe fn hook_fill(
   memory_region  : & mut [u8],
   target_hook    : * const core::ffi::c_void,
) -> crate::compiler::Result<& mut [u8]> {
   let mut memory_view = & mut memory_region[..];

   // Calculate relative offset for call
   let target_hook_relative = target_hook.cast::<u8>().offset_from(
      memory_view.as_ptr(),
   );

   // Required instruction - Assemble smallest call possible
   let call_length =
   if let Ok(rel32) = i32::try_from(target_hook_relative) {
      super::assembler::call_rel32(memory_view, rel32)
   } else {
      super::assembler::call_abs64(memory_view, target_hook as u64)
   }.map_err(|_| CompilationError::ImpossibleEncoding)?;
   memory_view = & mut memory_view[call_length..]; 

   // Performance optimization - Don't compile a jmp if we have
   // a small amount of nop instructions
   const NOP_MAX_BYTES_WITHOUT_JMP : usize = 18;

   if memory_view.len() > NOP_MAX_BYTES_WITHOUT_JMP {
      // Get relative offset for jump past nop bytes
      let target_skip_relative = memory_view.len();

      // Optional instruction - Assemble smallet jump to end of block
      let jmp_length =
      if let Ok(rel8) = i8::try_from(target_skip_relative) {
         super::assembler::jmp_rel8(memory_view, rel8)
      } else if let Ok(rel32) = i32::try_from(target_skip_relative) {
         super::assembler::jmp_rel32(memory_view, rel32)
      } else {
         super::assembler::jmp_abs64(memory_view, memory_view.as_ptr_range().end as u64)
      }.unwrap_or(0);
      memory_view = & mut memory_view[jmp_length..];

      // Optional instruction - Assemble ud2 instruction right after jmp
      let ud2_length =
      super::assembler::ud2(memory_view).unwrap_or(0);
      memory_view = & mut memory_view[ud2_length..];
   }

   // Fill the rest of the memory view with nop instructions
   nop_fill(memory_view)?;

   // Return original memory region successfully
   return Ok(memory_region);
}

