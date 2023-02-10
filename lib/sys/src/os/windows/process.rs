//! crate::process implementations for
//! Windows.

use core::ffi::c_void;
use winapi::{
   shared::{
      minwindef::{
         DWORD,
         HMODULE,
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
      libloaderapi::{
         GetModuleFileNameA,
      },
      processthreadsapi::{
         GetCurrentProcessId,
      },
   },
};

#[derive(Debug)]
pub enum ProcessError {
   BadExecutableName,
   Unknown,
}

#[derive(Debug)]
pub enum ModuleError {
   Unknown,
}

pub struct ProcessSnapshot {
   process_id        : DWORD,
   executable_name   : String,
}

pub struct ModuleSnapshot<'l> {
   parent_process : &'l ProcessSnapshot,
   base_address   : * const c_void,
   dll_name       : String,
}

impl ProcessSnapshot {
   pub fn executable_name<'l>(
      &'l self,
   ) -> &'l str {
      return &self.executable_name;
   }

   pub fn current_process(
   ) -> Result<Self, ProcessError> {
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
         return Err(ProcessError::BadExecutableName);
      }

      // Remove excess capacity
      executable_name.truncate(character_count as usize);
      
      // Convert to a String
      let mut executable_name = String::from_utf8(executable_name).map_err(|_| {
         ProcessError::BadExecutableName
      })?;

      // Isolate just the file name
      let isolate_at = match executable_name.rfind('\\') {
         Some(n)  => n + 1,   // Exclusive index by skipping slash
         None     => return Err(ProcessError::BadExecutableName),
      };
      executable_name.drain(..isolate_at);

      return Ok(Self{
         process_id        : process_id,
         executable_name   : executable_name,
      });
   }

   pub fn enumerate_all(
   ) -> Result<Vec<Self>, ProcessError> {
      todo!()
   }
}

impl<'l> ModuleSnapshot<'l> {
   pub fn enumerate_all(
      parent_process : &'l ProcessSnapshot,
   ) -> Result<Vec<Self>, ModuleError> {
      todo!()
   }
}

