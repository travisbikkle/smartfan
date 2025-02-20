#!/bin/bash

set -e

TARGET=$1

if [ "$TARGET" == "x86_64-unknown-linux-musl" ]; then
    # Build for Linux
    cargo build --release --target x86_64-unknown-linux-musl
    mkdir -p dist/linux
    cp target/x86_64-unknown-linux-musl/release/smartfan dist/linux/
    cp scripts/HR650X.yaml dist/linux/
    cp scripts/install.sh dist/linux/
    cd dist/linux
    zip -r ../smartfan-linux.zip .
    cd ../..
elif [ "$TARGET" == "x86_64-pc-windows-gnu" ]; then
    # Build for Windows
    cargo build --release --target x86_64-pc-windows-gnu
    mkdir -p dist/windows
    cp target/x86_64-pc-windows-gnu/release/smartfan.exe dist/windows/
    cp scripts/HR650X.yaml dist/windows/
    cp scripts/install.bat dist/windows/
    cd dist/windows
    zip -r ../smartfan-windows.zip .
    cd ../..
else
    echo "Unsupported target: $TARGET"
    exit 1
fi

echo "Build and packaging completed for target: $TARGET."
