cmake_minimum_required(VERSION 3.22.1)
project(personaflux_jni)

add_library(personaflux SHARED IMPORTED)
set_target_properties(personaflux PROPERTIES
    IMPORTED_LOCATION "${PERSONAFLUX_NATIVE_DIR}/${ANDROID_ABI}/libpersonaflux.so"
    INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_LIST_DIR}/../../../../../../crates/personaflux-ffi/include"
)

add_library(personaflux_jni SHARED personaflux_jni.cpp)
target_include_directories(personaflux_jni PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/../../../../../../crates/personaflux-ffi/include"
)
target_link_libraries(personaflux_jni personaflux log)
