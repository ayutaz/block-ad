#!/bin/bash

# Android build script for AdBlock

set -e

echo "Building AdBlock for Android..."

# Setup Gradle wrapper if needed
if [ ! -f "gradle/wrapper/gradle-wrapper.jar" ]; then
    echo "Setting up Gradle wrapper..."
    gradle wrapper --gradle-version=8.0.2 || {
        echo "Gradle not found, downloading wrapper manually..."
        mkdir -p gradle/wrapper
        curl -L https://services.gradle.org/distributions/gradle-8.0.2-bin.zip -o gradle-8.0.2-bin.zip
        unzip -q gradle-8.0.2-bin.zip
        ./gradle-8.0.2/bin/gradle wrapper --gradle-version=8.0.2
        rm -rf gradle-8.0.2*
    }
fi

# Build Rust library for Android targets
echo "Building Rust core library for Android..."
cd ../core

# Install Android targets if not present
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Build for all Android architectures using cargo-ndk
echo "Building for arm64-v8a..."
cargo ndk -t arm64-v8a build --release

echo "Building for armeabi-v7a..."
cargo ndk -t armeabi-v7a build --release

echo "Building for x86_64..."
cargo ndk -t x86_64 build --release

echo "Building for x86..."
cargo ndk -t x86 build --release

# Copy libraries to JNI libs directory
echo "Copying libraries to Android project..."
cd ../android/app/src/main/jniLibs

# Create directories
mkdir -p arm64-v8a armeabi-v7a x86_64 x86

# Copy libraries (cargo-ndk puts them in slightly different locations)
cp ../../../../../../core/target/aarch64-linux-android/release/libadblock_core.a arm64-v8a/
cp ../../../../../../core/target/armv7-linux-androideabi/release/libadblock_core.a armeabi-v7a/
cp ../../../../../../core/target/x86_64-linux-android/release/libadblock_core.a x86_64/
cp ../../../../../../core/target/i686-linux-android/release/libadblock_core.a x86/

# Build Android app
echo "Building Android app..."
cd ../../../../../

# Run Gradle build
./gradlew assembleDebug

echo "Android build complete!"
echo "APK location: app/build/outputs/apk/debug/app-debug.apk"