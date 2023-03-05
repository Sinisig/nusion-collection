//! crate::cpu::compiler implementation for AMD64.

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
   target_hook    : unsafe extern "C" fn(),
) -> crate::compiler::Result<& mut [u8]> {
   const NOP_BYTES_TO_COMPILE_JMP : usize
      = 22; // At most 2 consecutive 11-byte nops

   let mut memory_view = & mut memory_region[..];

   // Required instruction - Call to the hook
   let bytes = super::assembler::call(
      memory_view,
      target_hook as * const core::ffi::c_void,
   )?;
   memory_view = & mut memory_view[bytes..];

   // If the remaining bytes are small, don't
   // compile a jmp and ud2, this is a speed
   // optimization.  It also ensures the next
   // code should never return Err.
   if memory_view.len() <= NOP_BYTES_TO_COMPILE_JMP {
      nop_fill(memory_view)?;
      return Ok(memory_region);
   }
   
   // Compile a jump to the end of the
   // memory region
   let bytes = super::assembler::jmp(
      memory_view,
      memory_view.as_ptr_range().end.cast(),
   )?;
   memory_view = & mut memory_view[bytes..];

   // Compile a ud2 instruction after the
   // jmp in case something goes catastrophically
   // wrong and we fail to execute the jmp.
   let bytes = super::assembler::ud2(
      memory_view,
   )?;
   memory_view = & mut memory_view[bytes..];

   // Fill the rest of the memory
   // with nop instructions
   nop_fill(memory_view)?;

   // Successfully return
   return Ok(memory_region);
}

