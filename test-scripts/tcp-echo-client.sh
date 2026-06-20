#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
source "${SCRIPT_DIR}/common.sh"

SERVER_IP="${SERVER_IP:-10.0.0.82}"
PORT="${PORT:-12345}"
CLIENTS="${CLIENTS:-1}"
REQUESTS="${REQUESTS:-1000000}"
BUF_SIZE="${BUF_SIZE:-64}"
RUN_MODE="${RUN_MODE:-concurrent}"
LOG_INTERVAL="${LOG_INTERVAL:-5}"

echo "Starting ${LIBOS} TCP echo client to ${SERVER_IP}:${PORT}"
echo "CONFIG_PATH=${CONFIG_PATH}"

exec "${BUILD_DIR}/rust/tcp-echo.elf" \
  --peer client \
  --address "${SERVER_IP}:${PORT}" \
  --nclients "${CLIENTS}" \
  --nrequests "${REQUESTS}" \
  --bufsize "${BUF_SIZE}" \
  --run-mode "${RUN_MODE}" \
  --log "${LOG_INTERVAL}"
