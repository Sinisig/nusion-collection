//! Crate root for a game hack for
//! Deep Rock Galactic written with
//! the nusion crate.

///////////////////
/// MODULE PATH ///
///////////////////

mod exec;
mod init;
mod input;

////////////////////////////////
/// GENERALLY USED UTILITIES ///
////////////////////////////////

/// Macro for obtaining the game's
/// module.
#[macro_export]
macro_rules! game {
   () => {
      nusion_core::env!()
         .modules()
         .find_by_executable_file_name("FSD-Win64-Shipping.exe")
         .expect("Failed to find game module")
   }
}

/// Macro for obtaining the game's
/// module mutably.
#[macro_export]
macro_rules! game_mut {
   () => {
      nusion_core::env_mut!()
         .modules_mut()
         .find_mut_by_executable_file_name("FSD-Win64-Shipping.exe")
         .expect("Failed to find game module")
   }
}

