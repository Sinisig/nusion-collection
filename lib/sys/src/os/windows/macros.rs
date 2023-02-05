//! crate::macros OS implementations for Windows.

// This is how the sausage is made...
#[macro_export]
macro_rules! build_slib_entry_os {
   ($entry:ident, $init:ident)  => {
      use nusion::sys::os::windows::winapi as __winapi;

      #[no_mangle]
      #[allow(non_snake_case)]
      extern "system" fn DllMain(
         dll_module  : __winapi::shared::minwindef::HINSTANCE,
         call_reason : __winapi::shared::minwindef::DWORD,
         _           : __winapi::shared::minwindef::LPVOID,
      ) -> __winapi::shared::minwindef::BOOL {
         // Make sure we only execute on attach
         if call_reason != __winapi::um::winnt::DLL_PROCESS_ATTACH {
            return __winapi::shared::minwindef::FALSE;
         }

         // Create the main execution thread
         let thread_handle = unsafe{__winapi::um::processthreadsapi::CreateThread(
            0 as __winapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
            0,
            Some(main_thread),
            dll_module as __winapi::shared::minwindef::LPVOID,
            0,
            0 as __winapi::shared::minwindef::LPDWORD,
         )};
         if thread_handle == 0 as __winapi::shared::ntdef::HANDLE {
            return __winapi::shared::minwindef::FALSE;
         }

         // Close the thread handle
         if unsafe{__winapi::um::handleapi::CloseHandle(
            thread_handle
         )} == __winapi::shared::minwindef::FALSE {
            panic!("Failed to close main thread creation handle");
         }

         // Return success to the DLL loader
         return __winapi::shared::minwindef::TRUE;
      }

      #[no_mangle]
      extern "system" fn main_thread(
         dll_module : __winapi::shared::minwindef::LPVOID,
      ) -> __winapi::shared::minwindef::DWORD {
         // Execute main
         let return_code = nusion::sys::runtime::$init($entry) as __winapi::shared::minwindef::DWORD;

         // Attempt to unload the library
         unsafe{__winapi::um::libloaderapi::FreeLibraryAndExitThread(
            dll_module as __winapi::shared::minwindef::HMODULE,
            return_code,
         )}

         // Done to make the compiler happy
         return return_code;
      }
   };
}

