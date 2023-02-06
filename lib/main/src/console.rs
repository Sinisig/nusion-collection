//! Console window creation and management.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error relating to a Console
/// instance.
#[derive(Debug)]
pub struct ConsoleError {
   kind  : ConsoleErrorKind,
}

/// An error kind enum for ConsoleError.
#[derive(Debug)]
pub enum ConsoleErrorKind {
   Unknown,
}

/// A Result type with Err variant ConsoleError.
pub type Result<T> = std::result::Result<T, ConsoleError>;

/// A console window for printing
/// text to using the standard
/// print macros.
pub struct Console {
   console  : crate::sys::console::Console,
}

////////////////////////////
// METHODS - ConsoleError //
////////////////////////////

impl ConsoleError {
   /// Creates a new ConsoleError with
   /// the given error kind.
   pub fn new(
      kind : ConsoleErrorKind,
   ) -> Self {
      return Self{
         kind : kind,
      };
   }

   /// Gets the ConsoleErrorKind stored
   /// in the ConsoleError.
   pub fn kind<'l>(
      &'l self,
   ) -> &'l ConsoleErrorKind {
      return &self.kind;
   }
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ConsoleError //
//////////////////////////////////////////

impl std::fmt::Display for ConsoleError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", self.kind());
   }
}

impl std::error::Error for ConsoleError {
}

impl From<crate::sys::console::ConsoleError> for ConsoleError {
   fn from(
      item : crate::sys::console::ConsoleError,
   ) -> Self {
      use crate::sys::console::ConsoleErrorKind::*;

      return Self::new(match item.kind() {
         Unknown  => ConsoleErrorKind::Unknown,
      });
   }
}

//////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ConsoleErrorKind //
//////////////////////////////////////////////

impl std::fmt::Display for ConsoleErrorKind {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::Unknown
            => "Unknown",
      });
   }
}

///////////////////////
// METHODS - Console //
///////////////////////

impl Console {
   /// Creates a new console window.
   /// If creation fails, an error is
   /// returned.
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

   /// Sets the title of the Console.
   pub fn set_title(
      & mut self,
      title : & str,
   ) -> Result<& Self> {
      self.console.set_title(title)?;
      return Ok(self);
   }
}

