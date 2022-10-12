cmake_minimum_required(VERSION 3.9)
project(dtasm3)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED YES)
set(CMAKE_C_EXTENSIONS NO)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED YES)
set(CMAKE_CXX_EXTENSIONS NO)

set(target ${PROJECT_NAME})

add_subdirectory(${PROJECT_SOURCE_DIR}/../../third_party/wasm3.git/source m3)

include_directories(
        ${PROJECT_SOURCE_DIR}/include
        ${PROJECT_SOURCE_DIR}/src
        ${PROJECT_SOURCE_DIR}/../../lib/dtasm_abi/include
        ${PROJECT_SOURCE_DIR}/../../third_party/flatbuffers.git/include
        ${PROJECT_SOURCE_DIR}/../../third_party/wasm3.git/platforms/embedded/esp32-idf-wasi/main
)

file(GLOB all_srcs
        "${PROJECT_SOURCE_DIR}/include/*.h"
        "${PROJECT_SOURCE_DIR}/src/*.cpp"
        )

if(ESP_PLATFORM)
    add_library(${target} STATIC ${all_srcs} ${PROJECT_SOURCE_DIR}/../../third_party/wasm3.git/platforms/embedded/esp32-idf-wasi/main/m3_api_esp_wasi.c)
else()
    add_library(${target} STATIC ${all_srcs})
endif()

target_link_libraries(${target} PRIVATE m3)
set_target_properties(m3 PROPERTIES COMPILE_FLAGS -Wno-maybe-uninitialized)
target_compile_definitions(m3 PUBLIC d_m3MaxFunctionStackHeight=4096)

target_compile_definitions(${target} PUBLIC d_m3HasWASI)
if(ESP_PLATFORM)
    target_compile_definitions(${target} PUBLIC ESP32)
else()
    target_compile_definitions(m3 PUBLIC d_m3HasWASI)
endif()
