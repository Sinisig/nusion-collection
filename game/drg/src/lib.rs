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

   let test_patcher = nusion::patch::method::Nop{
      memory_offset_range  : 0x14D7CF0..0x14D7CF6,
      checksum             : nusion::patch::Checksum::from(0xF0EF21E8),
   };

   use nusion::patch::Patch;
   let _test_nop = unsafe{game!()?.patch_create(&test_patcher)}?;

   std::thread::sleep(std::time::Duration::from_secs(30));
   return Ok(());
}

