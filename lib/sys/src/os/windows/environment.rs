//! crate::os::environment implementations
//! for Windows.

pub type OSReturn = winapi::shared::minwindef::DWORD;

pub const EXIT_SUCCESS : OSReturn = 0;
pub const EXIT_FAILURE : OSReturn = 1;

