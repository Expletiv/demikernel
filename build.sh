#!/bin/bash
set -euo pipefail

DEMI_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
LIBOS="${1:-catnap}"
BUILD_ROOT="${DEMI_DIR}/build/${LIBOS}"

echo "================================================="
echo "Building Demikernel examples for LIBOS=${LIBOS}"
echo "================================================="

cd "${DEMI_DIR}"

export CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback

make LIBOS="${LIBOS}"

rm -rf "${BUILD_ROOT}"
mkdir -p "${BUILD_ROOT}"
cp -R "${DEMI_DIR}/bin/examples/"* "${BUILD_ROOT}/"

echo
echo "Build complete:"
echo "  ${BUILD_ROOT}"
echo
echo "Runtime defaults for the server side:"
echo "  export DEMI_DIR=/demikernel"
echo "  export LIBOS=${LIBOS}"
echo "  export CONFIG_PATH=/demikernel/config/${LIBOS}/\$(hostname).yaml"
