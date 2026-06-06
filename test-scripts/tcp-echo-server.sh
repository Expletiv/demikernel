#!/bin/bash
set -e

CONFIG_FILE="$HOME/demikernel/config/$(hostname).yaml"

SERVER_IP="10.0.0.82"
PORT="12345"
THREADS=1
LOG_LEVEL=5

export LIBOS="catnap"
export CONFIG_PATH="${CONFIG_FILE}"

echo "Starting TCP Server on ${SERVER_IP}:${PORT}..."

exec "./build/rust/tcp-echo.elf" \
  --peer server \
  --address "${SERVER_IP}:${PORT}" \
  --nthreads "${THREADS}" \
  --log "${LOG_LEVEL}"