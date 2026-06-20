#!/bin/bash
set -euo pipefail

DEMI_DIR="${DEMI_DIR:-/demikernel}"
LIBOS="${LIBOS:-catpowder}"
HOST_CONFIG="${HOST_CONFIG:-$(hostname).yaml}"
CONFIG_PATH="${CONFIG_PATH:-${DEMI_DIR}/config/${HOST_CONFIG}}"
BUILD_DIR="${BUILD_DIR:-${DEMI_DIR}/build/${LIBOS}}"

export LIBOS
export CONFIG_PATH

if [ -d "${DEMI_DIR}/lib" ]; then
  export LD_LIBRARY_PATH="${DEMI_DIR}/lib/x86_64-linux-gnu:${DEMI_DIR}/lib:${LD_LIBRARY_PATH:-}"
fi

if [ ! -f "${CONFIG_PATH}" ]; then
  echo "Missing CONFIG_PATH=${CONFIG_PATH}" >&2
  exit 1
fi

if [ ! -x "${BUILD_DIR}/rust/tcp-echo.elf" ]; then
  echo "Missing executable ${BUILD_DIR}/rust/tcp-echo.elf" >&2
  echo "Run ./build.sh ${LIBOS} on the build VM and ./sync.sh first." >&2
  exit 1
fi
