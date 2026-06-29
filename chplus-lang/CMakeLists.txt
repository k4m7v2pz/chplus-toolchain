cmake_minimum_required(VERSION 3.10)
project(chplus_interpreter)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(MSVC)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /W4")
    set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} /O2")
else()
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra")
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)

# 主线源码 (src/ 外层 6 个文件,已验证可编译运行)
# src/ 下的 lexer/ parser/ executor/ utils/ 子目录是历史残留,不参与编译
add_executable(chplus
    main.cpp
    src/lexer.cpp
    src/parser.cpp
    src/interpreter.cpp
    src/asm.cpp
    src/CodeFormatter.cpp
    src/CHFormatter.cpp
)

set_target_properties(chplus PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}
)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    target_compile_definitions(chplus PRIVATE DEBUG)
endif()
