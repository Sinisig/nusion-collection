macro_rules! game {
   () => {
      nusion::env_mut!()
         .modules_mut()
         .find_mut_by_executable_file_name("FSD-Win64-Shipping.exe")
         .unwrap()   // Should always be present because of main attribute
   }
}

#[nusion::main("FSD-Win64-Shipping.exe")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
   nusion::env_mut!().console_mut().set_title(
      "Nusion for Deep Rock Galactic by Sinisig",
   )?;

   println!("Hello from the other side!");

   println!(
      "Start address: {addr:#0fill$x}",
      addr = game!().address_range().start,
      fill = std::mem::size_of::<usize>() * 2 + 2,
   );
   println!(
      "End address:   {addr:#0fill$x}",
      addr = game!().address_range().end,
      fill = std::mem::size_of::<usize>() * 2 + 2,
   );
 
   std::thread::sleep(std::time::Duration::from_secs(30));
   return Ok(());
}

