<p align="center">
<img style="width: 200px; padding: 10px;" src="docs/images/dtasm_logo.png">
</p> 
<p align="center">
<img style="width: 250px; padding: 10px;" src="docs/images/dtasm.svg">
</p>

# Digital Twin Assembly

Digital Twin Assembly (_dtasm_) is a binary interface (ABI) for executable numerical simulators compiled into WebAssembly modules. Such simulators implement methods for dynamically stepping forward in discrete time steps, where in each step, the simulator takes values for its declared input variables, performs the time step calculation, and emits values for its declared output variables at the end of the time step. 
_dtasm_ is based on WebAssembly, WASI and FlatBuffers. See [here](docs/README.md) for a specification of the ABI. An in-depth description of _dtasm_ with a discussion of advantages and drawbacks has been [presented](https://2021.international.conference.modelica.org/proceedings/papers/Modelica2021session6A_paper3.pdf) at the _Modelica 2021_ conference. 

## Contents
This repository contains various implementations of _dtasm_ runtimes and modules for demonstration purposes. It is not meant as a finished product or reference implementation of the _dtasm_ interface, but rather as a starting point for compiling and running numerical simulators as WebAssembly modules.

The main components of this repository are: 
- [_dtasmtime_](runtime/dtasmtime) - A Rust library implementing a runtime for _dtasm_ modules based on [Wasmtime](http://wasmtime.dev). An example command-line program using this library for loading and executing _dtasm_ modules can be found in [`runtime/examples/dtasmtime_rs`](runtime/examples/dtasmtime_rs). 
- [_dtasmtime-c-api_](runtime/dtasmtime-c-api) - C API for _dtasmtime_, allowing the library to be called from C/C++, as well as other languages with C interop capabilities. An example command-line program in C that uses this library can be found in [`runtime/examples/dtasmtime_c`](runtime/examples/dtasmtime_c). 
- [_dtasm3_](runtime/dtasm3) - A lightweight C++ runtime for _dtasm_ modules based on [wasm3](https://github.com/wasm3/wasm3). Due to efficiency and low footprint of wasm3, this runtime is a good fit for smaller hardware or an MCU (Raspberry Pi, ESP32). An example for using this runtime can be found in [`runtime/examples/dtasm3_main`](runtime/examples/dtasm3_main).
- [_dpend_cpp_](module/dpend_cpp) - Exemplary _dtasm_ module implementing a double pendulum simulator (based on example code by [M. Wheatland](http://www.physics.usyd.edu.au/~wheat/dpend_html/). 
- [_dpend_rs_](module/dpend_rs) - Same double pendulum simulator written in Rust. 
- [_add_rs_](module/add_rs) - Simple test module adding (or concatenating, *and*ing) two inputs of each type. 

## Getting started - Dev Containers
The easiest way to obtain all necessary dependencies for building and running the code in this repo is using _Visual Studio Code_ with the _Dev Containers_ extension (this will also need an installation of _Docker_). Clone the repo including submodules (`git clone --recurse-submodules ...`), then open the repo folder in VS Code. It will prompt you to open inside a Dev Container, select 'Reopen in Dev Container' (if it does not prompt, select 'View' -> 'Command Palette' -> 'Dev Containers: Rebuild and Reopen in Container'). Initial building of the Dev Container and downloading of toolchains will take quite some time, wait until the process is finished and then proceed with [build](#build). 

## Getting Started - Manual
The following prerequisites are needed:
- Linux or Windows OS on x86_64 or aarch64 platform (except for dtasm3 which also supports many smaller platforms, see [here](https://github.com/wasm3/wasm3/blob/main/docs/Hardware.md))
- Rust, cargo and `wasm32-wasi` target for the active Rust toolchain (if using `rustup`, this can be installed by running `rustup target add wasm32-wasi`)
- C/C++ compiler (e.g. gcc/g++)
- GNU make
- cmake
- [WASI SDK](https://github.com/WebAssembly/wasi-sdk/releases) (version >= 12) installed into the default location at `/opt/wasi-sdk`
- `xxd` tool (e.g. `sudo apt install xxd`).

Once the prerequisites are installed, be sure to clone the repo including submodules (`git clone --recurse-submodules ...`), then proceed with the [build section](#build). 

## Build 
A top-level `Makefile` is provided for convenience to build the components. Running
```
make deps
```
will first build the FlatBuffers compiler `flatc`, then create all needed FlatBuffer stubs and buffers required by the components. Running 
```
make
```
builds _dtasmtime_, the Rust command-line example and the Rust double pendulum module. You can execute the module by running `make run-rs` afterwards. 

The same can be done for the C/C++ API and double pendulum simulator by running `make cpp` followed by `make run-c`. Finally, some examples using _dtasm3_ can be run with `make run-dtasm3`. 

# License
This project is released under the [MIT License](LICENSE).
