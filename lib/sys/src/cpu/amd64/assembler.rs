//! Internal instruction assembler for AMD64.

use crate::compiler::{
   CompilationError,
   Result,
};

//////////////////////
// INTERNAL HELPERS //
//////////////////////

fn assemble(
   buffer   : & mut [u8],
   opcode   : & [u8],
   operand  : & [u8],
) -> Result<usize> {
   let instruction_length = opcode.len() + operand.len();

   if instruction_length > buffer.len() {
      return Err(CompilationError::ResidualBytes{
         bytes : buffer.len() - instruction_length,
      });
   }

   let buffer  = buffer.iter_mut();
   let opcode  = opcode.iter();
   let operand = operand.iter();

   buffer.zip(opcode.chain(operand)).for_each(
      |(dest, src)| {
      *dest = *src;
   });

   return Ok(instruction_length);
}

//////////////////////////
// INSTRUCTION BUILDERS //
//////////////////////////

pub fn nop1(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x90],
      &[],
   );
}

pub fn nop2(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x66, 0x90],
      &[],
   );
}

pub fn nop3(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x0F, 0x1F, 0x00],
      &[],
   );
}

pub fn nop4(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x0F, 0x1F, 0x40, 0x00],
      &[],
   );
}

pub fn nop5(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x0F, 0x1F, 0x44, 0x00, 0x00],
      &[],
   );
}

pub fn nop6(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x66, 0x0F, 0x1F, 0x44, 0x00, 0x00],
      &[],
   );
}

pub fn nop7(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x0F, 0x1F, 0x80, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

pub fn nop8(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

pub fn nop9(
   buffer   : & mut [u8],
) -> Result<usize> {
   return assemble(
      buffer,
      &[0x66, 0x0F, 0x1F, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00],
      &[],
   );
}

