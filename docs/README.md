# Dtasm Specification

Digital Twin Assembly (_dtasm_) describes a module format and an interface for self-contained executable simulation modules. A dtasm module consists of a single WebAssembly (Wasm) file compliant with the [core WebAssembly 1.0 specification](https://www.w3.org/TR/wasm-core-1/) that implements a certain binary interface (ABI). This binary interface consists of a set of exported functions whose signature and semantics are described in this document. A dtasm module also needs to export a memory block that is utilized for sharing data between the host and the module instances. A dtasm module may import certain functions from the host for the purpose of logging runtime information, this is described in [#Logging].

## ABI
Functions exported from or imported into a WebAssembly module need to have a signature in terms of the basic data types known to Wasm (i32, i64, f32, f64). Since interfacing with dynamic simulation modules requires echange of more complex data structures, dtasm utilizes the Wasm linear memory in order to pass input arguments to functions and return output values. For description of interface data structures as well as for the binary representation used in linear memory, the format of the [FlatBuffers library](https://google.github.io/flatbuffers/) has been adopted. FlatBuffers implements serialization of complex data structures into an efficient and highly performant binary representation, while at the same time being very lightweight in terms of resources and dependencies. FlatBuffers data structures are described using a domain specific language (DSL) by writing a so-called FlatBuffers schema. Schemas are then read by the FlatBuffers compiler _flatc_ and turned into code for serializing and deserializing of binary buffers in various programming languages. 

The functions used in the dtasm interface typically have a signature similar to the following C-function declaration:
```C
uint8_t init(uint8_t* in_p, uint in_len, uint8_t* out_p, uint out_max_len)
```
`in_p` here is a pointer to an input buffer of length `in_len`, while `out_p` holds a pointer to an (already allocated) output buffer of size `out_max_len`. Pointers to linear memory are simply offsets from the start of the memory block, so that in WebAssembly text format (WAT) this signature translates to 
```Rust
func init(param i32 i32 i32 i32) (result i32)
```
The implementation of the function then a FlatBuffer of the input type specified for this function (`InitReq`) from a byte array starting at offset `in_p` with length `in_len`. It then calculates the output, turns it into a FlatBuffer of the output type (`InitRes`) and copies the buffer to `out_p` offset in linear memory. Since the precise size of the output returned from the function is not known ahead of its invocation, `out_max_len` is usually a sensibly chosen default size. If the length `out_max_len` does not suffice to hold the output, the function returns a non-zero return value indicating the number of bytes needed to hold the output buffer. The runtime should then invoke the same function again with a sufficiently large output buffer to retrieve the result (and all functions returning outputs of non-trivial size are idempotent). 

For allocations of buffers inside the Wasm linear memory, an heap allocator function 
```C
uint8_t* alloc(size_t len)
```
and a corresponding de-allocation function
```C
void dealloc(uint8_t* p)
```
are exported by each dtasm module. This allocator is used by the host as well as by the module itself. Dtasm follows the common pattern "allocator is responsible for freeing" and hence most of the exchanged buffers are allocated and freed by the host by invoking the corresponding exported module functions. 

## Semantics
The simulation interface of a dtasm module is given by the exported functions 
```rust
init(InitReq) -> StatusRes
doStep(DoStepReq) -> DoStepRes
getValues(GetValuesReq) -> GetValuesRes
setValues(SetValuesReq) -> StatusRes
resetStep(ResetStepReq) -> StatusRes
getModelDescription(Void) -> ModelDescription
```
The `getModelDescription` function serves as a way to retrieve information about the module such as inputs, outputs and local variables. Since dtasm modules are valid Wasm modules, they cannot package such a model description as an explicit file, but rather make it available as the result of a function call to `getModelDescription`. Instantiation of a simulation module is handled by creating an instance of the Wasm module (something that is covered by the WebAssembly specification) and hence no specific interface is needed for this. Once an instance has been created, `init` initializes the simulation with given initial values for local and output variables. After this initialization phase, the so-called cyclic phase begin, where in each cycle, the simulation proceeds from discrete time $t_i$ to $t_{i+1}$ by subsequent calls to
1. `setValues` to set values for the module's input variables at time $t_i$, 
2. `doStep` to simulated from $t_i$ to $t_{i+1}$, 
3. `getValues` to retrieve values of output variables at time $t_{i+1}$.
If the return value of `doStep` indicates that the simulation of the time step was not successful (by returning status _discard_, usually indicating that a shorter time step is necessary), a call to `resetStep` resets the time variable back to $t_i$ (it is expected that not all simulation modules can support this operation, however this operation can be performed by the Wasm runtime as well by snapshotting linear memory in each time step and reloading the last snapshot on detection of a _discard_ return code).
The data structures used as arguments and return values to these functions can be found in the [FlatBuffers schema](dtasm.fbs) and a detailed description is given in [Data Structures](#interface-data-structures). 

## Model Description
Each dtasm module carries a model description that is retrieved by invoking the `getModelDescription` export as described in [Semantics](#semantics). Since FlatBuffers format is heavily used for the ABI of dtasm, the model description is encoded as a FlatBuffer as well. FlatBuffers have a canonical json-representation which comes in handy when a model description is created manually: The model description can be written as json (corresponding to the model description FlatBuffer schema), and the FlatBuffer compiler flatc compiles the json file to a binary buffer. 

The model description consists of three pieces: 
1. A model info structure that contains metadata such as name and id of the model, as well as creation tool and date of creation. 
2. A list of variables. 
3. An experiment info structure that describes constraints for valid experimental conditions in which the module can participate, such as minimal and maximal time step size, as well as defaults for start time, end time and time step size.

A variable can be of type double (`DtasmReal`), integer (`DtasmInt`), boolean (`DtasmBool`) and UTF-8 encoded string (`DtasmString`). Each variable has a human-readable name, an integer id that uniquely identifies the variable, optional description and unit. Causality of variables can be `parameter` (i.e., can be set during initialization only), `input` (can be set before each timestep), `output` (can be read after a time step) and state (like `output`, but not meant for external consumption). Variables can supply default values. Note that these default values are only for information and the dtasm implementation will not ensure that defaults will be set for input variables if no custom value is set. 

A UML class diagram of the FlatBuffers `modelDescription` schema is given below. 

![ModelDescription](images/modelDescription.svg)  
<p align="center">
Model description data structure
</p>

## Logging
dtasm supports three log levels: `Error`, `Warn` and `Info`. The limit level for message logging is set during the call to `init`. dtasm modules write log messages to the standard output stream (stdout). When compiled to WebAssembly, writing to stdout needs to be translated into the corresponding function from the WebAssembly System Interface (WASI). These function are specified as imports by the dtasm module. WASI-compatible dtasm runtimes wire up these imports when instantiating the module, but the runtime can also choose to provide alternate implementations, such as logging to a file or to a central log collection service. 

## Appendix - Interface Data Structures
Below UML class diagrams are shown for all interface data structures of dtasm interface functions. For reference, see the 

### `initReq`
![InitReq](images/initReq.svg)  

### `StatusRes`
![StatusRes](images/statusRes.svg)  

### `doStepReq`
![DoStepReq](images/doStepReq.svg)  

### `doStepRes`
![DoStepRes](images/doStepRes.svg)

### `getValuesReq`
![GetValuesReq](images/getValuesReq.svg)  

### `getValuesRes`
![GetValuesRes](images/getValuesRes.svg)  

### `setValuesReq`
![SetValuesReq](images/setValuesReq.svg)  

### `resetStepReq`
![ResetStepReq](images/resetStepReq.svg)  
