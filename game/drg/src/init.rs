//! Entrypoint initialization and
//! begin of game hooking for the
//! main loop.

use nusion_lib::patch::Patch;

/// Struct which stores the last return
/// code from the main loop in a more
/// friendly way for synchronization.
struct LoopStatus{
   should_execute : bool,
   err_code       : Option<Box<dyn std::error::Error + Send>>,
}

// Stores the loop state and is
// passed by reference on each
// loop execution.  This is only
// actually used inside the loop,
// so no need to synchronize.
static mut LOOP_STATE
   : crate::exec::LoopState
   = crate::exec::LoopState::new();

// Stores the returned code from the
// main loop.  This DOES require
// synchronization.  This is also used
// as a mutex lock for knowing if we
// should execute the loop or not.
lazy_static::lazy_static!{
static ref LOOP_RETURN_CODE
   : std::sync::Mutex<LoopStatus>
   = std::sync::Mutex::new(LoopStatus{
      should_execute : true,
      err_code       : None,
   });
}

// Hook which executes the main loop.
// We have to be careful to minimize
// the chance of a race condition since
// this is applied asynchronously.
const HOOK_LOOP : nusion_lib::patch::writer::Hook<std::ops::Range<usize>> = nusion_lib::patch::writer::Hook{
   memory_offset_range  : 0x8241BC..0x8241CD,
   checksum             : nusion_lib::patch::Checksum::from(0xF7946268),
   hook                 : nusion_lib::hook!("
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
         // Don't block while waiting for the
         // lock, this increases the chance of
         // a race condition
         let mut lock = match LOOP_RETURN_CODE.try_lock() {
            Ok(lock) => lock,
            Err(_)   => return,
         };

         // If we aren't supposed to execute,
         // return early
         if lock.should_execute == false {
            return;
         }

         // Execute the main loop, unwraping the
         // return code for the loop status struct
         let should_execute   : bool;
         let err_code         : Option<Box<dyn std::error::Error + Send>>;
         match crate::exec::main_loop(unsafe{& mut LOOP_STATE}) {
            Ok(state) => {
               should_execute = state;
               err_code       = None;
            },
            Err(err) => {
               should_execute = false;
               err_code       = Some(err);
            },
         }
         
         // Store the unwrapped error code
         // in the mutex
         *lock = LoopStatus{
            should_execute : should_execute,
            err_code       : err_code,
         };

         return;
      }
   )
};

/// Nusion entrypoint.  Keep in mind this executes
/// on a separate thread to the main game thread, so
/// we need synchronization in order to safely read
/// and write between the main loop hook and this.
#[nusion_lib::main("FSD-Win64-Shipping.exe")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
   // Initialization
   nusion_lib::env_mut!().console_mut().set_title(
      "Nusion for Deep Rock Galactic by Sinsig",
   )?;

   // Hooks the game's main loop to execute our
   // main loop.  This currently has a race condition
   // because we might be executing this bit of code
   // while writing, but don't worry about it!
   let hook_loop = unsafe{crate::game_mut!().patch_create(&HOOK_LOOP)}?;

   // Wait for us to either receive an Ok(false) or Err(_)
   // from the main loop
   let loop_code : Result<(), Box<dyn std::error::Error>>;
   'main_loop : loop {
      // We don't need to check constantly
      std::thread::sleep(std::time::Duration::from_secs(1));

      // We don't want to block the thread waiting for the lock
      let mut lock = match LOOP_RETURN_CODE.try_lock() {
         Ok(lock) => lock,
         Err(_)   => continue,
      };

      // Unpack loop state
      let should_execute   = lock.should_execute;
      let err_code         = lock.err_code.take();
      
      // Act upon the state
      if let Some(err_code) = err_code {
         loop_code = Err(err_code);
         break 'main_loop;
      }

      if should_execute == false {
         loop_code = Ok(());
         break 'main_loop;
      }
   }

   // Explicitly drop the hook container to make
   // obvious where it is restored
   std::mem::drop(hook_loop);

   // Return the loop's code
   return loop_code;
}

