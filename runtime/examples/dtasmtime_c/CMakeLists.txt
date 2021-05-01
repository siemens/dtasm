cmake_minimum_required(VERSION 3.4.0)

project(dtasmtime_c)

set(CONFIG $ENV{CONFIG})
if(NOT CONFIG)
    set(CONFIG "debug")
endif()

set(C_API_DIR "${CMAKE_SOURCE_DIR}/../../dtasmtime-c-api/target")

if(WIN32)
	set(IMPORTED_LOC ${C_API_DIR}/${CONFIG}/dtasmtime_c_api)
else()
	set(IMPORTED_LOC ${C_API_DIR}/${CONFIG}/libdtasmtime_c_api.so)
endif()

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/target)

file(GLOB SOURCES "src/*.cpp" "src/*.c")
add_executable(main ${SOURCES})
set_property(TARGET main PROPERTY C_STANDARD 11)

add_library(dtasmtime_c_api SHARED IMPORTED)
set_target_properties(dtasmtime_c_api PROPERTIES
    IMPORTED_LOCATION ${IMPORTED_LOC}
    IMPORTED_IMPLIB "${C_API_DIR}/${CONFIG}/dtasmtime_c_api.dll.lib"
    INTERFACE_INCLUDE_DIRECTORIES ${C_API_DIR}
)
target_link_libraries(main dtasmtime_c_api)