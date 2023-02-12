#[nusion::entry]
pub fn entry() -> Result<(), Box<dyn std::error::Error>> {
   const GAME_PROCESS_NAME : &str = "FSD-Win64-Shipping.exe";

   let mut env = nusion::Environment::try_get_mut()?;

   if env.process().executable_file_name() != GAME_PROCESS_NAME {
      return Err(Box::new(MainError::WrongProcess));
   }

   env.console_mut().set_title(
      "Nusion for Deep Rock Galactic by Sinisig",
   )?;

   println!("Hello from the other side!");
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

