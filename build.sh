#!/bin/bash
set -e

DEMI_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"

# If an argument is passed (e.g., ./build_demi.sh catpowder), use it.
# Otherwise, default to 'catnap'.
LIBOS="${1:-catnap}"

echo "================================================="
echo "Starting Demikernel build for LIBOS: $LIBOS"
echo "================================================="

cd "$DEMI_DIR"

export CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback

echo "Compiling $LIBOS..."
make LIBOS="$LIBOS"

OUT_DIR="$DEMI_DIR/bin/examples"
BUILD_DIR="$DEMI_DIR/build"

cp -r "$OUT_DIR/"* "$BUILD_DIR/"

echo ""
echo "Build complete. The compiled executables are located in: $BUILD_DIR"

CONFIG_FILE="$DEMI_DIR/config/nodeXX.yaml"

echo ""
echo "================================================="
echo "✅ Build successful!"
echo "================================================="
echo "Before running the binaries, export the environment variables in your terminal."
echo "Copy and paste this block to set them up:"
echo ""
echo "export LIBOS=$LIBOS"
echo "export CONFIG_PATH=$CONFIG_FILE"
echo ""