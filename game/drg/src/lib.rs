#[nusion::entry]
pub fn entry() -> Result<(), Box<dyn std::error::Error>> {
   nusion::env::environment_mut().console_mut().set_title(
      "Nusion for Deep Rock Galactic by Sinisig",
   )?;

   println!("Hello from the other side!");
   std::thread::sleep(std::time::Duration::from_secs(5));

   return Ok(());
}

