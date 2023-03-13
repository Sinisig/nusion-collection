//! crate::entry OS implementations for Windows.

// This is how the sausage is made...
// Remember this isn't evaluated here, but
// instead in an arbitrary crate using nusion
// as a dependency.  This is why there is minimal
// usage of 'use' and functions are prefixed with
// double underscores.
#[macro_export]
macro_rules! build_entry {
   ($starter:path, $entry:ident, $osapi:path, $($proc:literal),*)  => {
      // Re-export because of weird issues expanding in-place
      use $osapi as __nusion_core_osapi;

      #[no_mangle]
      #[allow(non_snake_case)]
      extern "system" fn DllMain(
         handle_dll  : __nusion_core_osapi::shared::minwindef::HINSTANCE,
         call_reason : __nusion_core_osapi::shared::minwindef::DWORD,
         _           : __nusion_core_osapi::shared::minwindef::LPVOID,
      ) -> __nusion_core_osapi::shared::minwindef::BOOL {
         // Make sure we only execute on process attach
         if call_reason != __nusion_core_osapi::um::winnt::DLL_PROCESS_ATTACH {
            return __nusion_core_osapi::shared::minwindef::FALSE;
         }

         // Create the main execution thread
         let handle_thread = unsafe{__nusion_core_osapi::um::processthreadsapi::CreateThread(
            0 as __nusion_core_osapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
            0,
            Some(__nusion_slib_main_thread),
            handle_dll as __nusion_core_osapi::shared::minwindef::LPVOID,
            0,
            0 as __nusion_core_osapi::shared::minwindef::LPDWORD,
         )};
         if handle_thread == 0 as __nusion_core_osapi::shared::ntdef::HANDLE {
            if unsafe{__nusion_core_osapi::um::libloaderapi::FreeLibrary(
               handle_dll as __nusion_core_osapi::shared::minwindef::HMODULE,
            )} == __nusion_core_osapi::shared::minwindef::FALSE {
               let err = unsafe{__nusion_core_osapi::um::errhandlingapi::GetLastError()};
               panic!("Failed to free library after thread creation failed: {err:#X}");
            }
            return __nusion_core_osapi::shared::minwindef::FALSE;
         }

         // Close the thread handle
         if unsafe{__nusion_core_osapi::um::handleapi::CloseHandle(
            handle_thread,
         )} == __nusion_core_osapi::shared::minwindef::FALSE {
            let err = unsafe{__nusion_core_osapi::um::errhandlingapi::GetLastError()};
            panic!("Failed to close main thread creation handle: {err:#X}");
         }

         // Return success to the DLL loader
         return __nusion_core_osapi::shared::minwindef::TRUE;
      }

      extern "system" fn __nusion_slib_main_thread(
         handle_dll : __nusion_core_osapi::shared::minwindef::LPVOID,
      ) -> __nusion_core_osapi::shared::minwindef::DWORD {
         // Execute main, storing the return code for the end
         let return_code = $starter($entry, &[$($proc),*]).code;

         // Attempt to unload the library
         unsafe{__nusion_core_osapi::um::libloaderapi::FreeLibraryAndExitThread(
            handle_dll as __nusion_core_osapi::shared::minwindef::HMODULE,
            return_code,
         )}

         // Done to make the compiler happy
         return return_code;
      }
   };
}

