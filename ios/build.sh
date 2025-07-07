#!/bin/bash

set -e

echo "Building AdBlock for iOS..."

# Build Rust library for iOS targets
echo "Building Rust core library..."
cd ../core

# Install iOS targets if not present
rustup target add x86_64-apple-ios aarch64-apple-ios-sim aarch64-apple-ios

# Build for all iOS targets
echo "Building for x86_64 simulator..."
cargo build --release --target x86_64-apple-ios

echo "Building for arm64 simulator..."
cargo build --release --target aarch64-apple-ios-sim

echo "Building for arm64 device..."
cargo build --release --target aarch64-apple-ios

# Create XCFramework
echo "Creating XCFramework..."
cd ../ios

# Create fat library for simulator
lipo -create \
    ../core/target/x86_64-apple-ios/release/libadblock_core.a \
    ../core/target/aarch64-apple-ios-sim/release/libadblock_core.a \
    -output ./libadblock_core_sim.a

# Copy device library
cp ../core/target/aarch64-apple-ios/release/libadblock_core.a ./libadblock_core_device.a

# Remove existing XCFramework
rm -rf AdBlockCoreFFI.xcframework

# Create XCFramework without headers (using bridging header instead)
xcodebuild -create-xcframework \
    -library ./libadblock_core_sim.a \
    -library ./libadblock_core_device.a \
    -output AdBlockCoreFFI.xcframework

# Clean up temporary files
rm -f ./libadblock_core_sim.a ./libadblock_core_device.a

echo "iOS build complete!"
echo "XCFramework created at: AdBlockCoreFFI.xcframework"

# Verify the framework
echo ""
echo "Framework info:"
ls -la AdBlockCoreFFI.xcframework/