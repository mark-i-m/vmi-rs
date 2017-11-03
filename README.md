# vmi-rs

A simple Rust wrapper around LibVMI.

It includes a small safe Rust interface (that I have adding to as needed).

It also uses bindgen and re-exports the raw symbols in case I need to do something that wrapper does not already wrap.

### Requirements

- Bindgen requires `libclang`: `apt-get install llvm-3.9-dev libclang-3.9-dev clang-3.9`

- LibVMI can be installed via [these instructions](http://libvmi.com/docs/gcode-install.html).
