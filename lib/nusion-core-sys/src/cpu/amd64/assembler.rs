//! Internal instruction build_instruction_encodingr for AMD64.

//////////////////////
// INTERNAL HELPERS //
//////////////////////

fn build_instruction_encoding(
   memory_buffer  : & mut [u8],
   opcode         : & [u8],
   operand        : & [u8],
) -> crate::compiler::Result<usize> {
   let instruction_length = opcode.len() + operand.len();

   if memory_buffer.len() < instruction_length {
      return Err(crate::compiler::CompilationError::BufferTooSmall{
         instruction_length   : instruction_length,
         buffer_length        : memory_buffer.len(),
      });
   }

   let memory_buffer  = memory_buffer.iter_mut();
   let opcode  = opcode.iter();
   let operand = operand.iter();

   memory_buffer.zip(opcode.chain(operand)).for_each(
      |(dest, src)| {
      *dest = *src;
   });

   return Ok(instruction_length);
}

//////////////////////////
// INSTRUCTION BUILDERS //
//////////////////////////

pub fn nop1(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x90],
      &[],
   );
}

pub fn nop2(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x66, 0x90],
      &[],
   );
}

pub fn nop3(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x0F, 0x1F, 0x00],
      &[],
   );
}

pub fn nop4(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x0F, 0x1F, 0x40, 0x00],
      &[],
   );
}

pub fn nop5(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x0F, 0x1F, 0x44, 0x00, 0x00],
      &[],
   );
}

pub fn nop6(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x66, 0x0F, 0x1F, 0x44, 0x00, 0x00],
      &[],
   );
}

pub fn nop7(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x0F, 0x1F, 0x80, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

pub fn nop8(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

pub fn nop9(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

pub fn nop10(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x66, 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

pub fn nop11(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x66, 0x66, 0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

pub fn ud2(
   memory_buffer  : & mut [u8],
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0x0F, 0x0B],
      &[],
   );
}

pub fn jmp_rel8(
   memory_buffer  : & mut [u8],
   rel8           : i8,
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0xEB],
      &(rel8 - 2).to_le_bytes(),
   );
}

pub fn jmp_rel32(
   memory_buffer  : & mut [u8],
   rel32          : i32,
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0xE9],
      &(rel32 - 5).to_le_bytes(),
   );
}

pub fn jmp_abs64(
   memory_buffer  : & mut [u8],
   abs64          : u64,
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0xFF, 0x25, 0x00, 0x00, 0x00, 0x00],
      &abs64.to_le_bytes(),
   );
}

pub fn jmp(
   memory_buffer  : & mut [u8],
   target         : * const core::ffi::c_void,
) -> crate::compiler::Result<usize> {
   let target  = target as * const u8;
   let current = memory_buffer.as_ptr();

   let offset = unsafe{target.offset_from(current)};

   if let Ok(offset) = i8  ::try_from(offset) {
      return jmp_rel8   (memory_buffer, offset);
   }
   if let Ok(offset) = i32 ::try_from(offset) {
      return jmp_rel32  (memory_buffer, offset);
   }

   return jmp_abs64(memory_buffer, target as u64);
}

pub fn call_rel32(
   memory_buffer  : & mut [u8],
   rel32          : i32,
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0xE8],
      &(rel32 - 5).to_le_bytes(),
   );
}

pub fn call_abs64(
   memory_buffer  : & mut [u8],
   abs64          : u64,
) -> crate::compiler::Result<usize> {
   return build_instruction_encoding(
      memory_buffer,
      &[0xFF, 0x15, 0x02, 0x00, 0x00, 0x00, 0xEB, 0x08],
      &abs64.to_le_bytes(),
   );
}

pub fn call(
   memory_buffer  : & mut [u8],
   target         : * const core::ffi::c_void,
) -> crate::compiler::Result<usize> {
   let target  = target as * const u8;
   let current = memory_buffer.as_ptr();

   let offset = unsafe{target.offset_from(current)};

   if let Ok(offset) = i32 ::try_from(offset) {
      return call_rel32 (memory_buffer, offset);
   }

   return call_abs64(memory_buffer, target as u64);
}

