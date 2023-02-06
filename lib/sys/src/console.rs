//! Console management module.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Contains error information relating
/// to the console.
#[derive(Debug)]
pub struct ConsoleError {
   kind : ConsoleErrorKind,
}

/// Contains the kind of error for
/// the console.
#[derive(Debug)]
pub enum ConsoleErrorKind {
   Unknown,
}

/// Result type with Ok variant T and Err variant ConsoleError.
pub type Result<T> = std::result::Result<T, ConsoleError>;

/// Stores a handle to the console.
pub struct Console(crate::os::console::Console);

////////////////////////////
// METHODS - ConsoleError //
////////////////////////////

impl ConsoleError {
   /// Creates a new ConsoleError from
   /// a ConsoleErrorKind.
   pub fn new(
      kind : ConsoleErrorKind,
   ) -> Self {
      return Self{
         kind : kind
      };
   }

   /// Gets a reference to the stored
   /// error kind.
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

impl From<crate::os::console::ConsoleError> for ConsoleError {
   fn from(
      item :   crate::os::console::ConsoleError,
   ) -> Self {
      return Self::new(item.into());
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

impl From<crate::os::console::ConsoleError> for ConsoleErrorKind {
   fn from(
      item : crate::os::console::ConsoleError,
   ) -> Self {
      use crate::os::console::ConsoleError::*;
      return match item {
         Unknown  => Self::Unknown,
      }
   }
}

///////////////////////
// METHODS - Console //
///////////////////////

impl Console {
   /// Creates a new console.  If an
   /// issue occurs such as a console
   /// already existing, an error is
   /// returned.
   pub fn new() -> Result<Self> {
      return Ok(Self(crate::os::console::Console::new()?));
   }

   /// Copies the window title of the
   /// console into an owned String.
   pub fn get_title(
      & self,
   ) -> Result<String> {
      return Ok(self.0.get_title()?);
   }

   /// Sets the window title of the
   /// console.
   pub fn set_title(
      & mut self,
      title : & str,
   ) -> Result<& mut Self> {
      self.0.set_title(title)?;
      return Ok(self);
   }
}

