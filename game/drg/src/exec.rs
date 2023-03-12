//! Main loop which is executed
//! inside of the game's main
//! thread.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Stores information about the main
/// loop and executes the main loop.
pub struct MainLoop {
   test_counter : u32,
}

/// <code>Result</code> type returned by
/// <code>MainLoop::execute</code> where
/// the <code>Ok</code> variant signifies
/// whether to keep executing or not and
/// the <code>Err</code> variant implies
/// to stop looping and return the error
/// code.
pub type Result = std::result::Result<bool, Box<dyn std::error::Error + Send>>;

////////////////////////
// METHODS - MainLoop //
////////////////////////

impl MainLoop {
   // Initializes a new main loop.
   pub fn init(
   ) -> Self {
      return Self{
         test_counter   : 0,
      };
   }

   // Executes one iteration of the main loop.
   // This should only ever run on the same
   // thread as the game.
   pub fn execute(
      & mut self,
   ) -> Result {
      println!("Main loop test counter: {}", self.test_counter);

      self.test_counter += 1;

      return Ok(self.test_counter <= 500);
   }
}

