//! Console management module.

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// Contains error information relating
/// to the console.
#[derive(Debug)]
pub enum ConsoleError {
   InvalidTitleCharacters,
   Unknown,
}

/// <code>Result</code> type with error
/// variant <code>ConsoleError</code>.
pub type Result<T> = std::result::Result<T, ConsoleError>;

/// Creates a console window for displaying
/// output text from <code>stdout</code> and
/// <code>stderr</code>.  The console window
/// does not allow for input.
pub struct Console {
   console : crate::os::console::Console,
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

///////////////////////
// METHODS - Console //
///////////////////////

impl Console {
   /// Creates a new console window.
   pub fn new() -> Result<Self> {
      let mut console = crate::os::console::Console::allocate()?;

      console.set_title("unnamed console")?;

      return Ok(Self{
         console : console,
      });
   }

   /// Copies the window title of the
   /// console into an owned String.
   pub fn get_title(
      & self,
   ) -> Result<String> {
      return Ok(self.console.get_title()?);
   }

   /// Sets the console's window title.
   pub fn set_title(
      & mut self,
      new_title : & str,
   ) -> Result<()> {
      self.console.set_title(new_title)?;
      return Ok(());
   }
}

/////////////////////////////////////
// TRAIT IMPLEMENTATIONS - Console //
/////////////////////////////////////

impl Drop for Console {
   fn drop(
      & mut self,
   ) {
      crate::os::console::Console::free(
         & mut self.console,
      ).expect(
         "Failed to free console instance",
      );
      return;
   }
}

