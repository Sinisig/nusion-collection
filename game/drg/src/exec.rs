//! Main loop which is executed
//! inside of the game's main
//! thread.

/// Stores various information about the
/// state of the mod in the loop.
pub struct LoopState {
   pub test_counter : u32,
}

impl LoopState {
   pub const fn new() -> Self {
      return Self{
         test_counter   : 0,
      };
   }
}

/// Ok variant of the return type
/// signifies whether the loop should
/// keep executing or not.  The Err
/// variant will return the error to
/// the caller and implicitly stop
/// looping.
pub fn main_loop(
   state : & mut LoopState,
) -> Result<bool, Box<dyn std::error::Error + Send>> {
   state.test_counter += 1;
   println!("Executed loop! {}", state.test_counter);

   return Ok(state.test_counter < 500);
}

