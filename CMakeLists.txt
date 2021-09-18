cmake_minimum_required(VERSION 3.20)

project(a52dec VERSION 0.7.4)

option(ENABLE_RUST "Enable Rust code" ON)

list(APPEND CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake)

include(CheckIncludeFile)

check_include_file(io.h HAVE_IO_H)

configure_file(config.h.cmake config.h)

if(ENABLE_RUST)
    enable_language(Rust)
    include (CMakeCargo)
    add_subdirectory(a52-rust)
endif()

add_library(a52
    include/a52.h
    ${PROJECT_BINARY_DIR}/config.h
    liba52/bit_allocate.c
    liba52/bitstream.c
    liba52/downmix.c
    liba52/imdct.c
    liba52/parse.c
    liba52/a52_internal.h
    liba52/bitstream.h
    liba52/tables.h
)
target_include_directories(a52
    PUBLIC
        include
    PRIVATE
        ${PROJECT_BINARY_DIR}
)
target_link_libraries(a52
    PRIVATE
        $<$<BOOL:${ENABLE_RUST}>:a52-rust>
        $<$<AND:$<BOOL:${ENABLE_RUST}>,$<BOOL:${WIN32}>>:ws2_32>
        $<$<AND:$<BOOL:${ENABLE_RUST}>,$<BOOL:${WIN32}>>:userenv>
)

add_library(ao
    include/audio_out.h
    libao/audio_out.c
    libao/audio_out_aif.c
    libao/audio_out_al.c
    libao/audio_out_float.c
    libao/audio_out_null.c
    libao/audio_out_oss.c
    libao/audio_out_peak.c
    libao/audio_out_solaris.c
    libao/audio_out_wav.c
    libao/audio_out_win.c
    libao/float2s16.c
    libao/audio_out_internal.h
)
target_include_directories(ao
    PUBLIC
        include
    PRIVATE
        ${PROJECT_BINARY_DIR}
)

add_executable(a52dec ${PROJECT_BINARY_DIR}/config.h src/a52dec.c)
target_include_directories(a52dec PRIVATE ${PROJECT_BINARY_DIR})
target_link_libraries(a52dec PRIVATE a52 ao)
