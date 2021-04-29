.PHONY: clean default

FB_DIR = ../../third_party/flatbuffers.git/
WASI_SDK = /opt/wasi-sdk/
DTASM_ABI = ../../dtasm_abi/

FLATC = $(FB_DIR)flatc
FB_INCLUDE = $(FB_DIR)include

INCLUDES = -I./include -I./target/include -I$(FB_INCLUDE) -I$(DTASM_ABI)include
CC = $(WASI_SDK)bin/clang
CXX = $(WASI_SDK)bin/clang++
LD = $(WASI_SDK)bin/wasm-ld
C_FLAGS = --target=wasm32-wasi --sysroot=$(WASI_SDK)share/wasi-sysroot -O3 -std=gnu99 -Wall -Wextra -Wno-unused-parameter -fPIC
CXX_FLAGS = --target=wasm32-wasi --sysroot=$(WASI_SDK)share/wasi-sysroot -O3 -fPIC -std=c++11 -fno-exceptions
LD_FLAGS = -L$(WASI_SDK)share/wasi-sysroot/lib/wasm32-wasi -lc -lc++ -lc++abi --entry=_initialize -O3 \
	$(WASI_SDK)lib/clang/11.0.0/lib/wasi/libclang_rt.builtins-wasm32.a \
	--export=alloc --export=dealloc --export=init --export=getModelDescription --export=getValues --export=doStep --export=setValues

TARGET = target/dpend_cpp.wasm

IN = $(wildcard ../dataModel/*.fbs)
OUT1 = $(addsuffix _generated.h,$(basename $(IN)))
OUT = $(subst dataModel/,dataModel/,$(OUT1))

SRCCPP = $(wildcard src/*.cpp)
TMPCPP = $(subst .cpp,.o,$(SRCCPP))
OBJCPP = $(subst src/,target/obj/,$(TMPCPP))

SRCC = $(wildcard src/*.c)
TMPC = $(subst .c,.o,$(SRCC))
OBJC = $(subst src/,target/obj/,$(TMPC))

default: $(TARGET)

$(TARGET): $(OBJCPP) $(OBJC)
	$(LD) $(LD_FLAGS) target/obj/*.o $(WASI_SDK)share/wasi-sysroot/lib/wasm32-wasi/crt1-reactor.o -o $@

target/include/modelDescription.h: ../dpend/src/modelDescription.json
	$(FLATC) -b -o $(dir $@) $(DTASM_ABI)schema/dtasm.fbs $<
	mv $(dir $@)/modelDescription.bin $(dir $@)/modelDescription.fb
	(cd $(dir $@); xxd -i modelDescription.fb | sed 's/\([0-9a-f]\)$$/\0, 0x00/' > modelDescription.h)

target/obj/%.o: src/%.cpp target/include/modelDescription.h
	mkdir -p $(dir $@)
	$(CXX) $(CXX_FLAGS) $(INCLUDES) -c $< -o $@

target/obj/%.o: src/%.c target/include/modelDescription.h
	mkdir -p $(dir $@)
	$(CC) $(C_FLAGS) $(INCLUDES) -c $< -o $@

clean: 
	rm -rf target
