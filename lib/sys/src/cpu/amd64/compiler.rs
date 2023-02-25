//! crate::cpu::compiler implementation for AMD64.

pub fn nop_fill(
   memory_region : & mut [u8],
) -> crate::compiler::Result<& mut [u8]> {
   let mut memory_view = & mut memory_region[..];

   'assemble_loop : loop {
      let instruction_length = match memory_view.len() {
         0  => break 'assemble_loop,
         1  => super::assembler::nop1(memory_view)?,
         2  => super::assembler::nop2(memory_view)?,
         3  => super::assembler::nop3(memory_view)?,
         4  => super::assembler::nop4(memory_view)?,
         5  => super::assembler::nop5(memory_view)?,
         6  => super::assembler::nop6(memory_view)?,
         7  => super::assembler::nop7(memory_view)?,
         8  => super::assembler::nop8(memory_view)?,
         _  => super::assembler::nop9(memory_view)?,
      };

      memory_view = & mut memory_view[instruction_length..];
   }

   return Ok(memory_region);
}

pub unsafe fn hook_fill(
   memory_region  : & mut [u8],
   target_hook    : * const core::ffi::c_void,
) -> crate::compiler::Result<& mut [u8]> {
   #![allow(unused)]
   todo!()
}

