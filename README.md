# Nusion Collection
A work-in-progress collection of libraries, frameworks, and tools
for creating game modifications and modding frameworks.



### Project Layout
Each crate has their own readme and documentation.  For more
information, view the documentation contained inside each
crate's directory.

#### nusion
Main game modding library/framework

#### nusion-sys
System abstractions used by nusion.
<b>This should never be used directly as a dependency.</b>
Instead, you should use the safe(r) high-level wrappers
found in the nusion library crate.

#### nusion-macros
Macro implementations incorporated into nusion.
<b>This should not be used directly as a dependency.</b>
Instead, you should use the nusion crate, which includes
this crate bundled with it.

#### nusion-sys-macros
System abstraction macros used by nusion-macros.
<b>This should never be used directly as a dependency.</b>
Instead, you should use the safe(r) macros found in
nusion-macros.



### Build Requirements
 - A Linux installation
 - [Rustup](https://rustup.rs/)



### Building
#### Windows
 - Install [Windows Subsystem for Linux](https://learn.microsoft.com/en-us/windows/wsl/about)
 - Navigate to the folder where the repository will be cloned
 ```
 cd /mnt/c/[path-to-folder]
 ```
 - Continue to Linux compilation guide

#### Linux
 - Clone the repository, either by downloading the zip file off Github or by using [git](https://git-scm.com/about)
 - Enter the repository
 ```
 cd nusion
 ```
 - Ensure the correct toolchains for your target platforms are installed.
 ```
 # Example: Add Windows x86_64 as a target
 rustup target add x86_64-pc-windows-gnu
 ```
 - Compile the project with cargo
 ```
 cargo build --release
 ```
 - The output binaries will be under 'target/[target-platform]/release/'



### About
This started off as an adaptation of a cheat client I made for
Deep Rock Galactic called 'Fusion' for the Rust programming language.
I was curious how viable Rust was for game hacking considering all the
compile-time and run-time safety checks.  Eventually this would evolve
to become a standalone general-purpose modding utility aimed at video
games, but can theoretically be used for any native executable.  This
switch in direction and branding was done for two reasons.  The first
reason is to more accurately reflect the potential for this repository.
It's not meant exclusively for cheating in video games; it can be used
for many other purposes.  The second reason is to prevent showing up in
search results for skids trying to find cheats for online games and
cause mayhem.  I don't want to be the one enabling griefers to go and
ruin online games.  I understand this library makes the lives of these
people easier as this library takes care of much of the mess and low-level
knowledge of game hacking, but actual hackers would just do it themselves
anyways.  It slightly lowers the barrier of entry for cheaters while
providing a modding suite for all Rustaceans.  I believe this is a worthy
tradeoff.



### Timeline
 * January 29th, 2023 - Development started by Sinisig

