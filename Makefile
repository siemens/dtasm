.PHONY: clean distclean default all cpp run-rs run-c deps

CONFIG ?= debug
WASI_SDK ?= /opt/wasi-sdk

FB_DIR = third_party/flatbuffers.git

ifeq ($(OS),Windows_NT)
	FB_CMAKE_FLAGS =
	LIB_EXT = .dll
	LIB_PREFIX = 
	EXE_EXT = .exe
	CMAKE_X64 = -DCMAKE_GENERATOR_PLATFORM=x64
	CARGO_FLAGS = -j1
	CONFIG_DIR = $(CONFIG)/
else # assuming Linux/gcc
	FB_CMAKE_FLAGS = -DCMAKE_C_COMPILER=/usr/bin/gcc -DCMAKE_CXX_COMPILER=/usr/bin/g++
	LIB_EXT = .so
	LIB_PREFIX = lib
	EXE_EXT =
	CMAKE_X64 =
	CARGO_FLAGS = 
	CONFIG_DIR = 
endif

ifeq ($(CONFIG),release)
	CARGO_BUILD_FLAGS = $(CARGO_FLAGS) --release
else
	CARGO_BUILD_FLAGS = $(CARGO_FLAGS)
endif

FLATC = $(FB_DIR)/flatc$(EXE_EXT)

DPEND_RS = module/dpend_rs/target/wasm32-wasi/$(CONFIG)/dpend_rs.wasm
ADD_RS = module/add_rs/target/wasm32-wasi/$(CONFIG)/add_rs.wasm
DPEND_C = module/dpend_cpp/target/dpend_cpp.wasm
DTASMTIME = runtime/dtasmtime/target/$(CONFIG)/libdtasmtime.rlib
DTASMTIME_C = runtime/dtasmtime-c-api/target/$(CONFIG)/$(LIB_PREFIX)dtasmtime_c_api$(LIB_EXT)
DTASMTIME_MAIN = runtime/examples/dtasmtime_rs/target/$(CONFIG)/dtasmtime_rs$(EXE_EXT)
DTASMTIME_MAIN_C = runtime/examples/dtasmtime_c/target/$(CONFIGDIR)main$(EXE_EXT)
DEP_FILES = lib/dtasm_abi/src/dtasm_generated/mod.rs module/dpend/target/modelDescription.fb lib/dtasm_abi/include/dtasm_generated.h module/dpend/target/modelDescription.h

default: $(DPEND_RS) $(ADD_RS) $(DTASMTIME_MAIN)

cpp: $(DPEND_C) $(DTASMTIME_MAIN_C)

all: default cpp

deps: $(FLATC) $(DEP_FILES)

run-rs: $(DPEND_RS) $(DTASMTIME_MAIN)
	cd runtime/examples/dtasmtime_rs; cargo run -- --input ../../../$(DPEND_RS)

run-c: $(DPEND_C) $(DTASMTIME_MAIN_C)
	cp $(DTASMTIME_C) runtime/examples/dtasmtime_c/target/$(CONFIG_DIR)
	cp $(DPEND_C) runtime/examples/dtasmtime_c/target/$(CONFIG_DIR)
	cp $(ADD_RS) runtime/examples/dtasmtime_c/target/$(CONFIG_DIR)
	cd runtime/examples/dtasmtime_c/target/$(CONFIG_DIR); ./main$(EXE_EXT) $(notdir $(DPEND_C)) 0.0 10.0 100
	cd runtime/examples/dtasmtime_c/target/$(CONFIG_DIR); ./main$(EXE_EXT) $(notdir $(ADD_RS)) 0.0 1.0 1

test: $(DTASMTIME)
	cd runtime/dtasmtime; cargo test $(CARGO_BUILD_FLAGS)

$(FLATC):
	mkdir -p $(FB_DIR)/_build
	cd $(FB_DIR)/_build; cmake $(FB_CMAKE_FLAGS) ..
	cd $(FB_DIR)/_build; cmake --build . --target flatc --config $(CONFIG)
	cp $(FB_DIR)/_build/$(CONFIG_DIR)flatc$(EXE_EXT) $(FB_DIR)

$(DTASMTIME): deps
	cd runtime/dtasmtime; cargo build $(CARGO_BUILD_FLAGS)

$(DTASMTIME_C): deps
	cd runtime/dtasmtime-c-api; cargo build $(CARGO_BUILD_FLAGS)

$(DTASMTIME_MAIN): deps
	cd runtime/examples/dtasmtime_rs; cargo build $(CARGO_BUILD_FLAGS)

$(DTASMTIME_MAIN_C): $(DTASMTIME_C)
	mkdir -p runtime/examples/dtasmtime_c/build
	cd runtime/examples/dtasmtime_c/build; cmake .. $(CMAKE_X64)
	cd runtime/examples/dtasmtime_c/build; cmake --build . --config  $(CONFIG) --verbose

$(DPEND_RS): deps
	cd module/dpend_rs && cargo build $(CARGO_BUILD_FLAGS)

$(ADD_RS): deps
	cd module/add_rs && cargo build $(CARGO_BUILD_FLAGS)

$(DPEND_C): deps
	mkdir -p module/dpend_cpp/build
	cd module/dpend_cpp/build; cmake .. -G "Unix Makefiles" -DCMAKE_TOOLCHAIN_FILE="$(WASI_SDK)/share/cmake/wasi-sdk.cmake" -DWASI_SDK_PREFIX="$(WASI_SDK)" -DCMAKE_BUILD_TYPE=$(CONFIG)
	cd module/dpend_cpp/build; cmake --build . --config  $(CONFIG) --verbose

lib/dtasm_abi/src/dtasm_generated/mod.rs: lib/dtasm_abi/schema/dtasm.fbs $(FLATC)
	$(FLATC) --rust --gen-mutable -o $(dir $@) $<
	mv $(dir $@)/dtasm_generated.rs $@

lib/dtasm_abi/include/dtasm_generated.h: lib/dtasm_abi/schema/dtasm.fbs $(FLATC)
	$(FLATC) -c -o $(dir $@) $<

module/dpend/target/modelDescription.fb: module/dpend/src/modelDescription.json $(FLATC)
	$(FLATC) -b -o module/dpend/target lib/dtasm_abi/schema/dtasm.fbs $<
	mv module/dpend/target/modelDescription.bin $@

module/dpend/target/modelDescription.h: module/dpend/target/modelDescription.fb
	cd $(dir $@); xxd -i modelDescription.fb | sed 's/\([0-9a-f]\)$$/\0, 0x00/' > modelDescription.h

clean: 
	rm -rf lib/dtasm_abi/src/dtasm_generated
	rm -f lib/dtasm_abi/include/dtasm_generated.h
	rm -rf lib/dtasm_abi/target
	rm -rf module/dpend/target
	rm -rf module/dpend_rs/target
	rm -rf module/add_rs/target
	rm -rf module/dpend_cpp/target
	rm -rf module/dpend_cpp/build
	rm -rf runtime/dtasmtime/target
	rm -rf runtime/dtasmtime-c-api/target
	rm -rf runtime/examples/dtasmtime_rs/target
	rm -rf runtime/examples/dtasmtime_c/target
	rm -rf runtime/examples/dtasmtime_c/build

distclean: clean
	rm -rf $(FB_DIR)/_build
	rm -f $(FLATC)