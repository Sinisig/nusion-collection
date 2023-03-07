//! Main loop which is executed
//! inside of the game's main
//! thread.

/// Ok variant of the return type
/// signifies whether the loop should
/// keep executing or not.  The Err
/// variant will return the error to
/// the caller and implicitly stop
/// looping.
pub fn exec_loop(
) -> Result<bool, Box<dyn std::error::Error + Send>> {
   println!("Executed loop!");
   return Ok(true);
}

