//! crate::entry OS implementations for Windows.

// This is how the sausage is made...
// Remember this isn't evaluated here, but
// instead in an arbitrary crate using nusion
// as a dependency.  This is why there is minimal
// usage of 'use' and functions are prefixed with
// double underscores.
#[macro_export]
macro_rules! os_build_slib_entry {
   ($entry:ident, $init:ident)  => {
      #[no_mangle]
      #[allow(non_snake_case)]
      extern "system" fn DllMain(
         dll_module  : nusion::__osapi::shared::minwindef::HINSTANCE,
         call_reason : nusion::__osapi::shared::minwindef::DWORD,
         _           : nusion::__osapi::shared::minwindef::LPVOID,
      ) -> nusion::__osapi::shared::minwindef::BOOL {
         // Make sure we only execute on attach
         if call_reason != nusion::__osapi::um::winnt::DLL_PROCESS_ATTACH {
            return nusion::__osapi::shared::minwindef::FALSE;
         }

         // Create the main execution thread
         let thread_handle = unsafe{nusion::__osapi::um::processthreadsapi::CreateThread(
            0 as nusion::__osapi::um::minwinbase::LPSECURITY_ATTRIBUTES,
            0,
            Some(__nusion_slib_main_thread),
            dll_module as nusion::__osapi::shared::minwindef::LPVOID,
            0,
            0 as nusion::__osapi::shared::minwindef::LPDWORD,
         )};
         if thread_handle == 0 as nusion::__osapi::shared::ntdef::HANDLE {
            return nusion::__osapi::shared::minwindef::FALSE;
         }

         // Close the thread handle
         if unsafe{nusion::__osapi::um::handleapi::CloseHandle(
            thread_handle,
         )} == nusion::__osapi::shared::minwindef::FALSE {
            panic!("Failed to close main thread creation handle");
         }

         // Return success to the DLL loader
         return nusion::__osapi::shared::minwindef::TRUE;
      }

      #[no_mangle]
      extern "system" fn __nusion_slib_main_thread(
         dll_module : nusion::__osapi::shared::minwindef::LPVOID,
      ) -> nusion::__osapi::shared::minwindef::DWORD {
         // Execute main
         let return_code = nusion::environment::Environment::$init($entry).get();

         // Attempt to unload the library
         unsafe{nusion::__osapi::um::libloaderapi::FreeLibraryAndExitThread(
            dll_module as nusion::__osapi::shared::minwindef::HMODULE,
            return_code,
         )}

         // Done to make the compiler happy
         return return_code;
      }
   };
}

