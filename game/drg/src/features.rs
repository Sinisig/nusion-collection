//! Feature management module.

use nusion_lib::patch::Patch;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Applies and restores features.
pub struct FeatureState {
   flight         : Option<Vec<FeatureContainer>>,
   infinite_ammo  : Option<Vec<FeatureContainer>>,
   no_fire_delay  : Option<Vec<FeatureContainer>>,
}

/// Internal type, represents the state of a
/// mod, storing the overwritten bytes
type FeatureContainer = nusion_lib::process::ModuleSnapshotPatchContainer;

////////////////////////////
// METHODS - FeatureState //
////////////////////////////

impl FeatureState {
   /// Creates an empty feature state.
   pub fn new(
   ) -> Self {
      return Self{
         flight         : None,
         infinite_ammo  : None,
         no_fire_delay  : None,
      };
   }

   /// Updates the feature state according
   /// to the given input state.
   pub fn update(
      & mut self,
      input : & crate::input::InputState,
   ) -> nusion_lib::patch::Result<& mut Self> {
      // Helper macros to reduce on typing
      macro_rules! update_feature {
         ($feature:ident, $input:ident, $create:ident, $as_str:literal) => {
            // Check if the desired and actual state differ
            if self.$feature.is_some() != input.$input {
               // Create the feature patch
               if input.$input == true {
                  self.$feature = Some($create()?);
                  println!("Enabled feature {}", $as_str);
               }

               // Drop the feature patch and restore
               if input.$input == false {
                  std::mem::drop(self.$feature.take());
                  println!("Disabled feature {}", $as_str);
               }
            }
         };
      }

      update_feature!(
         flight,
         key_toggle_flight,
         feature_flight,
         "Flight"
      );
      update_feature!(
         infinite_ammo,
         key_toggle_infinite_ammo,
         feature_infinite_ammo,
         "Infinite ammo"
      );
      update_feature!(
         no_fire_delay,
         key_toggle_no_fire_delay,
         feature_no_fire_delay,
         "No fire delay"
      );
      return Ok(self);
   }
}

//////////////
// FEATURES //
//////////////

fn feature_flight(
) -> nusion_lib::patch::Result<Vec<FeatureContainer>> {
   todo!()
}

fn feature_infinite_ammo(
) -> nusion_lib::patch::Result<Vec<FeatureContainer>> {
   let mut container = Vec::with_capacity(1);
  
   // General weapon ammo shoot
   container.push(unsafe{crate::game_mut!().patch_create(&nusion_lib::patch::writer::Asm{
      memory_offset_range  : 0x14D7CDB..0x14D7CF6,
      checksum             : nusion_lib::patch::Checksum::from(0xF2185EA3),
      alignment            : nusion_lib::patch::Alignment::Left,
      asm_bytes            : nusion_lib::asm_bytes!("
         // Overwritten instruction, keep this
         mov   qword ptr [rsp+0xB8],r12

         // Writes the constant 99 to the ammo count
         mov   dword ptr [rcx+0x648],99
      "),
   })}?);

   return Ok(container);
}

fn feature_no_fire_delay(
) -> nusion_lib::patch::Result<Vec<FeatureContainer>> {
   let mut container = Vec::with_capacity(1);
  
   // General weapon fire cooldown
   container.push(unsafe{crate::game_mut!().patch_create(&nusion_lib::patch::writer::Asm{
      memory_offset_range  : 0x14D7D02..0x14D7D1F,
      checksum             : nusion_lib::patch::Checksum::from(0xA96DA467),
      alignment            : nusion_lib::patch::Alignment::Left,
      asm_bytes            : nusion_lib::asm_bytes!("
         // Overwritten instructions, keep these
         xor   r12d,r12d   // TECHNICALLY should be the lowest 8 bits, but whatever
         mov   byte ptr [r14+0x6C2],02

         // Zero out fire cooldown
         xorps xmm0,xmm0 
      "),
   })}?);

   return Ok(container);
}

