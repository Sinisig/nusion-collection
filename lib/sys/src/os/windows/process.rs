//! crate::process implementations for
//! Windows.

use crate::process::{ProcessError, Result};

use winapi::{
   shared::{
      basetsd::{
         ULONG_PTR,
      },
      minwindef::{
         BYTE,
         DWORD,
         HMODULE,
         FALSE,
         MAX_PATH,
      },
      ntdef::{
         LPSTR,
      },
      winerror::{
         ERROR_INSUFFICIENT_BUFFER,
      },
   },
   um::{
      errhandlingapi::{
         GetLastError,
      },
      handleapi::{
         CloseHandle,
         INVALID_HANDLE_VALUE,
      },
      libloaderapi::{
         GetModuleFileNameA,
      },
      processthreadsapi::{
         GetCurrentProcessId,
      },
      tlhelp32::{
         CreateToolhelp32Snapshot,
         Process32First,
         Process32Next,
         Module32First,
         Module32Next,
         PROCESSENTRY32,
         MODULEENTRY32,
         TH32CS_SNAPPROCESS,
         TH32CS_SNAPMODULE,
         TH32CS_SNAPMODULE32,
      },
   },
};

const EXECUTABLE_FILE_PATH_MAX_LENGTH : DWORD
   = MAX_PATH as DWORD;

pub struct ProcessSnapshot {
   pub process_id       : DWORD,
   pub executable_name  : String,
}

pub struct ModuleSnapshot {
   pub address_range : std::ops::Range<usize>,
   pub module_name   : String,
}

macro_rules! try_close_handle {
   ($handle:ident, $msg:literal) => {
      if unsafe{CloseHandle($handle)} == FALSE {
         panic!("Failed to close {} handle", $msg);
      }
   };
}

fn cstr_to_owned_string(
   string : &[i8],
) -> Option<String> {
   let string = unsafe{std::slice::from_raw_parts(
      string.as_ptr() as * const u8,
      string.len(),
   )};
   
   // Strips out null bytes if there are any
   // This works with UTF-8, which luckily is
   // all we care about
   let idx_null   = string.iter().position(|e| *e == 0x00)?;
   let string     = &string[..idx_null];

   let string = string.to_vec();
   let string = match String::from_utf8(string) {
      Ok(s)    => s,
      Err(_)   => return None,
   };

   return Some(string);
}

impl ProcessSnapshot {
   pub fn local(
   ) -> Result<Self> {
      // MAX_PATH plus room for a null terminator
      const NAME_BUFFER_SIZE : DWORD 
         = EXECUTABLE_FILE_PATH_MAX_LENGTH + 1;

      // Gets the process id
      let process_id = unsafe{GetCurrentProcessId()};

      // Creates byte buffer for file path (including null terminator)
      let mut executable_name = Vec::<i8>::with_capacity(NAME_BUFFER_SIZE as usize);
      unsafe{executable_name.set_len(NAME_BUFFER_SIZE as usize)};

      // Retrieves the file path
      let character_count = unsafe{GetModuleFileNameA(
         0 as HMODULE,
         executable_name.as_mut_ptr() as LPSTR,
         NAME_BUFFER_SIZE,
      )};

      // Check for failure
      if character_count         == NAME_BUFFER_SIZE  ||
         unsafe{GetLastError()}  == ERROR_INSUFFICIENT_BUFFER
      {
         return Err(ProcessError::BadExecutableFileName);
      }

      // Convert to a String, yes this involves
      // making a duplicate vector...too bad!
      let mut executable_name = match cstr_to_owned_string(&executable_name) {
         Some(s)  => s,
         None     => return Err(crate::process::ProcessError::BadExecutableFileName),
      };

      // Isolate just the file name
      let isolate_at = match executable_name.rfind('\\') {
         Some(n)  => n + 1,   // Exclusive index by skipping slash
         None     => 0,       // Don't remove anything
      };
      executable_name.drain(..isolate_at);

      return Ok(Self{
         process_id        : process_id,
         executable_name   : executable_name,
      });
   }

   pub fn all(
   ) -> Result<Vec<Self>> {
      // Create a process snapshot
      let process_snapshot = unsafe{CreateToolhelp32Snapshot(
         TH32CS_SNAPPROCESS, 0,
      )};
      if process_snapshot == INVALID_HANDLE_VALUE {
         return Err(ProcessError::Unknown);
      };

      // Get the process info for the first process
      let mut process_entry = PROCESSENTRY32{
         dwSize               : std::mem::size_of::<PROCESSENTRY32>() as DWORD,
         cntUsage             : 0,
         th32ProcessID        : 0,
         th32DefaultHeapID    : 0 as ULONG_PTR,
         th32ModuleID         : 0,
         cntThreads           : 0,
         th32ParentProcessID  : 0,
         pcPriClassBase       : 0,
         dwFlags              : 0,
         szExeFile            : [0; 260],
      };
      if unsafe{Process32First(process_snapshot, & mut process_entry)} == FALSE {
         try_close_handle!(process_snapshot, "process snapshot");
         return Err(ProcessError::Unknown);
      }

      // Get process information for every process
      let mut process_list = Vec::new();
      'process_loop : loop {
         // Macro for loading the next process
         // in the list
         macro_rules! load_next {
            () => {
               if unsafe{Process32Next(
                  process_snapshot, & mut process_entry,
               )} == FALSE {
                  break 'process_loop;
               }
            }
         }

         // Get the PID and EXE name for the process
         let process_id    = process_entry.th32ProcessID;
         let process_exe   = &process_entry.szExeFile[..];

         // Convert the EXE name to an owned string
         let process_exe = match cstr_to_owned_string(process_exe) {
            Some(s)  => s,
            None     => {
               load_next!();
               continue;
            },
         };

         // Create a ProcessSnapshot from the current
         // process entry and add it to the list
         process_list.push(Self{
            process_id        : process_id,
            executable_name   : process_exe,
         });

         // Load the next process entry
         load_next!();
      } 

      // Close the process snapshot handle and return
      try_close_handle!(process_snapshot, "process snapshot");
      return Ok(process_list);
   }
}

impl ModuleSnapshot {
   pub fn all(
      parent_process : & ProcessSnapshot,
   ) -> Result<Vec<Self>> {
      // Create a snapshot of modules in the given process
      let module_snapshot = unsafe{CreateToolhelp32Snapshot(
         TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, parent_process.process_id,
      )};
      if module_snapshot == INVALID_HANDLE_VALUE {
         return Err(ProcessError::Unknown);
      }

      // Get the first module entry
      let mut module_entry = MODULEENTRY32{
         dwSize         : std::mem::size_of::<MODULEENTRY32>() as DWORD,
         th32ModuleID   : 0,
         th32ProcessID  : 0,
         GlblcntUsage   : 0,
         ProccntUsage   : 0,
         modBaseAddr    : 0 as * mut BYTE,
         modBaseSize    : 0,
         hModule        : 0 as HMODULE,
         szModule       : [0; 256],
         szExePath      : [0; 260],
      };
      if unsafe{Module32First(module_snapshot, & mut module_entry)} == FALSE {
         try_close_handle!(module_snapshot, "module snapshot");
         return Err(ProcessError::Unknown);
      }

      // Create the module list and start enumerating
      let mut module_list = Vec::new();
      'module_loop : loop {
         // Macro for loading the next module
         // in the list
         macro_rules! load_next {
            () => {
               if unsafe{Module32Next(
                  module_snapshot, & mut module_entry,
               )} == FALSE {
                  break 'module_loop;
               }
            }
         }

         // Get the address range
         let base_address  = module_entry.modBaseAddr as usize;
         let end_address   = unsafe{(base_address as * const u8).add(module_entry.modBaseSize as usize + 1)} as usize;
         let address_range = base_address..end_address;

         // Get DLL name and convert to an owned String
         let dll_name = &module_entry.szModule[..];
         let dll_name = match cstr_to_owned_string(dll_name) {
            Some(s)  => s,
            None     => {
               load_next!();
               continue;
            },
         };

         // Create a new ModuleSnapshot and add it to
         // the list
         module_list.push(Self{
            address_range  : address_range,
            module_name    : dll_name,
         });

         // Load the next module entry
         load_next!();
      }

      // Close the module snapshot handle and return
      try_close_handle!(module_snapshot, "module snapshot");
      return Ok(module_list);
   }
}

