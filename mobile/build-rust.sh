# build-rust.sh

#!/bin/bash

set -e

THISDIR=$(dirname $0)
cd $THISDIR

# Build the project for the desired platforms:
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
mkdir -p ./target/universal-macos/release

lipo \
    ./target/aarch64-apple-darwin/release/libds_emulator_mobile.a \
    ./target/x86_64-apple-darwin/release/libds_emulator_mobile.a -create -output \
    ./target/universal-macos/release/libds_emulator_mobile.a

cargo build --release --target aarch64-apple-ios

cargo build --release --target x86_64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
mkdir -p ./target/universal-ios/release

lipo \
    ./target/aarch64-apple-ios-sim/release/libds_emulator_mobile.a \
    ./target/x86_64-apple-ios/release/libds_emulator_mobile.a -create -output \
    ./target/universal-ios/release/libds_emulator_mobile.a