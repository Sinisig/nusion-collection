//! <h1 id=  nusion_lib>
//! <a href=#nusion_lib>
//! Nusion Modding Library
//! </a></h1>
//! 
//! Create mods for video games effortlessly
//! and safely using the power of Rust!
//!
//! <h2 id=  nusion_lib_guide>
//! <a href=#nusion_lib_guide>
//! Introductory guide
//! </a></h2>
//!
//! <h5 id=  nusion_lib_guide_crate_setup>
//! <a href=#nusion_lib_guide_crate_setup>
//! Crate setup
//! </a></h5>
//!
//! First, you will need to create a
//! library crate which outputs a dynamic
//! shared library.  This is done by creating
//! a new Cargo project as usual.  You then need
//! to add nusion-lib as a dependency and mark
//! the library type as a dynamic library.
//!
//! ```
//! [package]
//! name     = "my_first_mod"
//! version  = "0.1.0"
//! edition  = "2021"
//!
//! [lib]
//! crate-type = "cdylib"
//! 
//! [dependencies]
//! nusion-lib = "*"
//! ```
//!
//! This will create a library crate which
//! outputs to target/\[profile\]/\[name\].dll
//! for Windows or target/\[profile\]/\[name\].so
//! for Linux.
//!
//! <h5 id=  nusion_lib_guide_entrypoint>
//! <a href=#nusion_lib_guide_entrypoint>
//! Declaring an entrypoint
//! </a></h5>
//!
//! Declaring an entrypoint is so simple,
//! you know how to do it already!
//!
//! ```
//! #[nusion_lib::main]
//! fn main() {
//!    println!("Hello, world!");
//! }
//! ```
//!
//! This declaration can by anywhere you please
//! within your crate, but it can never be defined
//! more than once.  Currently, attempting to
//! declare more than one main function will cause
//! an avalanche of linker errors.  It is best to
//! have a clear project structure where it is
//! obvious where your entrypoint is declared.
//!
//! <h5 id=  nusion_lib_guide_process_filtering>
//! <a href=#nusion_lib_guide_process_filtering>
//! Only execute when loaded into a specific process
//! </a></h5>
//!
//! You <i>could</i> manually check the process
//! name and return from <code>main</code> if
//! it doesn't match your intended process, but
//! that would be tedious and error-prone to
//! do for every project.  Instead, you can
//! specify a <b>process whitelist</b> inside
//! the <code>main</code> attribute macro.
//!
//! ```
//! #[nusion_lib::main("hl2.exe")]
//! fn main() {
//!    println!("Hello, world!");
//! }
//! ```
//!
//! It is also possible to allow more than
//! one process name.
//!
//! ```
//! #[nusion_lib::main("hl2.exe", "csgo.exe", "paint.exe")]
//! fn main() {
//!    println!("Hello, world");
//! }
//! ```
//!
//! You still must have only one entrypoint function,
//! even if the excess definition has a different
//! process whitelist.
//!
//! <h5 id=  nusion_lib_guide_return_errors>
//! <a href=#nusion_lib_guide_return_errors>
//! Return errors from your entrypoint
//! </a></h5>
//!
//! The <code>main</code> macro does not only
//! accept entrypoints with no return type.
//! It can also except a <code><a href=
//! https://doc.rust-lang.org/std/result/enum.Result.html>Result</a></code>
//! return type with the <code>Ok</code> variant
//! being the unit type, represented with a
//! "<code>()</code>", and the <code>Err</code> variant
//! being a type which implements the <code><a href=
//! https://doc.rust-lang.org/std/error/trait.Error.html>Error</a></code>
//! trait.
//!
//! ```
//! #[nusion::main]
//! fn main() -> Result<(), MainError> {
//!    let sanity = 0;
//!
//!    if sanity == 0 {
//!       return Err(MainError{});
//!    }
//!
//!    return Ok(());
//! }
//!
//! #[derive(Debug)]
//! struct MainError;
//! 
//! impl std::fmt::Display for MainError {
//!    fn fmt(& self, stream : & mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//!       return write!(stream, "");
//!    }
//! }
//!
//! impl std::error::Error for MainError {
//! }
//! ```
//!
//! You may also use a trait object wrapped
//! inside a <code><a href=
//! https://doc.rust-lang.org/std/boxed/struct.Box.html>Box</a></code>
//! to allow returning of multiple different
//! error types.
//!
//! ```
//! #[nusion::main]
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    let some_crazy_number        = -1;
//!    let some_less_crazy_number   = u32::try_from(some_crazy_number)?;
//!
//!    std::fs::write(
//!       std::path::Path::new("foobar.txt"),
//!       &some_less_crazy_number.to_string(),
//!    )?;
//!
//!    return Ok(());
//! }
//! ```
//!
//! <h5 id=  nusion_lib_guide_environment>
//! <a href=#nusion_lib_guide_environment>
//! Know your environment
//! </a></h5>
//!
//! If all we could do is write goofy
//! <code>println!</code> statements, our
//! "mod" would be pretty useless!  We need
//! to be able to access process memory and
//! inject our own code. In order to do that,
//! we need to access the provided <b>nusion
//! environment</b>. The environment contains
//! access to many useful interfaces for managing
//! and interacting with the loaded process in
//! an OS-independent way.
//!
//! ```
//! #[nusion::main("hl2.exe")]
//! fn main() {
//!    // Change the default console title
//!    nusion_lib::env_mut!().console_mut().set_title(
//!       "Hello Modding World Console",
//!    );
//!
//!    // Access the module for our target
//!    // process "hl2.exe"
//!    let mut game = nusion_lib::env_mut!()
//!      .modules_mut()
//!      .find_by_executable_file_name("hl2.exe")
//!      .unwrap();  // You should properly handle this in real projects
//! }
//! ```
//!
//! Be aware that the <code>env!</code> macro
//! return a lock from a <code><a href=
//! https://doc.rust-lang.org/std/sync/struct.Mutex.html>Mutex</a></code>,
//! so you should make sure to release the
//! lock as soon as you can.  Failing to do
//! so can lead to anything from poor performance
//! all the way to deadlocks and freezes.
//! You should obtain and immediately free
//! the lock as needed instead of holding it
//! for extended periods of time.
//!
//! <h5 id=  nusion_lib_guide_basic_patching>
//! <a href=#nusion_lib_guide_basic_patching>
//! Read and patch the game's memory
//! </a></h5>
//!
//! Now that we can access the game module
//! through the environment, how do we actually
//! patch the game module?  This is accomplished
//! through three traits: <code>Patch</code>,
//! <code>Reader</code>, and <code>Writer</code>.
//! The <code>Patch</code> trait is implemented
//! on types which have memory which can be
//! manipulated.  This trait is responsible
//! for applying memory actions to the process.
//! The <code>Reader</code> trait is implemented
//! on types responsible for storing information
//! on how to read and interpret memory from a
//! <code>Patch</code> type.  The <code>Writer</code>
//! trait is implemented on types responsible for
//! storing information on how to apply a memory
//! patch and writing it to memory.  In our case,
//! let's say we want to read a health value and
//! then write a new health value if it's less
//! than some threshold...so let's do it!
//!
//! ```
//! #[nusion::main("hl2.exe")]
//! fn main() {
//!    // Our Reader struct for reading the health value
//!    const READER_HEALTH : nusion_lib::patch::reader::Item<i32> = nusion_lib::patch::reader::Item<i32>{
//!       marker              : Default::default(),
//!       memory_offset_range : 0x7FFF1337..0x7FFF133B,
//!    };
//!
//!    // Our Writer struct for writing a new health value
//!    const WRITER_HEATLH : nusion_lib::patch::writer::Item<'_, i32> = nusion_lib::patch::writer::Item<'_, i32>{
//!       memory_offset_range : 0x7FFF1337..0x7FFF133B,
//!       checksum            : nusion_lib::patch::Checksum::from(0),
//!       item                : &100,
//!    };
//!
//!    // Change the default console title
//!    nusion_lib::env_mut!().console_mut().set_title(
//!       "Hello Modding World Console",
//!    );
//!
//!    // Access the module for our target
//!    // process "hl2.exe"
//!    let mut game = nusion_lib::env_mut!()
//!      .modules_mut()
//!      .find_by_executable_file_name("hl2.exe")
//!      .unwrap();  // You should properly handle this in real projects
//!      
//!    // Import the Patch trait so we can modify process memory
//!    use nusion_lib::patch::Patch;
//!
//!    // Read the player's current health using the above Reader
//!    let health = unsafe{game.patch_read(&READER_HEALTH)}?;
//!
//!    // If it's below half HP, update it to max HP
//!    if health < 50 {
//!       // We don't care about the integrity of the
//!       // data overwritten, so we ignore the checksum
//!       unsafe{game.patch_write_unchecked(&WRITER_HEALTH)}?;
//!    }
//! }
//! ```
//!
//! We create our reader and writer structs
//! which contain offsets into the module's
//! process memory.  The writer also stores
//! our desired new health value.  After accessing
//! the game module, we read the health value
//! from memory.  We check if the read value
//! is below half HP, and if it is we write
//! our new HP value back to memory.  Two things
//! need clarification: Why the checksum and WTF
//! is the "marker"?
//!
//! The checksum is used in the case where we
//! expect some data to be in memory and we want
//! to patch it.  If the data changes due to
//! offsets changing, data corruption, or an update
//! changing the code, our patch will no longer
//! work!  We compare the expected checksum against
//! the freshly calculated checksum of the memory
//! region. If they don't match, we know somethin
//! has gone wrong and return an error.  If we
//! don't care about the checksum value, such as
//! this case where the value could change under
//! normal circumstances, we can use one of the
//! <code>unchecked</code> variants of the <code>Patch</code>
//! methods.
//!
//! The reason for this weird "marker" is to
//! impose a trait bound on the struct.  If we
//! don't have this marker, we couldn't compile
//! because the trait parameter would go unused
//! inside the struct definition.  It's a bit
//! of a hacky workaround, but it works.
//!
//! In addition, it should be noted the use
//! of <code>unsafe</code> here.  Reading and
//! writing arbitrary memory with any type we
//! want and with no respect to thread synchronization
//! is <b>extremely</b> unsafe!  We need to be
//! very careful our code is "sound" when writing
//! our patches.  If the memory offsets are slightly
//! off, our types aren't valid for their locations,
//! or anything else, we can easily crash our
//! program with a memory violation or garbage
//! data which can easily generate <i>undefined behavior</i>.
//! For more information on safety, refer to the
//! documentation for the <code>Patch</code> trait.
//!
//! <h5 id=  nusion_lib_guide_advanced_patching>
//! <a href=#nusion_lib_guide_advanced_patching>
//! Advanced patching
//! </a></h5>
//!
//! Reading and writing health values is fun, but
//! gets boring quick.  What if we want to modify
//! the game to execute our own code?  In that case,
//! we can use two provided <code>Writer</code> structs:
//! <code>Hook</code> and <code>Asm</code>.
//!
//! ```
//! #[nusion::main("hl2.exe")]
//! fn main() {
//!    // Our Writer struct for hooking the damage function
//!    const HOOK_DAMAGE : nusion_lib::patch::writer::Hook = nusion_lib::patch::writer::Hook{ 
//!       memory_offset_range  : 0x9FCD4000..0x9FCD4010,
//!       checksum             : nusion_lib::patch::Checksum::from(0xFC204AFD),
//!       hook                 : nusion_lib::hook!("
//!          // Execute the instructions the hook overwrote
//!          mov   edi,[rcx+0x40]
//!          sub   edi,[rcx+0x44]
//!          mov   [rcx+0x40],edi
//!
//!          // Align the stack to a 16-byte boundary and preserve
//!          // important volatile registers
//!          push  rcx
//!
//!          // Load the pointer to the health
//!          // value and call our hook closure
//!          lea   rcx,[rcx+0x40]
//!          call  {target}
//!
//!          // Restore the stack and the saved
//!          // volatile register
//!          pop   rcx
//!
//!          // Return to the patched function
//!          ret
//!       ", |health_value : & mut i32| {
//!          *health_value = 100;
//!          println!("We take control of the damage function here! New health: {health_value}");
//!       }),
//!    };
//!
//!    // Change the default console title
//!    nusion_lib::env_mut!().console_mut().set_title(
//!       "Hello Modding World Console",
//!    );
//!
//!    // Access the module for our target
//!    // process "hl2.exe"
//!    let mut game = nusion_lib::env_mut!()
//!      .modules_mut()
//!      .find_by_executable_file_name("hl2.exe")
//!      .unwrap();  // You should properly handle this in real projects
//!      
//!    // Import the Patch trait so we can modify process memory
//!    use nusion_lib::patch::Patch;
//!
//!    // Apply our damage function hook so we can take control
//!    // We store the overwritten bytes and automatically restore
//!    // the patched region to its original value when the patch
//!    // result goes out of scope and is dropped.
//!    let _patch_result = unsafe{game.patch_create(&HOOK_DAMAGE)}?;
//!
//!    // Sleep so we can see the fruits of our labor
//!    std::thread::sleep(std::time::Duration::from_secs(30));
//!
//!    // _patch_result is dropped, the hook is removed and
//!    // the overwritten code is restored to what it should be.
//! }
//! ```
//!
//! The method to the madness is largely the same
//! as the previous example.  If you want to learn
//! about the crazy syntax for generating the hook
//! function, check out the documentation for the
//! <code>hook!</code> macro.
//!
//! In this case, instead of using the
//! <code>patch_write_unchecked</code> method
//! to write to the game's memory, we use the
//! <code>patch_create</code> method.  This method
//! checks the checksum value against what is stored
//! in memory.  We want this behavior in this case
//! because we expect static, immutable data at this
//! location.  If it has changed, our hook could break.
//! Second the overwritten bytes are stored in the
//! returned value and are automatically restored
//! when the container is dropped.  This can lead to
//! a head-banging bug where the patch seemingly reports
//! success, but appears to never apply.  What happens
//! is if we don't give the returned container a real
//! binding, the compiler drops it immediately after the
//! function call.  This leads to the above effect.
//! To prevent this, we must assign a named variable
//! binding to the container.  This container will
//! rarely be used after initialization, so it is
//! fine to prefix it with an underscore to prevent
//! unused variable warnings.
//!
//! Awesome, so we can now take control of the game's
//! code!  Notice how this hook involves a function
//! call, though?  In very performance-critical sections
//! of code, the overhead of calling your assembly
//! intermediate and then <i>another</i> call to your
//! hook closure may have a significant impact. 
//! In some cases, it may be possible to fit your
//! entire hook inside the memory region <i>without</i>
//! introducing the overhead of a function call.
//! Luckily, we can "inline" our hook directly
//! into the memory region using the <code>Asm</code>
//! struct!
//!
//! ```
//! #[nusion::main("hl2.exe")]
//! fn main() {
//!    // Our Writer struct for patching the damage function
//!    const ASM_DAMAGE : nusion_lib::patch::writer::Asm = nusion_lib::patch::writer::Asm{ 
//!       memory_offset_range  : 0x9FCD4000..0x9FCD4010,
//!       checksum             : nusion_lib::patch::Checksum::from(0xFC204AFD),
//!       hook                 : nusion_lib::asm_bytes!("
//!          // Overwrite the damage code and replace
//!          // with a direct move of 100hp
//!          mov   [rcx+0x40],100
//!       "),
//!    };
//!
//!    // Change the default console title
//!    nusion_lib::env_mut!().console_mut().set_title(
//!       "Hello Modding World Console",
//!    );
//!
//!    // Access the module for our target
//!    // process "hl2.exe"
//!    let mut game = nusion_lib::env_mut!()
//!      .modules_mut()
//!      .find_by_executable_file_name("hl2.exe")
//!      .unwrap();  // You should properly handle this in real projects
//!      
//!    // Import the Patch trait so we can modify process memory
//!    use nusion_lib::patch::Patch;
//!
//!    // Apply our damage function patch to effectively cancel
//!    // out the damage and lock our health at 100
//!    let _patch_result = unsafe{game.patch_create(&ASM_DAMAGE)}?;
//!
//!    // Sleep so we can experience god mode
//!    std::thread::sleep(std::time::Duration::from_secs(30));
//!
//!    // _patch_result is dropped, the hook is removed and
//!    // the overwritten code is restored to what it should be.
//! }
//! ```
//!
//! Again, the syntax of the ASM code generation
//! won't be covered here, as it's pretty involved.
//! For more information, see the documentation for
//! the <code>asm_bytes!</code> macro.
//!
//! Not only is our code faster, it is also simpler
//! since we only need the one chunk of ASM!  This
//! can be desireable when there are dozens of patches
//! which may start to impact code performance and
//! cleanliness due to lots of boilerplate to use the
//! hook method.
//!
//! <h5 id=  nusion_lib_guide_conclusion>
//! <a href=#nusion_lib_guide_conclusion>
//! Conclusion
//! </a></h5>
//!
//! This library is very flexible and can allow
//! for very powerful modding frameworks to be
//! developed.  If there is a different type of
//! patch method which can't be implemented using
//! the provided types, you can implement your own
//! using the <code>Writer</code> trait.  If you
//! want to patch multiple modules at once, you
//! can also do that.  It's all up to you.  What
//! will you build?

// Internal crate re-exports
use nusion_lib_proc  as proc;
use nusion_lib_sys   as sys;

// Public modules
pub mod console;
pub mod environment;
pub mod macros;
pub mod patch;
pub mod process;

// Public module re-exports
pub use proc::*;

// Public-internal items
pub mod __private {
   //! This module should never be used.
   //! This only exists to allow certain
   //! macros to function properly.  There
   //! is no excuse good enough for accessing
   //! this module in your code, no matter
   //! what you say.
   
   use super::*;

   pub use sys::        __osapi        as osapi;
   pub use crate::      __build_entry  as build_entry;
   pub use environment::__start_main   as start_main;
   pub use sys::        build_entry    as sys_build_entry;
}

