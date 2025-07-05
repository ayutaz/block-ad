#!/bin/bash
set -e

echo "🚀 Setting up AdBlock development environment..."

# Check for required tools
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    fi
    echo "✅ $1 is installed"
}

echo "Checking prerequisites..."
check_command rustc
check_command cargo

# Install Rust targets for mobile
echo "Installing Rust targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi || true
rustup target add aarch64-apple-ios x86_64-apple-ios || true

# Install cargo-ndk for Android
if ! command -v cargo-ndk &> /dev/null; then
    echo "Installing cargo-ndk..."
    cargo install cargo-ndk
fi

# Install cbindgen for header generation
if ! command -v cbindgen &> /dev/null; then
    echo "Installing cbindgen..."
    cargo install cbindgen
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. For Android development: Install Android Studio and NDK"
echo "2. For iOS development: Install Xcode"
echo "3. Run './scripts/build_all.sh' to build the project"