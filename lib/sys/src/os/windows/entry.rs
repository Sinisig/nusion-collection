//! crate::entry OS implementations for Windows.

// This is how the sausage is made...
// Remember this isn't evaluated here, but
// instead in an arbitrary crate using nusion
// as a dependency.  This is why there is minimal
// usage of 'use' and functions are prefixed with
// double underscores.
#[macro_export]
macro_rules! build_entry {
   ($starter:path, $entry:ident, $osapi:path)  => {
      // Re-export because of weird issues expanding in-place
      use $osapi as __nusion_osapi;

      #[no_mangle]
      #[allow(non_snake_case)]
      extern "system" fn DllMain(
         dll_module  : __nusion_osapi::shared::minwindef::HINSTANCE,
         call_reason : __nusion_osapi::shared::minwindef::DWORD,
         _           : __nusion_osapi::shared::minwindef::LPVOID,
      ) -> __nusion_osapi::shared::minwindef::BOOL {
         // Make sure we only execute on attach
         if call_reason != __nusion_osapi::um::winnt::DLL_PROCESS_ATTACH {
            return __nusion_osapi::shared::minwindef::FALSE;
         }

         // Create the main execution thread
         let thread_handle = unsafe{__nusion_osapi::um::processthreadsapi::CreateThread(
            0 as __nusion_osapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
            0,
            Some(__nusion_slib_main_thread),
            dll_module as __nusion_osapi::shared::minwindef::LPVOID,
            0,
            0 as __nusion_osapi::shared::minwindef::LPDWORD,
         )};
         if thread_handle == 0 as __nusion_osapi::shared::ntdef::HANDLE {
            return __nusion_osapi::shared::minwindef::FALSE;
         }

         // Close the thread handle
         if unsafe{__nusion_osapi::um::handleapi::CloseHandle(
            thread_handle,
         )} == __nusion_osapi::shared::minwindef::FALSE {
            panic!("Failed to close main thread creation handle");
         }

         // Return success to the DLL loader
         return __nusion_osapi::shared::minwindef::TRUE;
      }

      #[no_mangle]
      extern "system" fn __nusion_slib_main_thread(
         dll_module : __nusion_osapi::shared::minwindef::LPVOID,
      ) -> __nusion_osapi::shared::minwindef::DWORD {
         // Execute main, double deref to get raw i32
         let return_code = **$starter($entry);

         // Attempt to unload the library
         unsafe{__nusion_osapi::um::libloaderapi::FreeLibraryAndExitThread(
            dll_module as __nusion_osapi::shared::minwindef::HMODULE,
            return_code,
         )}

         // Done to make the compiler happy
         return return_code;
      }
   };
}

