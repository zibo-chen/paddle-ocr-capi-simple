#!/bin/bash

set -e

echo "Building OCR C API library..."
cd ../..
cargo build --release

echo "Building C example..."
cd examples/c_simple

# macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    gcc -o example example.c \
        -I../../include \
        -L../../target/release \
        -locr_capi \
        -Wl,-rpath,@loader_path/../../target/release
    echo "Build successful! (macOS)"
fi

# Linux
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    gcc -o example example.c \
        -I../../include \
        -L../../target/release \
        -locr_capi \
        -Wl,-rpath,\$ORIGIN/../../target/release
    echo "Build successful! (Linux)"
fi

echo ""
echo "Run example with:"
echo "  ./example <image_path>"
