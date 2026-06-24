#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
source "${SCRIPT_DIR}/common.sh"

SERVER_IP="${SERVER_IP:-10.0.0.82}"
PORT="${PORT:-12345}"
CLIENTS="${CLIENTS:-1}"
REQUESTS="${REQUESTS:-1000000}"
BUF_SIZE="${BUF_SIZE:-64}"
RUN_MODE="${RUN_MODE:-sequential}"
LOG_INTERVAL="${LOG_INTERVAL:-5}"

CORE_ID="${CORE_ID:-3}"
OS_TUNING="nice -n -20 chrt -f 99 taskset -c ${CORE_ID}"

echo "Starting ${LIBOS} TCP echo client to ${SERVER_IP}:${PORT} (Pinned to core ${CORE_ID})"
echo "CONFIG_PATH=${CONFIG_PATH}"

exec ${OS_TUNING} "${BUILD_DIR}/rust/tcp-echo.elf" \
  --peer client \
  --address "${SERVER_IP}:${PORT}" \
  --nclients "${CLIENTS}" \
  --nrequests "${REQUESTS}" \
  --bufsize "${BUF_SIZE}" \
  --run-mode "${RUN_MODE}" \
  --log "${LOG_INTERVAL}"
