#!/bin/bash

set -e

# Build for Linux
cargo build --release --target x86_64-unknown-linux-musl
mkdir -p dist/linux
cp target/x86_64-unknown-linux-musl/release/smartfan dist/linux/
cp config/HR650X.yaml dist/linux/
cd dist/linux
zip -r ../smartfan-linux.zip .
cd ../..

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu
mkdir -p dist/windows
cp target/x86_64-pc-windows-gnu/release/smartfan.exe dist/windows/
cp config/HR650X.yaml dist/windows/
cd dist/windows
zip -r ../smartfan-windows.zip .
cd ../..

echo "Build and packaging completed."
