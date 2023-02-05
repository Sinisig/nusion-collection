//! crate::os::console implementation for Windows.

use winapi::{
   shared::{
      minwindef::{
         FALSE,
      },
   },
   um::{
      consoleapi::{
         AllocConsole,
      },
      wincon::{
         FreeConsole,
         SetConsoleTitleA,
      },
      winnt::{
         CHAR,
      },
   },
};

#[derive(Debug)]
pub enum ConsoleError {
   Unknown,
}

pub struct Console {
}

impl std::fmt::Display for ConsoleError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::Unknown  => "Unknown",
      });
   }
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
      return Ok(String::from(""));
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
         panic!("Failed to free console");
      }

      return;
   }
}

