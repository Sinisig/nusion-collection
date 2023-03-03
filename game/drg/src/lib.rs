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

#[nusion::entry]
pub fn entry() -> Result<(), Box<dyn std::error::Error>> {
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

   // Test function hook which intercepts
   // the ammo decrement for most weapons
   use nusion::patch::Patch;
   let _ammo_hook = unsafe{game!()?.patch_create_hook(
      0x14D7FAB..0x14D7FC6,
      nusion::patch::Checksum::from(0xF2185EA3),
      TEST_HOOK_AMMO_TRAMPOLINE,
   )}?;

   std::thread::sleep(std::time::Duration::from_secs(30));
   return Ok(());
}

#[nusion::hook(
   "
   // Stolen bytes
   sub      eax,[rcx+0x630]
   xor      ebp,ebp
   test     eax,eax
   mov      [rsp+0xC0],r12 // Add +0x08 to account for call
   cmovle   eax,ebp
   mov      [rcx+0x648],eax

   // Preserve volatiles and align stack
   push     rcx

   // Call HLL hook
   lea      rcx,[rcx+0x648]
   call     {hook}

   // Restore volatiles and stack
   pop      rcx

   // Return, woot woot
   ret
   ",
   TEST_HOOK_AMMO_TRAMPOLINE,
)]
fn test_hook_ammo(ammo : & mut i32) {
   *ammo += 2; // Account for decrement and add one to increment
   
   println!("Received ammo hook! New value: {ammo}");
   return;
}

