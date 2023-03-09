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
   let _test = nusion::asm_bytes!("xor eax,eax");
   println!("--- ASM Bytes ---");
   for b in _test {
      println!("{b}");
   }
   println!("-----------------");

   println!("Executed loop!");
   return Ok(true);
}

