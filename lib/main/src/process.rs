//! Retrieve information about running
//! processes and create a snapshot of
//! processes and their loaded libraries.

use std::collections::hash_map::HashMap;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error enum containing the
/// reason behind a process function
/// failing.
#[derive(Debug)]
pub enum ProcessError {
   BadExecutableFileName,
   Unknown,
}

/// A Result type with Err variant
/// ProcessError
pub type Result<T> = std::result::Result<T, ProcessError>;

/// A snapshot of various information
/// about a process.
pub struct ProcessSnapshot {
   snap  : crate::sys::process::ProcessSnapshot,
}

/// A snapshot of a loaded library
/// or executable within a process.
pub struct ModuleSnapshot {
   snap  : crate::sys::process::ModuleSnapshot,
}

/// A memory patch acting on a module
/// snapshot.  Patched bytes are restored
/// to their original bytes when the
/// container is dropped.
pub struct ModuleSnapshotPatchContainer {
   address_range  : std::ops::Range<usize>,
   old_bytes      : Vec<u8>,
}

/// A list of process snapshots.  Useful
/// for enumerating and searching the
/// entire system process tree.
pub struct ProcessSnapshotList {
   processes   : HashMap<String, ProcessSnapshot>,
}

/// A list of module snapshots from a
/// process.  Useful for searching for
/// a specific module within a process.
pub struct ModuleSnapshotList {
   parent   : ProcessSnapshot,
   modules  : HashMap<String, ModuleSnapshot>,
}

/// An iterator over a ProcessSnapshotList.
pub struct ProcessSnapshotListIterator<'s> {
   iter : std::collections::hash_map::Iter<'s, String, ProcessSnapshot>,
}

/// An iterator over a ModuleSnapshotList.
pub struct ModuleSnapshotListIterator<'s> {
   iter : std::collections::hash_map::Iter<'s, String, ModuleSnapshot>,
}

/// A consuming iterator over a
/// ProcessSnapshotList.
pub struct ProcessSnapshotListIntoIterator {
   iter : std::collections::hash_map::IntoValues<String, ProcessSnapshot>,
}

/// A consuming iterator over a
/// ModuleSnapshotList.
pub struct ModuleSnapshotListIntoIterator {
   iter : std::collections::hash_map::IntoValues<String, ModuleSnapshot>,
}

//////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ProcessError //
//////////////////////////////////////////

impl std::fmt::Display for ProcessError {
   fn fmt(
      & self,
      stream : & mut std::fmt::Formatter<'_>,
   ) -> std::fmt::Result {
      return write!(stream, "{}", match self {
         Self::BadExecutableFileName
            => "Executable file name contains invalid characters",
         Self::Unknown
            => "Unknown",
      });
   }
}

impl std::error::Error for ProcessError {
}

impl From<crate::sys::process::ProcessError> for ProcessError {
   fn from(
      item : crate::sys::process::ProcessError,
   ) -> Self {
      use crate::sys::process::ProcessError::*;
      return match item {
         BadExecutableFileName
            => Self::BadExecutableFileName,
         Unknown
            => Self::Unknown,
      };
   }
}

///////////////////////////////
// METHODS - ProcessSnapshot //
///////////////////////////////

impl ProcessSnapshot {
   /// Creates a snapshot of the
   /// program's own process.
   pub fn local(
   ) -> Result<Self> {
      return Ok(Self{
         snap : crate::sys::process::ProcessSnapshot::local()?,
      });
   }

   /// Gets the file name of the
   /// executable which spawned the
   /// process.  This only includes
   /// the file name and extension
   /// without the containing file
   /// path.
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return self.snap.executable_file_name();
   }
}

//////////////////////////////
// METHODS - ModuleSnapshot //
//////////////////////////////

impl ModuleSnapshot {
   /// Gets the address space range
   /// occupied by the module within
   /// the parent process.
   pub fn address_range<'l>(
      &'l self,
   ) -> &'l std::ops::Range<usize> {
      return self.snap.address_range();
   }

   /// Gets the file name of the
   /// module.  This only includes
   /// the file name and extension
   /// without the containing file
   /// path.
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return self.snap.executable_file_name();
   }
}

////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ModuleSnapshot //
////////////////////////////////////////////

unsafe impl crate::patch::Patch for ModuleSnapshot {
   type Container = ModuleSnapshotPatchContainer;

   /// Patches using an offset in the
   /// module's address space.  See
   /// the Patch trait for more
   /// documentation.
   unsafe fn patch<R, P>(
      & mut self,
      memory_offset_range  : R,
      predicate            : P,
   ) -> crate::patch::Result<Self::Container>
   where R: std::ops::RangeBounds<usize>,
         P: FnOnce(& mut [u8]) -> crate::patch::Result<()>
   {
      // Map input offset range into real address range
      let base_address = self.address_range().start;

      use std::ops::Bound;
      let lower_bound = match memory_offset_range.start_bound() {
         Bound::Included(b)
            => base_address + *b,
         Bound::Excluded(b)
            => base_address + *b + 1,
         Bound::Unbounded
            => base_address,
      };
      let upper_bound = match memory_offset_range.end_bound() {
         Bound::Included(b)
            => base_address + *b + 1,
         Bound::Excluded(b)
            => base_address + *b,
         Bound::Unbounded
            => self.address_range().end,
      };

      let memory_address_range = lower_bound..upper_bound;

      // Open the range for reading/writing
      let mut editor = crate::sys::memory::MemoryEditor::open_read_write(
         memory_address_range.clone(),
      )?;

      // Store the old bytes in a new container
      let container = Self::Container{
         address_range  : memory_address_range.clone(),
         old_bytes      : editor.bytes_mut().to_vec(),
      };

      // Run the closure to patch the bytes
      predicate(editor.bytes_mut())?;

      // Return success
      return Ok(container);
   }
}

//////////////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ModuleSnapshotPatchContainer //
//////////////////////////////////////////////////////////

impl std::ops::Drop for ModuleSnapshotPatchContainer {
   fn drop(
      & mut self,
   ) {
      let mut editor = crate::sys::memory::MemoryEditor::open_read_write(
         self.address_range.clone(),
      ).expect("Failed to restore patched module bytes");

      unsafe{editor.bytes_mut().copy_from_slice(&self.old_bytes)};

      return;
   }
}

///////////////////////////////////
// METHODS - ProcessSnapshotList //
///////////////////////////////////

impl ProcessSnapshotList {
   /// Creates an empty process
   /// snapshot list.
   pub fn new(
   ) -> Self {
      return Self{
         processes : HashMap::new(),
      };
   }

   /// Creates a snapshot of every
   /// process visible to the user
   /// and stores it in the list.
   pub fn all(
   ) -> Result<Self> {
      let proc = crate::sys::process::ProcessSnapshot::all()?;

      let mut hash = HashMap::with_capacity(proc.len());
      for proc in proc {
         let proc = ProcessSnapshot{
            snap : proc,
         };

         hash.insert(
            String::from(proc.executable_file_name()),
            proc,
         );
      }

      return Ok(Self{
         processes : hash,
      });
   } 

   /// Adds a process snapshot to
   /// the list.
   pub fn insert(
      & mut self,
      process_snapshot  : ProcessSnapshot
   ) -> & mut Self {
      self.processes.insert(
         String::from(process_snapshot.executable_file_name()),
         process_snapshot,
      );
      return self;
   }

   /// Removes a process from the
   /// list by searching for its
   /// executable file name, returning
   /// the process snapshot.
   pub fn remove_by_executable_file_name(
      & mut self,
      executable_file_name : & str,
   ) -> Option<ProcessSnapshot> {
      return self.processes.remove(executable_file_name);
   }

   /// Tries to find a process by
   /// its executable file name.
   pub fn find_by_executable_file_name(
      & self,
      executable_file_name : & str,
   ) -> Option<& ProcessSnapshot> {
      return self.processes.get(executable_file_name);
   } 

   /// Tries to find a mutable process
   /// by its executable file name.
   pub fn find_mut_by_executable_file_name(
      & mut self,
      executable_file_name : & str,
   ) -> Option<& mut ProcessSnapshot> {
      return self.processes.get_mut(executable_file_name);
   }

   /// Creates an iterator over the
   /// processes in the list.
   pub fn iter<'l>(
      &'l self,
   ) -> ProcessSnapshotListIterator<'l> {
      return ProcessSnapshotListIterator{
         iter : self.processes.iter(),
      };
   }

   /// Creates a consuming iterator
   /// over the processes in the list.
   pub fn into_iter(
      self,
   ) -> ProcessSnapshotListIntoIterator {
      return ProcessSnapshotListIntoIterator{
         iter : self.processes.into_values(),
      };
   }
}

//////////////////////////////////
// METHODS - ModuleSnapshotList //
//////////////////////////////////

impl ModuleSnapshotList {
   /// Creates an empty module
   /// snapshot list bound to
   /// a process snapshot.
   pub fn new(
      process_snapshot : ProcessSnapshot,
   ) -> Self {
      return Self{
         parent   : process_snapshot,
         modules  : HashMap::new(),
      };
   }

   /// Creates a snapshot of every
   /// module within a given process
   /// snapshot and stores it in the
   /// list.
   pub fn all(
      process_snapshot  : ProcessSnapshot,
   ) -> Result<Self> {
      let list = crate::sys::process::ModuleSnapshot::all_within(
         &process_snapshot.snap,
      )?;

      let mut hash = HashMap::with_capacity(list.len());
      for module in list {
         let module = ModuleSnapshot{
            snap : module,
         };

         hash.insert(
            String::from(module.executable_file_name()),
            module,
         );
      }

      return Ok(Self{
         parent   : process_snapshot,
         modules  : hash,
      });
   }

   /// Adds a module snapshot to
   /// the list.
   pub fn insert(
      & mut self,
      module_snapshot   : ModuleSnapshot
   ) -> & mut Self {
      self.modules.insert(
         String::from(module_snapshot.executable_file_name()),
         module_snapshot,
      );
      return self;
   }

   /// Removes a module from the
   /// list by searching for its
   /// executable file name, returning
   /// the module snapshot.
   pub fn remove_by_executable_file_name(
      & mut self,
      executable_file_name : & str,
   ) -> Option<ModuleSnapshot> {
      return self.modules.remove(executable_file_name);
   }

   /// Tries to find a module by
   /// its executable file name.
   pub fn find_by_executable_file_name(
      & self,
      executable_file_name : & str,
   ) -> Option<& ModuleSnapshot> {
      return self.modules.get(executable_file_name);
   }

   /// Tries to find a mutable module
   /// by its executable file name.
   pub fn find_mut_by_executable_file_name(
      & mut self,
      executable_file_name : & str,
   ) -> Option<& mut ModuleSnapshot> {
      return self.modules.get_mut(executable_file_name);
   }

   /// Returns a reference to the process
   /// snapshot which the module snapshot
   /// list belongs to.
   pub fn parent_process(
      & self,
   ) -> & ProcessSnapshot {
      return &self.parent;
   }

   /// Creates an iterator over the
   /// modules in the list.
   pub fn iter<'l>(
      &'l self,
   ) -> ModuleSnapshotListIterator<'l> {
      return ModuleSnapshotListIterator{
         iter : self.modules.iter(),
      };
   }

   /// Creates a consuming iterator
   /// over the modules in the list.
   pub fn into_iter(
      self,
   ) -> ModuleSnapshotListIntoIterator {
      return ModuleSnapshotListIntoIterator{
         iter : self.modules.into_values(),
      };
   }
}

/////////////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ProcessSnapshotListIterator //
/////////////////////////////////////////////////////////

impl<'s> std::iter::Iterator for ProcessSnapshotListIterator<'s> {
   type Item = &'s ProcessSnapshot;

   fn next(
      & mut self,
   ) -> Option<Self::Item> {
      return self.iter.next().map(|(_, v)| v);
   }
}

////////////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ModuleSnapshotListIterator //
////////////////////////////////////////////////////////

impl<'s> std::iter::Iterator for ModuleSnapshotListIterator<'s> {
   type Item = &'s ModuleSnapshot;

   fn next(
      & mut self,
   ) -> Option<Self::Item> {
      return self.iter.next().map(|(_, v)| v);
   }
}

/////////////////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ProcessSnapshotListIntoIterator //
/////////////////////////////////////////////////////////////

impl std::iter::IntoIterator for ProcessSnapshotListIntoIterator {
   type Item      = ProcessSnapshot;
   type IntoIter  = std::collections::hash_map::IntoValues<String, ProcessSnapshot>;

   fn into_iter(
      self,
   ) -> Self::IntoIter {
      return self.iter;
   }
}

////////////////////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ModuleSnapshotListIntoIterator //
////////////////////////////////////////////////////////////

impl std::iter::IntoIterator for ModuleSnapshotListIntoIterator {
   type Item      = ModuleSnapshot;
   type IntoIter  = std::collections::hash_map::IntoValues<String, ModuleSnapshot>;

   fn into_iter(
      self,
   ) -> Self::IntoIter {
      return self.iter;
   }
}

