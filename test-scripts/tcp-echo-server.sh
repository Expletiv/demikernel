#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
source "${SCRIPT_DIR}/common.sh"

SERVER_IP="${SERVER_IP:-10.0.0.82}"
PORT="${PORT:-12345}"
THREADS="${THREADS:-1}"
LOG_INTERVAL="${LOG_INTERVAL:-5}"

echo "Starting ${LIBOS} TCP echo server on ${SERVER_IP}:${PORT}"
echo "CONFIG_PATH=${CONFIG_PATH}"

exec "${BUILD_DIR}/rust/tcp-echo.elf" \
  --peer server \
  --address "${SERVER_IP}:${PORT}" \
  --nthreads "${THREADS}" \
  --log "${LOG_INTERVAL}"
