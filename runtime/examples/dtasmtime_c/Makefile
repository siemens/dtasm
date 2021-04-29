.PHONY: clean default

C_API_DIR = ../../dtasmtime-c-api/target

CFLAGS = --std=c11
INCLUDES = -I$(C_API_DIR)
LDFLAGS = -L$(C_API_DIR)/debug -lm -ldtasmtime_c_api

default: target/main

target/main: 
	mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(INCLUDES) -o $@ src/main.c $(LDFLAGS)

clean: 
	rm -r ./target

run: 
	LD_LIBRARY_PATH=$(C_API_DIR)/debug ./target/main