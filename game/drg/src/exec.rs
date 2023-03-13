//! Main loop which is executed
//! inside of the game's main
//! thread.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Stores information about the main
/// loop and executes the main loop.
pub struct MainLoop {
   input    : crate::input::InputState,
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
      println!("--- Welcome to Nusion for Deep Rock Galactic! ---");
      println!("");
      println!("This is in the VERY early stages of development,");
      println!("so everything will be very basic and possibly");
      println!("unstable.  Nevertheless, here are keybinds for");
      println!("all the currently available features:");
      println!("");
      println!("Exit and unload  - Delete");
      println!("");
      println!("-------------------------------------------------");
      println!("");

      return Self{
         input : crate::input::InputState::new(),
      };
   }

   // Executes one iteration of the main loop.
   // This should only ever run on the same
   // thread as the game.
   pub fn execute(
      & mut self,
   ) -> Result {
      // Poll input devices
      self.input.poll();

      // Exit if we are supposed to
      return Ok(self.input.key_press_exit == false);
   }
}

