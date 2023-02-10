//! crate::process implementations for
//! Windows.

use crate::process::{ProcessError, Result};

use core::ffi::c_void;
use winapi::{
   shared::{
      minwindef::{
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

pub struct ProcessSnapshot {
   process_id        : DWORD,
   executable_name   : String,
}

pub struct ModuleSnapshot<'l> {
   parent_process : &'l ProcessSnapshot,
   address_range  : std::ops::Range<* const c_void>,
   module_name    : String,
}

impl ProcessSnapshot {
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return &self.executable_name;
   }

   pub fn local(
   ) -> Result<Self> {
      // MAX_PATH plus room for a null terminator and another
      // byte to check for errors
      const NAME_BUFFER_SIZE : DWORD = MAX_PATH as DWORD + 2;

      // Gets the process id
      let process_id = unsafe{GetCurrentProcessId()};

      // Creates byte buffer for file path (including null terminator)
      let mut executable_name = Vec::<u8>::with_capacity(NAME_BUFFER_SIZE as usize);
      unsafe{executable_name.set_len(NAME_BUFFER_SIZE as usize)};

      // Retrieves the file path
      let character_count = unsafe{GetModuleFileNameA(
         0 as HMODULE,
         executable_name.as_mut_ptr() as LPSTR,
         NAME_BUFFER_SIZE,
      )};

      // Check for failure
      // Double checks for compatibility with WinXP
      if character_count == 0                ||
         character_count == NAME_BUFFER_SIZE ||
         unsafe{GetLastError()} == ERROR_INSUFFICIENT_BUFFER
      {
         return Err(ProcessError::BadExecutableFileName);
      }

      // Remove excess capacity
      executable_name.truncate(character_count as usize);
      
      // Convert to a String
      let mut executable_name = String::from_utf8(executable_name).map_err(|_| {
         ProcessError::BadExecutableFileName
      })?;

      // Isolate just the file name
      let isolate_at = match executable_name.rfind('\\') {
         Some(n)  => n + 1,   // Exclusive index by skipping slash
         None     => return Err(ProcessError::BadExecutableFileName),
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
         // TODO: Better error propagation
         return Err(ProcessError::Unknown);
      };

      // Get the process info for the first process
      let mut process_entry = unsafe{
         std::mem::MaybeUninit::<PROCESSENTRY32>::uninit().assume_init()
      };
      process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as DWORD;
      if unsafe{Process32First(process_snapshot, & mut process_entry)} == FALSE {
         // TODO: Proper error handling
         if unsafe{CloseHandle(process_snapshot)} == FALSE {
            panic!("Failed to close process snapshot handle");
         }

         return Err(ProcessError::Unknown);
      }

      // Get process information for every process
      let mut process_list = Vec::new();
      'process_loop : loop {
         // Get the PID and EXE name for the process
         let process_id    = process_entry.th32ProcessID;
         let process_exe   = &process_entry.szExeFile[..];

         // Convert EXE name to an owned string
         let process_exe = unsafe{std::slice::from_raw_parts(
            process_exe.as_ptr() as * const u8,
            process_exe.len() * std::mem::size_of::<u8>(),
         )};
         let process_exe = process_exe.to_vec();
         let process_exe = match String::from_utf8(process_exe) {
            Ok(s)    => s,
            Err(_)   => {
               if unsafe{CloseHandle(process_snapshot)} == FALSE {
                  panic!("Failed to close process snapshot handle");
               }
               return Err(ProcessError::BadExecutableFileName);
            },
         };

         // Create a ProcessSnapshot from the current
         // process entry and add it to the list
         process_list.push(Self{
            process_id        : process_id,
            executable_name   : process_exe,
         });

         // Load the next process entry
         if unsafe{Process32Next(
            process_snapshot, & mut process_entry,
         )} == FALSE {
            break 'process_loop;
         }
      } 

      // Close the process snapshot handle and return
      if unsafe{CloseHandle(process_snapshot)} == FALSE {
         panic!("Failed to close process snapshot handle");
      }
      return Ok(process_list);
   }
}

impl<'l> ModuleSnapshot<'l> {
   pub fn parent_process(
      & self,
   ) -> &'l ProcessSnapshot {
      return self.parent_process;
   }

   pub fn executable_file_name<'u>(
      &'u self,
   ) -> &'u str {
      return &self.module_name;
   }

   pub fn address_range<'u>(
      &'u self,
   ) -> &'u std::ops::Range<* const c_void> {
      return &self.address_range;
   }

   pub fn all(
      parent_process : &'l ProcessSnapshot,
   ) -> Result<Vec<Self>> {
      // Create a snapshot of modules in the given process
      let module_snapshot = unsafe{CreateToolhelp32Snapshot(
         TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, parent_process.process_id,
      )};
      if module_snapshot == INVALID_HANDLE_VALUE {
         // TODO: Better error propagation
         return Err(ProcessError::Unknown);
      }

      // Get the first module entry
      let mut module_entry = unsafe{
         std::mem::MaybeUninit::<MODULEENTRY32>::uninit().assume_init()
      };
      module_entry.dwSize = std::mem::size_of::<MODULEENTRY32>() as DWORD;
      if unsafe{Module32First(module_snapshot, & mut module_entry)} == FALSE {
         // TODO: Better error propagation
         if unsafe{CloseHandle(module_snapshot)} == FALSE {
            panic!("Failed to close module snapshot handle");
         }
         return Err(ProcessError::Unknown);
      }

      // Create the module list and start enumerating
      let mut module_list = Vec::new();
      'module_loop : loop {
         // Get the address range
         let base_address  = module_entry.modBaseAddr as * const c_void;
         let end_address   = unsafe{(base_address as * const u8).add(module_entry.modBaseSize as usize + 1)} as * const c_void;
         let address_range = base_address..end_address;

         // Get the C-string array and string length
         let dll_name      = &module_entry.szModule[..];
         let dll_name_len  = match dll_name.iter().position(|c| *c == 0x00) {
            Some(len)   => len,
            None        => MAX_PATH,
         };

         // Convert to an owned String
         let dll_name = unsafe{std::slice::from_raw_parts(
            dll_name.as_ptr() as * const u8,
            dll_name_len,
         )};
         let dll_name = dll_name.to_vec();
         let dll_name = match String::from_utf8(dll_name) {
            Ok(s)    => s,
            Err(_)   => {
               if unsafe{CloseHandle(module_snapshot)} == FALSE {
                  panic!("Failed to close module snapshot handle");
               }
               return Err(ProcessError::BadExecutableFileName);
            },
         };

         // Create a new ModuleSnapshot and add it to
         // the list
         module_list.push(Self{
            parent_process : parent_process,
            address_range  : address_range,
            module_name    : dll_name,
         });

         // Load the next module entry
         if unsafe{Module32Next(
            module_snapshot, & mut module_entry,
         )} == FALSE {
            break 'module_loop;
         }
      }

      // Close the module snapshot handle and return
      if unsafe{CloseHandle(module_snapshot)} == FALSE {
         panic!("Failed to close module snapshot handle");
      }
      return Ok(module_list);
   }
}

