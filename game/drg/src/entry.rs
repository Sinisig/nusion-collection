//! Entrypoint initialization and
//! begin of game hooking for the
//! main loop.

use nusion::patch::Patch;

// Globals for sending information to
// DLL thread from hook
lazy_static::lazy_static!{
static ref LOOP_EXECUTE
   : std::sync::Mutex<bool>
   = std::sync::Mutex::new(true);
}
lazy_static::lazy_static!{
static ref LOOP_ERROR
   : std::sync::Mutex<Option<Box<dyn std::error::Error + Send>>>
   = std::sync::Mutex::new(None);
}

// Hook information which also writes
// the returned info from the loop to
// our globals safely.
const HOOK_LOOP : nusion::patch::writer::Hook = nusion::patch::writer::Hook{
   memory_offset_range  : 0x8241BC..0x8241CD,
   checksum             : nusion::patch::Checksum::from(0xF7946268),
   target_hook          : nusion::hook!("
      // Store volatiles and align stack
      push  rax

      // Call the hook
      call  {target}

      // Restore stack and volatiles
      pop   rax

      // Execute stolen bytes and return
      mov   [rsp+0x60],rax
      mov   rcx,[rax]
      mov   [rsp+0x68],rcx
      mov   rcx,[rax+0x10]
      ret
      ", || {
         match crate::exec::exec_loop() {
            Ok(exec) => {
               *LOOP_EXECUTE.lock().unwrap() = exec;
            },
            Err(err) => {
               *LOOP_ERROR.lock().unwrap() = Some(err);
            },
         }
         return;
      }
   )
};

/// Main entrypoint, this is where
/// the fun stuff begins!
#[nusion::main("FSD-Win64-Shipping.exe")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
   // Initialization
   nusion::env_mut!().console_mut().set_title(
      "Nusion for Deep Rock Galactic by Sinsig",
   )?;

   // Patch the game's loop to run our loop
   // Cross your fingers we aren't currently executing this part of code!
   let _hook_loop = unsafe{crate::game_mut!().patch_create(&HOOK_LOOP)}?;

   // Loop until either the loop code return false or an error
   'main_loop : loop {
      if *LOOP_EXECUTE.lock().unwrap() == false {
         break 'main_loop;
      }

      if let Some(e) = LOOP_ERROR.lock().unwrap().take() {
         return Err(e);
      }
   }

   // Return success
   return Ok(());
}

