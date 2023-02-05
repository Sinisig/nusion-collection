//! Console management module.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Contains error information relating
/// to the console.
#[derive(Debug)]
pub struct ConsoleError(crate::os::console::ConsoleError);

/// Result type with Ok variant T and Err variant ConsoleError.
pub type Result<T> = std::result::Result<T, ConsoleError>;

/// Stores a handle to the console.
pub struct Console(crate::os::console::Console);

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ConsoleError //
//////////////////////////////////////////

impl std::fmt::Display for ConsoleError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", self.0);
   }
}

impl std::error::Error for ConsoleError {
}

impl From<crate::os::console::ConsoleError> for ConsoleError {
   fn from(
      item :   crate::os::console::ConsoleError,
   ) -> Self {
      return Self(item);
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

