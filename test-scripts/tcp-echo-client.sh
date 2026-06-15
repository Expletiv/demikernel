#!/bin/bash
set -e

CONFIG_FILE="/demikernel/config/$(hostname).yaml"
BUILD_DIR="/demikernel/build"


SERVER_IP="10.0.0.82"
PORT="12345"
CLIENTS=1
REQUESTS=1000000
BUF_SIZE=64
RUN_MODE="concurrent"
LOG_LEVEL=5

export LIBOS="catnip"
export CONFIG_PATH="${CONFIG_FILE}"
export LD_LIBRARY_PATH="/demikernel/lib/x86_64-linux-gnu:/tmp/demikernel/lib"

echo "Starting TCP Client connecting to ${SERVER_IP}:${PORT}..."

exec "$BUILD_DIR/rust/tcp-echo.elf" \
  --peer client \
  --address "${SERVER_IP}:${PORT}" \
  --nclients "${CLIENTS}" \
  --nrequests "${REQUESTS}" \
  --bufsize "${BUF_SIZE}" \
  --run-mode "${RUN_MODE}" \
  --log "${LOG_LEVEL}"