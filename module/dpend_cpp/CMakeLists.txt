cmake_minimum_required(VERSION 3.4.0)

project(dpend_cpp)

if(DEFINED ENV{WASI_SDK_PREFIX})
	set(WASI_SDK_PREFIX $ENV{WASI_SDK_PREFIX})
else()
	set(WASI_SDK_PREFIX "/opt/wasi-sdk")
endif()

set(INCLUDE_DIRS 
	"include" 
	"../dpend/target"
	"../../lib/dtasm_abi/include"
	"../../third_party/flatbuffers.git/include")
set(CXX_FLAGS "-fPIC -std=c++11 -fno-exceptions")
set(C_FLAGS "-std=gnu99 -Wall -Wextra -Wno-unused-parameter -fPIC")
string(CONCAT LD_FLAGS "-lc -lc++ -lc++abi -mexec-model=reactor "
	"-L${WASI_SDK_PREFIX}share/wasi-sysroot/lib/wasm32-wasi "
	"-Wl,--export=alloc "
	"-Wl,--export=dealloc "
	"-Wl,--export=init "
	"-Wl,--export=getModelDescription "
	"-Wl,--export=getValues "
	"-Wl,--export=doStep "
	"-Wl,--export=setValues")

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/target)
# set(CMAKE_C_COMPILER_WORKS ON)
# set(CMAKE_CXX_COMPILER_WORKS ON)

include_directories(${INCLUDE_DIRS})
file(GLOB SOURCES "src/*.cpp" "src/*.c")
add_executable(dpend_cpp ${SOURCES})
set_target_properties(dpend_cpp PROPERTIES SUFFIX ".wasm")

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${CXX_FLAGS}")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${C_FLAGS}")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_LINKER_FLAGS} ${LD_FLAGS}")