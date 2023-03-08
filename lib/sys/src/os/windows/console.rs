//! crate::os::console implementation for Windows.

use winapi::{
   shared::{
      minwindef::{
         DWORD,
         FALSE,
      },
   },
   um::{
      consoleapi::{
         AllocConsole,
      },
      wincon::{
         FreeConsole,
         GetConsoleTitleA,
         SetConsoleTitleA,
      },
      winnt::{
         LPSTR,
         LPCSTR,
      },
   },
};

// Maximum allowable title length when
// set with SetConsoleTitleA.
const MAX_TITLE_LENGTH : DWORD = 65535;

pub struct Console {
}

impl Console {
   pub fn allocate(
   ) -> crate::console::Result<Self> {
      if unsafe{AllocConsole()} == FALSE {
         return Err(crate::console::ConsoleError::Unknown);
      }

      return Ok(Self{});
   }

   pub fn free(
      & mut self,
   ) -> crate::console::Result<()> {
      if unsafe{FreeConsole()} == FALSE {
         return Err(crate::console::ConsoleError::Unknown);
      }

      return Ok(());
   }

   pub fn get_title(
      & self,
   ) -> crate::console::Result<String> {
      // Max title length + 1 for a null
      // terminator
      const READ_BUFFER_LENGTH : DWORD
         = MAX_TITLE_LENGTH + 1;

      // Create our read buffer allocated on
      // the heap because it's 64 kilobytes
      let mut read_buffer = Vec::<u8>::with_capacity(READ_BUFFER_LENGTH as usize);
      unsafe{read_buffer.set_len(READ_BUFFER_LENGTH as usize)};

      let character_count = unsafe{GetConsoleTitleA(
         read_buffer.as_mut_ptr() as LPSTR,
         READ_BUFFER_LENGTH,
      )};

      if character_count == 0 {
         return Err(crate::console::ConsoleError::Unknown);
      }

      read_buffer.truncate(character_count as usize);

      let read_buffer = match String::from_utf8(read_buffer) {
         Ok(s)    => s,
         Err(_)   => return Err(
            crate::console::ConsoleError::InvalidTitleCharacters,
         ),
      };

      return Ok(read_buffer);
   }

   pub fn set_title(
      & mut self,
      new_title : & str,
   ) -> crate::console::Result<()> {
      // null-terminated C-string
      let mut title  = String::from(new_title);
      let title      = unsafe{title.as_mut_vec()};
      title.push(0);

      if title.len() > MAX_TITLE_LENGTH as usize {
         return Err(crate::console::ConsoleError::Unknown);
      }

      if unsafe{SetConsoleTitleA(title.as_ptr() as LPCSTR)} == FALSE {
         return Err(crate::console::ConsoleError::Unknown);
      }

      return Ok(());
   }
}

