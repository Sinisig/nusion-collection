//! Runtime initialization for custom
//! entrypoints.

/// OS return code when start_main succeeds.
pub const EXIT_SUCCESS : i32 = 0;
pub const EXIT_FAILURE : i32 = 1;

/// Starts main with no return type.
pub fn run_main_default<F>(
   entry : F,
) -> i32
where F: FnOnce(),
{
   entry();
   return EXIT_SUCCESS;
}

/// Starts main with a
/// Result<(), E: std::error::Error>
/// return type.
pub fn run_main_result_static<F, E>(
   entry : F,
) -> i32
where F: FnOnce() -> Result<(), E>,
      E: std::error::Error,
{
   if let Err(err) = entry() {
      eprintln!("Error: {}", err.to_string());
      return EXIT_FAILURE;
   }

   return EXIT_SUCCESS;
}

/// Starts main with a
/// Result<(), Box<dyn std::error::Error>>
/// return type.
pub fn run_main_result_dynamic<F>(
   entry : F,
) -> i32
where F: FnOnce() -> Result<(), Box<dyn std::error::Error>>
{
   if let Err(err) = entry() {
      eprintln!("Error: {}", err.to_string());
      return EXIT_FAILURE;
   }

   return EXIT_SUCCESS;
}

