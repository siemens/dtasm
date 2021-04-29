.PHONY: clean distclean default all cpp run-rs run-c

CONFIG = debug

FB_DIR = third_party/flatbuffers.git/
FLATC = $(FB_DIR)flatc

DPEND_RS = module/dpend_rs/target/wasm32-wasi/$(CONFIG)/dpend_rs.wasm
DPEND_C = module/dpend_cpp/target/dpend_cpp.wasm
DTASMTIME = runtime/dtasmtime/target/$(CONFIG)/libdtasmtime.rlib
DTASMTIME_C = runtime/dtasmtime-c-api/target/$(CONFIG)/libdtasmtime_c_api.so
DTASMTIME_MAIN = runtime/examples/dtasmtime_rs/target/$(CONFIG)/dtasmtime_rs
DTASMTIME_MAIN_C = runtime/examples/dtasmtime_c/target/main

default: $(DPEND_RS) $(DTASMTIME_MAIN)

cpp: $(DPEND_C) $(DTASMTIME_MAIN_C)

all: default cpp

run-rs: $(DPEND_RS) $(DTASMTIME_MAIN)
	cd runtime/examples/dtasmtime_rs && cargo run -- --input ../../../$(DPEND_RS)

run-c: $(DPEND_C) $(DTASMTIME_MAIN_C)
	cd runtime/examples/dtasmtime_c && LD_LIBRARY_PATH=../../dtasmtime-c-api/target/debug target/main ../../../module/dpend_cpp/target/dpend_cpp.wasm 0.0 10.0 100

$(FLATC):
	$(MAKE) -C third_party

$(DTASMTIME): dtasm_abi/src/dtasm_generated.rs
	cd runtime/dtasmtime && cargo build

$(DTASMTIME_C): dtasm_abi/src/dtasm_generated.rs
	cd runtime/dtasmtime-c-api && cargo build

$(DTASMTIME_MAIN): dtasm_abi/src/dtasm_generated.rs
	cd runtime/examples/dtasmtime_rs && cargo build

$(DTASMTIME_MAIN_C): dtasm_abi/src/dtasm_generated.rs $(DTASMTIME_C)
	$(MAKE) -C runtime/examples/dtasmtime_c

$(DPEND_RS): dtasm_abi/src/dtasm_generated.rs module/dpend/target/modelDescription.fb
	cd module/dpend_rs && cargo build

$(DPEND_C): dtasm_abi/include/dtasm_generated.h module/dpend/target/modelDescription.fb
	$(MAKE) -C module/dpend_cpp

dtasm_abi/src/dtasm_generated.rs: dtasm_abi/schema/dtasm.fbs $(FLATC)
	$(FLATC) --rust --gen-mutable -o $(dir $@) $<

dtasm_abi/include/dtasm_generated.h: dtasm_abi/schema/dtasm.fbs $(FLATC)
	$(FLATC) -c -o $(dir $@) $<

module/dpend/target/modelDescription.fb: module/dpend/src/modelDescription.json $(FLATC)
	$(FLATC) -b -o module/dpend/target dtasm_abi/schema/dtasm.fbs $<
	mv module/dpend/target/modelDescription.bin $@

clean: 
	rm -f dtasm_abi/src/dtasm_generated.rs
	rm -f dtasm_abi/include/dtasm_generated.h
	rm -rf dtasm_abi/target
	rm -rf module/dpend/target
	rm -rf module/dpend_rs/target
	rm -rf module/dpend_cpp/target
	rm -rf runtime/dtasmtime/target
	rm -rf runtime/dtasmtime-c-api/target
	rm -rf runtime/examples/dtasmtime_rs/target
	rm -rf runtime/examples/dtasmtime_c/target

distclean: clean
	$(MAKE) -C third_party clean
