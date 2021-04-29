<p align="center">
<img style="width: 200px; padding: 10px;" src="docs/images/dtasm_logo.png">
</p> 
<p align="center">
<img style="width: 250px; padding: 10px;" src="docs/images/dtasm.svg">
</p>

# Digital Twin Assembly

Digital Twin Assembly (_dtasm_) is a binary interface (ABI) for executable numerical simulators compiled into WebAssembly modules. Such simulators implement methods for dynamically stepping forward in discrete time steps, where in each step, the simulator takes values for its declared input variables, performs the time step calculation, and emits values for its declared output variables at the end of the time step. 
_dtasm_ is based on WebAssembly, Wasi and FlatBuffers. See [here](docs/README.md) for a specifiation of the ABI. An in-depth description of _dtasm_ with a discussion of advantages and drawbacks can be found [here]. 

## Contents
This repository contains various implementations of _dtasm_ runtimes and modules for demonstration purposes. It is not meant as a finished product or reference implementation of the dtasm interface, but rather as a starting point for compiling and running numerical simulators as WebAssembly modules.

The main artifacts of this repository are: 
- [_dtasmtime_](runtime/dtasmtime) - A Rust library implementing a runtime for _dtasm_ modules based on [Wasmtime](http://wasmtime.dev). An example command-line program using this library for loading and executing _dtasm_ modules can be found [here](runtime/examples/dtasmtime_rs). 
- [_dtasmtime-c-api_](runtime/dtasmtime-c-api) - C api for _dtasmtime_, allowing the library to be called from C/C++, as well as other languages with C interop capabilities. An example command-line program in C that uses this library can be found [here](runtime/examples/dtasmtime_c). 
- [_dpend-cpp_](module/dpend_cpp) - Exemplary dtasm module with an implementation of a Runge-Kutta solver for a double pendulum based on an example by [M. Wheatland](http://www.physics.usyd.edu.au/~wheat/dpend_html/). 
- [_dpend-rs_](module/dpend_rs) - Same double pendulum example written in Rust. 

## Prerequisites
- Linux on x86_64 or aarch64 (tested on Ubuntu 20.04)
- Rust, cargo and `wasm32-wasi` target for the active Rust toolchain (if using `rustup`, install by running `rustup target add wasm32-wasi`)
- C/C++ compiler (e.g. gcc/g++)
- GNU make
- cmake
- [WASI SDK 12](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-12) installed into the default location at `/opt/wasi-sdk`

## Build 
Be sure to clone the repo including submodules (`git clone --recurse-submodules ...`). A top-level `Makefile` is provided for convenience to build all components. Running
```
make
```
builds the Rust CLI example and the Rust double pendulum module. You can execute the module by running `make run-rs` afterwards. 
The same can be done for the C/C++ example by runnig `make cpp` followed by `make run-c`. 
All components are build by running
```
make all
```

# License

This project is released under [The MIT License (MIT)](LICENSE).
