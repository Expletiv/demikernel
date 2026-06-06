#!/bin/bash
set -e

CONFIG_FILE="$HOME/demikernel/config/$(hostname).yaml"

SERVER_IP="10.0.0.82"
PORT="12345"
CLIENTS=1
REQUESTS=1000000
BUF_SIZE=64
RUN_MODE="concurrent"
LOG_LEVEL=5

export LIBOS="catnap"
export CONFIG_PATH="${CONFIG_FILE}"

echo "Starting TCP Client connecting to ${SERVER_IP}:${PORT}..."

exec "./build/rust/tcp-echo.elf" \
  --peer client \
  --address "${SERVER_IP}:${PORT}" \
  --nclients "${CLIENTS}" \
  --nrequests "${REQUESTS}" \
  --bufsize "${BUF_SIZE}" \
  --run-mode "${RUN_MODE}" \
  --log "${LOG_LEVEL}"