#!/bin/bash

# Build script for Android

set -e

echo "Building Rust library for Android..."

# Set up cargo for Android cross-compilation
cd core

# Add Android targets if not already added
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android

# Build for each Android architecture
echo "Building for arm64-v8a..."
cargo build --target aarch64-linux-android --release

echo "Building for armeabi-v7a..."
cargo build --target armv7-linux-androideabi --release

echo "Building for x86..."
cargo build --target i686-linux-android --release

echo "Building for x86_64..."
cargo build --target x86_64-linux-android --release

# Copy built libraries to Android project
echo "Copying libraries to Android project..."

ANDROID_LIBS="../android/app/src/main/jniLibs"
mkdir -p "$ANDROID_LIBS"/{arm64-v8a,armeabi-v7a,x86,x86_64}

cp target/aarch64-linux-android/release/libadblock_core.a "$ANDROID_LIBS/arm64-v8a/"
cp target/armv7-linux-androideabi/release/libadblock_core.a "$ANDROID_LIBS/armeabi-v7a/"
cp target/i686-linux-android/release/libadblock_core.a "$ANDROID_LIBS/x86/"
cp target/x86_64-linux-android/release/libadblock_core.a "$ANDROID_LIBS/x86_64/"

echo "Android build complete!"