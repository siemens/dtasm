# Dtasm3

_Dtasm3_ is a lightweight runtime for _dtasm_ modules that is based on the WebAssembly interpreter [wasm3](https://github.com/wasm3/wasm3). While raw execution performance of such interpeters is considerably lower compared to JIT runtimes like [Wasmtime](http://wasmtime.dev), interpreters are also more slim since they do not need compilation and code generation components. _wasm3_'s implementation consists of only a few C source files and carries no external dependencies, such that _dtasm3_ when compiled is only a hew hundred kilobytes in size and hence is a perfect fit for small targets like the ESP32 or other similar MCUs. 

## How to

_Dtasm3_ is implemented as a C++ library and its interface is defined in [dtasm3.h](include/dtasm3.h). An example program for using the library can be found in [dtasm3_main](../examples/dtasm3_main). To compile and run this example, follow these steps from the top level of the repository:
```
make deps
make run-dtasm3
```
This will build _dtasm3_main_ and all the _dtasm_ modules, and then invoke _dtasm3_main_ with each one of the modules in turn. To build _dtasm3_main_ separately, follow these steps:
```
make deps
cd runtime/examples/dtasm3_main
mkdir Debug && cd Debug
cmake ..
cmake --build .
```
After compilation, _dtasm3_main_ can be invoked as follows: 
```
./dtasm3 <dtasm_module.wasm> --tmin 0.0 --tmax 10.0 --n_steps 100
```
The `tmin` and `tmax` parameters specify start time and end time of the simulation, while `n_steps` is the number of (equally spaced) timesteps. The resulting values of the output variables are written to the terminal in every timestep. 