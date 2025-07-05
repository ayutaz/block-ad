#!/bin/bash

# iOS build script for AdBlock

set -e

echo "Building AdBlock for iOS..."

# Build Rust library for iOS targets
echo "Building Rust core library..."
cd ../core

# Build for iOS simulator (x86_64)
cargo build --release --target x86_64-apple-ios

# Build for iOS simulator (aarch64)
cargo build --release --target aarch64-apple-ios-sim

# Build for iOS device (aarch64)
cargo build --release --target aarch64-apple-ios

# Create fat library
echo "Creating XCFramework..."
cd ../ios

# Create directories
mkdir -p AdBlockCoreFFI.xcframework

# Create xcframework
xcodebuild -create-xcframework \
    -library ../core/target/x86_64-apple-ios/release/libadblock_core.a \
    -headers AdBlock \
    -library ../core/target/aarch64-apple-ios-sim/release/libadblock_core.a \
    -headers AdBlock \
    -library ../core/target/aarch64-apple-ios/release/libadblock_core.a \
    -headers AdBlock \
    -output AdBlockCoreFFI.xcframework

echo "iOS build complete!"