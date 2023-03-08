//! crate::cpu::compiler implementation for AMD64.

pub fn nop_fill(
   memory_buffer : & mut [u8],
) -> crate::compiler::Result<()> {
   let mut memory_buffer_view = & mut memory_buffer[..];

   'assemble_loop : loop {
      let instruction_length = match memory_buffer_view.len() {
         0  => break 'assemble_loop,
         1  => super::assembler::nop1 (memory_buffer_view)?,
         2  => super::assembler::nop2 (memory_buffer_view)?,
         3  => super::assembler::nop3 (memory_buffer_view)?,
         4  => super::assembler::nop4 (memory_buffer_view)?,
         5  => super::assembler::nop5 (memory_buffer_view)?,
         6  => super::assembler::nop6 (memory_buffer_view)?,
         7  => super::assembler::nop7 (memory_buffer_view)?,
         8  => super::assembler::nop8 (memory_buffer_view)?,
         9  => super::assembler::nop9 (memory_buffer_view)?,
         10 => super::assembler::nop10(memory_buffer_view)?,
         _  => super::assembler::nop11(memory_buffer_view)?,
      };

      memory_buffer_view = & mut memory_buffer_view[instruction_length..];
   }

   return Ok(());
}

pub fn hook_fill(
   memory_buffer  : & mut [u8],
   hook           : crate::compiler::HookTarget,
) -> crate::compiler::Result<()> {
   const NOP_BYTES_TO_COMPILE_JMP : usize
      = 22; // At most 2 consecutive 11-byte nops

   let mut memory_buffer_view = & mut memory_buffer[..];

   // Required instruction - Call to the hook
   let instruction_length = super::assembler::call(
      memory_buffer_view,
      hook as * const core::ffi::c_void,
   )?;
   memory_buffer_view = & mut memory_buffer_view[instruction_length..];

   // If the remaining bytes are small, don't
   // compile a jmp and ud2, this is a speed
   // optimization.  It also ensures the next
   // code should never return Err.
   if memory_buffer_view.len() <= NOP_BYTES_TO_COMPILE_JMP {
      nop_fill(memory_buffer_view)?;
      return Ok(());
   }
   
   // Compile a jump to the end of the
   // memory region
   let instruction_length = super::assembler::jmp(
      memory_buffer_view,
      memory_buffer_view.as_ptr_range().end as * const core::ffi::c_void,
   )?;
   memory_buffer_view = & mut memory_buffer_view[instruction_length..];

   // Compile a ud2 instruction after the
   // jmp in case something goes catastrophically
   // wrong and we fail to execute the jmp.
   let instruction_bytes = super::assembler::ud2(
      memory_buffer_view,
   )?;
   memory_buffer_view = & mut memory_buffer_view[instruction_bytes..];

   // Fill the rest of the memory
   // with nop instructions
   nop_fill(memory_buffer_view)?;

   // Successfully return
   return Ok(());
}

