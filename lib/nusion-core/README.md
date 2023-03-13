# nusion-core
The main modding library for the Nusion project.

### Usage
Add the crate as a dependency in Cargo.toml by
specifying the filesystem path to the crate source
directory.
```
# Cargo.toml crate dependency example
[dependencies]
nusion-core = { path = "<repository path>/lib/nusion-core" }
```
To write a dynamic library for a specific game, make sure
to build as a library crate and enable dynamic linking in
Cargo.toml.
```
# Cargo.toml dynamic library specification example
[lib]
crate-type = ["cdylib"]
```
For further usage such as declaring an entrypoint, please
refer to the crate documentation.

### Documentation
Generate HTML documentation using Cargo and view crate
documentation in your web browser.
```
cargo doc --open
```

