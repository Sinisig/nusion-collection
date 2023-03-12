//! Enumerate information about running
//! processes.

use std::collections::hash_map::HashMap;
use std::ops::RangeBounds;

//////////////////////
// TYPE DEFINITIONS //
//////////////////////

/// An error relating to a process
/// or module function failing.
#[derive(Debug)]
pub enum ProcessError {
   BadExecutableFileName,
   Unknown,
}

/// <code>Result</code> type with error
/// variant <code>ProcessError</code>.
pub type Result<T> = std::result::Result<T, ProcessError>;

/// A snapshot of a process' information.
/// If the process exits while the snapshot
/// is in use, all functions acting on the
/// process itself and not just the snapshot
/// will return an error.
pub struct ProcessSnapshot {
   snapshot : crate::sys::process::ProcessSnapshot,
}

/// A snapshot of a module loaded
/// within a process such as a
/// dynamically loaded library.
/// If the module unloads while the
/// snapshot is in use, all functions
/// acting on the process itself and not
/// just the snapshot will return an
/// error.
pub struct ModuleSnapshot {
   snapshot : crate::sys::process::ModuleSnapshot,
}

/// The container for storing patched
/// bytes in a module for restoration
/// when the instance is dropped.
pub struct ModuleSnapshotPatchContainer {
   address_range  : std::ops::Range<usize>,
   old_bytes      : Vec<u8>,
}

/// A list of process snapshots created
/// by enumerating the system for running
/// process information.
pub struct ProcessSnapshotList {
   processes   : HashMap<String, ProcessSnapshot>,
}

/// A list of module snapshots created
/// by enumerating all modules within
/// a process snapshot.
pub struct ModuleSnapshotList {
   parent   : ProcessSnapshot,
   modules  : HashMap<String, ModuleSnapshot>,
}

pub struct ProcessSnapshotListIterator<'s> {
   iter : std::collections::hash_map::Iter<'s, String, ProcessSnapshot>,
}

pub struct ModuleSnapshotListIterator<'s> {
   iter : std::collections::hash_map::Iter<'s, String, ModuleSnapshot>,
}

pub struct ProcessSnapshotListIntoIterator {
   iter : std::collections::hash_map::IntoValues<String, ProcessSnapshot>,
}

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
         snapshot : crate::sys::process::ProcessSnapshot::local()?,
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
      return self.snapshot.executable_file_name();
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
      return self.snapshot.address_range();
   }

   /// Gets the file name of the
   /// module.  This only includes
   /// the file name and extension
   /// without the containing file
   /// path.
   pub fn executable_file_name<'l>(
      &'l self,
   ) -> &'l str {
      return self.snapshot.executable_file_name();
   }
}

///////////////////////////////////////
// INTERNAL HELPERS - ModuleSnapshot //
///////////////////////////////////////

impl ModuleSnapshot {
   fn offset_range_to_address_range<R>(
      & self,
      offset_range   : & R,
   ) -> crate::patch::Result<std::ops::Range<usize>>
   where R: RangeBounds<usize>,
   {
      let address_start = self.address_range().start;
      let address_end   = self.address_range().end;

      use std::ops::Bound;
      let offset_start = match offset_range.start_bound() {
         Bound::Included(b)
            => b.clone(),
         Bound::Excluded(b)
            => b.checked_add(1).ok_or(crate::patch::PatchError::OutOfRange{
               maximum  : usize::MAX,
               provided : b.clone(),
            })?,
         Bound::Unbounded
            => 0,
      };
      let offset_end = match offset_range.end_bound() {
         Bound::Included(b)
            => b.checked_add(1).ok_or(crate::patch::PatchError::OutOfRange{
               maximum  : usize::MAX,
               provided : b.clone(),
            })?,
         Bound::Excluded(b)
            => b.clone(),
         Bound::Unbounded
            => address_end - address_start, // Will always be valid
      };

      let address_target_start = address_start
         .checked_add(offset_start)
         .ok_or(crate::patch::PatchError::OutOfRange{
            maximum  : usize::MAX - address_start,
            provided : offset_start,
         })?;

      let address_target_end = address_start
         .checked_add(offset_end)
         .ok_or(crate::patch::PatchError::OutOfRange{
            maximum  : usize::MAX - address_end,
            provided : offset_end,
         })?;

      if address_target_end > address_end {
         return Err(crate::patch::PatchError::OutOfRange{
            maximum  : address_end - address_start,
            provided : offset_end,
         });
      }
      if address_target_end < address_target_start {
         return Err(crate::patch::PatchError::EndOffsetBeforeStartOffset);
      }

      return Ok(address_target_start..address_target_end);
   }
}

////////////////////////////////////////////
// TRAIT IMPLEMENTATIONS - ModuleSnapshot //
////////////////////////////////////////////

impl crate::patch::Patch for ModuleSnapshot {
   type Container = ModuleSnapshotPatchContainer;

   unsafe fn patch_read<Rd, Mr>(
      & self,
      reader : & Rd,
   ) -> crate::patch::Result<Rd::Item>
   where Rd: crate::patch::Reader<Mr>,
         Mr: RangeBounds<usize>,
   {
      let address_range = self.offset_range_to_address_range(
         reader.memory_offset_range(),
      )?;

      let editor = crate::sys::memory::MemoryEditor::open_read(
         address_range,
      )?;

      let bytes = editor.as_bytes();

      let item = reader.read_item(bytes)?;

      return Ok(item);
   }

   unsafe fn patch_write<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> crate::patch::Result<()>
   where Wt: crate::patch::Writer<Mr>,
         Mr: RangeBounds<usize>,
   {
      let address_range = self.offset_range_to_address_range(
         writer.memory_offset_range(),
      )?;

      let mut editor = crate::sys::memory::MemoryEditor::open_read_write(
         address_range,
      )?;

      let bytes = editor.as_bytes_mut();

      let bytes_checksum = crate::patch::Checksum::new(bytes);
      let patch_checksum = writer.checksum();

      if &bytes_checksum != patch_checksum {
         return Err(crate::patch::PatchError::ChecksumMismatch{
            found    : bytes_checksum,
            expected : patch_checksum.clone(),
         });
      }

      writer.build_patch(bytes)?;
      
      return Ok(());
   }

   unsafe fn patch_write_unchecked<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> crate::patch::Result<()>
   where Wt: crate::patch::Writer<Mr>,
         Mr: RangeBounds<usize>,
   {
      let address_range = self.offset_range_to_address_range(
         writer.memory_offset_range(),
      )?;

      let mut editor = crate::sys::memory::MemoryEditor::open_read_write(
         address_range,
      )?;

      let bytes = editor.as_bytes_mut();

      writer.build_patch(bytes)?;

      return Ok(());
   }

   unsafe fn patch_create<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> crate::patch::Result<Self::Container>
   where Wt: crate::patch::Writer<Mr>,
         Mr: RangeBounds<usize>,
   {
      let address_range = self.offset_range_to_address_range(
         writer.memory_offset_range(),
      )?;

      let mut editor = crate::sys::memory::MemoryEditor::open_read_write(
         address_range.clone(),
      )?;

      let bytes = editor.as_bytes_mut();

      let bytes_checksum = crate::patch::Checksum::new(bytes);
      let patch_checksum = writer.checksum();

      if &bytes_checksum != patch_checksum {
         return Err(crate::patch::PatchError::ChecksumMismatch{
            found    : bytes_checksum,
            expected : patch_checksum.clone(),
         });
      }

      let container = Self::Container{
         address_range  : address_range,
         old_bytes      : bytes.to_vec(),
      };

      writer.build_patch(bytes)?;

      return Ok(container);
   }

   unsafe fn patch_create_unchecked<Wt, Mr>(
      & mut self,
      writer : & Wt,
   ) -> crate::patch::Result<Self::Container>
   where Wt: crate::patch::Writer<Mr>,
         Mr: RangeBounds<usize>,
   {
      let address_range = self.offset_range_to_address_range(
         writer.memory_offset_range(),
      )?;

      let mut editor = crate::sys::memory::MemoryEditor::open_read_write(
         address_range.clone(),
      )?;

      let bytes = editor.as_bytes_mut();

      let container = Self::Container{
         address_range  : address_range,
         old_bytes      : bytes.to_vec(),
      };

      writer.build_patch(bytes)?;

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
      ).expect("Failed to restore patched bytes");

      unsafe{editor.as_bytes_mut().copy_from_slice(&self.old_bytes)};

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
   /// process visible to the user.
   pub fn all(
   ) -> Result<Self> {
      let proc = crate::sys::process::ProcessSnapshot::all()?;

      let mut hash = HashMap::with_capacity(proc.len());
      for proc in proc {
         let proc = ProcessSnapshot{
            snapshot : proc,
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

   /// Tries to remove a process from
   /// the list by its executable file
   /// name.
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
   /// snapshot.
   pub fn all(
      process_snapshot  : ProcessSnapshot,
   ) -> Result<Self> {
      let list = crate::sys::process::ModuleSnapshot::all_within(
         &process_snapshot.snapshot,
      )?;

      let mut hash = HashMap::with_capacity(list.len());
      for module in list {
         let module = ModuleSnapshot{
            snapshot : module,
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

   /// Tries to removes a module from the
   /// list by its executable file name.
   pub fn remove_by_executable_file_name(
      & mut self,
      executable_file_name : & str,
   ) -> Option<ModuleSnapshot> {
      return self.modules.remove(executable_file_name);
   }

   /// Tries to find a module snapshot
   /// by its executable file name.
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

