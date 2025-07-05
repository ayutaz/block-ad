cmake_minimum_required(VERSION 3.18.1)
project("adblock_jni")

# Add the JNI wrapper library
add_library(
    adblock_jni
    SHARED
    adblock_jni.cpp
)

# Find and link required libraries
find_library(log-lib log)

# Set output library name to match what Java expects
set_target_properties(adblock_jni PROPERTIES OUTPUT_NAME "adblock_jni")

# Link the Rust static library
add_library(adblock_core STATIC IMPORTED)
set_target_properties(
    adblock_core
    PROPERTIES IMPORTED_LOCATION
    ${CMAKE_CURRENT_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libadblock_core.a
)

# Link libraries
target_link_libraries(
    adblock_jni
    adblock_core
    ${log-lib}
)