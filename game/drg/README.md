# nusion-game-drg
A modding framework implementation for Deep Rock Galactic.

### Warning
The generated binary is a DLL which must be loaded
into the game process using a DLL injector.  <b>You
can get your steam account banned</b> if you leave the
your DLL injector open when playing other games. Currently,
Deep Rock Galactic has no anti-cheat system, but <b>other
games have forms of anti-cheat which can detect DLL injectors
and ban your account</b>.

### Usage
Inject the DLL compiled using Cargo into the game process
using your DLL injector of choice.  The modding API will
load and show a console window offering modding options.

### Documentation
Generate HTML documentation using Cargo and view crate
documentation in your web browser.
```
cargo doc --open
```

