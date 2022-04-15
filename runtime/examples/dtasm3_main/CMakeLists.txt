cmake_minimum_required(VERSION 3.9)
project(dtasm3_main)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED YES)
set(CMAKE_C_EXTENSIONS NO)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED YES)
set(CMAKE_CXX_EXTENSIONS NO)

set(target ${PROJECT_NAME})

add_subdirectory(${PROJECT_SOURCE_DIR}/../../dtasm3 dtasm3_lib)

include_directories(
        ${PROJECT_SOURCE_DIR}/../../dtasm3/include
        ${PROJECT_SOURCE_DIR}/../../../lib/dtasm_abi/include
        ${PROJECT_SOURCE_DIR}/../../../third_party/wasm3.git/source
        ${PROJECT_SOURCE_DIR}/../../../third_party/flatbuffers.git/include
)

add_executable(${target} "src/main.cpp")
target_link_libraries(${target} PRIVATE dtasm3)
set_target_properties(${target} PROPERTIES OUTPUT_NAME "dtasm3")
