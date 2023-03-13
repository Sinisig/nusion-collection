//! Input management and handling
//! for toggling certain features
//! on or off.

//////////////////
// KEY BINDINGS //
//////////////////

pub mod bind {
   //! Key bindings used for each feature/action.

   use inputbot::KeybdKey::{self, *};

   pub const EXIT          : KeybdKey
      = DeleteKey;
   pub const FLIGHT        : KeybdKey
      = Numpad1Key;
   pub const INFINITE_AMMO : KeybdKey
      = Numpad2Key;
   pub const NO_FIRE_DELAY : KeybdKey
      = Numpad3Key;
}

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Stores the input state for each
/// feature or action.  This can be
/// updated by the <code>poll</code>
/// method.
pub struct InputState {
   pub key_press_exit            : bool,
   pub key_toggle_flight         : bool,
   pub key_toggle_infinite_ammo  : bool,
   pub key_toggle_no_fire_delay  : bool,
}

///////////////////////////
// METHODS - ActionState //
///////////////////////////

impl InputState {
   // Creates a new action state
   // struct with everything disabled.
   pub fn new(
   ) -> Self {
      return Self{
         key_press_exit             : false,
         key_toggle_flight          : false,
         key_toggle_infinite_ammo   : false,
         key_toggle_no_fire_delay   : false,
      };
   }

   // Polls input devices and updates
   // the action state accordingly.
   pub fn poll(
      & mut self,
   ) -> & mut Self {
      // Helper macros for updating input state
      // of a member variable and a key binding
      macro_rules! update_press {
         ($member_var:ident, $keybind:ident) => {
            self.$member_var = bind::$keybind.is_pressed();
         };
      }
      macro_rules! update_toggle {
         ($member_var:ident, $keybind:ident) => {
            self.$member_var = bind::$keybind.is_toggled();
         };
      }

      // Update every member variable's state
      update_press!  (key_press_exit,           EXIT);
      update_toggle! (key_toggle_flight,        FLIGHT);
      update_toggle! (key_toggle_infinite_ammo, INFINITE_AMMO);
      update_toggle! (key_toggle_no_fire_delay, NO_FIRE_DELAY);
      return self;
   }
}

