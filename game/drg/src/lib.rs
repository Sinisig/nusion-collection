macro_rules! game {
   () => {
      nusion::env_mut!()
         .modules_mut()
         .find_mut_by_executable_file_name("FSD-Win64-Shipping.exe")
         .ok_or(MainError::WrongProcess)
   }
}

#[derive(Debug)]
enum MainError {
   WrongProcess,
}

impl std::fmt::Display for MainError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::WrongProcess
            => "Process is not Deep Rock Galactic",
      });
   }
}

impl std::error::Error for MainError {
}

#[nusion::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
   nusion::env_mut!().console_mut().set_title(
      "Nusion for Deep Rock Galactic by Sinisig",
   )?;

   println!("Hello from the other side!");

   println!(
      "Start address: {addr:#0fill$x}",
      addr = game!()?.address_range().start,
      fill = std::mem::size_of::<usize>() * 2 + 2,
   );
   println!(
      "End address:   {addr:#0fill$x}",
      addr = game!()?.address_range().end,
      fill = std::mem::size_of::<usize>() * 2 + 2,
   );

   const TEST_PATCHER : nusion::patch::method::Asm = nusion::patch::method::Asm{
      memory_offset_range  : 0x14D7CDB..0x14D7CF9,
      checksum             : nusion::patch::Checksum::from(0x8CC8AE3B),
      alignment            : nusion::patch::Alignment::Center,
      asm_bytes            : &[] //nusion::asm_bytes!("xor eax,eax"),
   };

   use nusion::patch::Patch;
   let _test_hook_ammo = unsafe{game!()?.patch_create(&TEST_PATCHER)}?;

   std::thread::sleep(std::time::Duration::from_secs(30));
   return Ok(());
}

