cmake_minimum_required(VERSION 3.10.2)

# Project name
project("adblock")

# Find and include the log library
find_library(log-lib log)

# Add the JNI wrapper library
add_library(adblock_jni SHARED
    src/main/cpp/adblock_jni.cpp
)

# Link libraries
target_link_libraries(adblock_jni
    ${log-lib}
    ${CMAKE_CURRENT_SOURCE_DIR}/src/main/jniLibs/${ANDROID_ABI}/libadblock_core.a
)

# Include directories
target_include_directories(adblock_jni PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src/main/cpp
)