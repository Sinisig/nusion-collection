#[nusion::entry]
pub fn entry() -> Result<(), Box<dyn std::error::Error>> {
   const GAME_PROCESS_NAME : &str = "FSD-Win64-Shipping.exe";

   let mut env = nusion::Environment::try_get_mut()?;
   env.console_mut().set_title(
      "Nusion for Deep Rock Galactic by Sinisig",
   )?;

   let game_module = match env.modules().find_by_executable_file_name(
      GAME_PROCESS_NAME,
   ) {
      Some(module)   => module,
      None           => return Err(Box::new(MainError::WrongProcess)),
   };

   println!("Hello from the other side!");

   println!(
      "Start address: {addr:#0fill$x}",
      addr = game_module.address_range().start,
      fill = std::mem::size_of::<usize>() * 2 + 2,
   );
   println!(
      "End address:   {addr:#0fill$x}",
      addr = game_module.address_range().end,
      fill = std::mem::size_of::<usize>() * 2 + 2,
   );
   
   std::thread::sleep(std::time::Duration::from_secs(5));
   return Ok(());
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

