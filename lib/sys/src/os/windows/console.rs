//! crate::os::console implementation for Windows.

use crate::console::{ConsoleError};
use winapi::{
   shared::{
      minwindef::{
         DWORD,
         FALSE,
         MAX_PATH,
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
         CHAR,
         LPSTR,
      },
   },
};

pub struct Console {
}

impl Console {
   pub fn new() -> Result<Self, ConsoleError> {
      if unsafe{AllocConsole()} == FALSE {
         return Err(ConsoleError::Unknown);
      }

      let mut con = Self{};
      con.set_title("Nusion Console")?;

      return Ok(con);
   }

   pub fn get_title(
      & self,
   ) -> Result<String, ConsoleError> {
      // MAX_PATH + 1 to read the console title
      // plus a null terminator and another + 1
      // to check for errors
      const READ_BUFFER_LENGTH : usize = MAX_PATH + 2;

      let mut read_buffer = Vec::<u8>::with_capacity(READ_BUFFER_LENGTH);
      unsafe{read_buffer.set_len(READ_BUFFER_LENGTH)};

      let character_count = unsafe{GetConsoleTitleA(
         read_buffer.as_mut_ptr() as LPSTR,
         READ_BUFFER_LENGTH as DWORD,
      )};

      if character_count == 0 {
         // TODO: Propagate error message
         return Err(ConsoleError::Unknown);
      }

      read_buffer.truncate(character_count as usize);

      let read_buffer = match String::from_utf8(read_buffer) {
         Ok(s)    => s,
         Err(_)   => return Err(ConsoleError::InvalidTitleCharacters),
      };

      return Ok(read_buffer);
   }

   pub fn set_title(
      & mut self,
      title : & str,
   ) -> Result<& mut Self, ConsoleError> {
      if title.is_empty() {
         return Ok(self);
      }

      // null-terminated C-string
      let mut title  = String::from(title);
      let title      = unsafe{title.as_mut_vec()};
      title.push(0);

      if unsafe{SetConsoleTitleA(title.as_ptr() as * const CHAR)} == FALSE {
         return Err(ConsoleError::Unknown);
      }

      return Ok(self);
   }
}

impl Drop for Console {
   fn drop(
      & mut self,
   ) {
      if unsafe{FreeConsole()} == FALSE {
         panic!("Failed to free the console");
      }

      return;
   }
}

