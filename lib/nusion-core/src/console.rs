//! Console window creation and management.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error relating to a given
/// console instance.
#[derive(Debug)]
pub enum ConsoleError {
   InvalidTitleCharacters,
   Unknown,
}

/// <code>Result</code> type with error
/// variant <code>ConsoleError</code>.
pub type Result<T> = std::result::Result<T, ConsoleError>;

/// A console window for displaying
/// standard I/O streams.
pub struct Console {
   console  : crate::sys::console::Console,
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ConsoleError //
//////////////////////////////////////////

impl std::fmt::Display for ConsoleError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::InvalidTitleCharacters
            => "Title contains invalid characters",
         Self::Unknown
            => "Unknown",
      });
   }
}

impl std::error::Error for ConsoleError {
}

impl From<crate::sys::console::ConsoleError> for ConsoleError {
   fn from(
      item : crate::sys::console::ConsoleError,
   ) -> Self {
      use crate::sys::console::ConsoleError::*;
      return match item {
         InvalidTitleCharacters
            => Self::InvalidTitleCharacters,
         Unknown
            => Self::Unknown,
      }
   }
}

///////////////////////
// METHODS - Console //
///////////////////////

impl Console {
   /// Creates a new console.
   pub fn new() -> Result<Self> {
      return Ok(Self{
         console : crate::sys::console::Console::new()?,
      });
   }

   /// Gets an owned string copy of
   /// the title of the Console.
   pub fn get_title(
      & self,
   ) -> Result<String> {
      return Ok(self.console.get_title()?);
   }

   /// Sets the window title of
   /// the console.
   pub fn set_title(
      & mut self,
      title : & str,
   ) -> Result<& Self> {
      self.console.set_title(title)?;
      return Ok(self);
   }
}

