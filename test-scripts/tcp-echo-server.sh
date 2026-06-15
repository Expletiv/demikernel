#!/bin/bash
set -e

CONFIG_FILE="/demikernel/config/$(hostname).yaml"
BUILD_DIR="/demikernel/build"

SERVER_IP="10.0.0.82"
PORT="12345"
THREADS=1
LOG_LEVEL=5

export LIBOS="catnip"
export CONFIG_PATH="${CONFIG_FILE}"
export LD_LIBRARY_PATH="/demikernel/lib/x86_64-linux-gnu:/tmp/demikernel/lib"

echo "Starting TCP Server on ${SERVER_IP}:${PORT}..."

exec "$BUILD_DIR/rust/tcp-echo.elf" \
  --peer server \
  --address "${SERVER_IP}:${PORT}" \
  --nthreads "${THREADS}" \
  --log "${LOG_LEVEL}"