#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
source "${SCRIPT_DIR}/common.sh"

SERVER_IP="${SERVER_IP:-10.0.0.82}"
SERVER_PORT="${SERVER_PORT:-12345}"

CORE_ID="${CORE_ID:-2}"
OS_TUNING="nice -n -20 chrt -f 99 taskset -c ${CORE_ID}"

echo "Starting ${LIBOS} UDP echo server on ${SERVER_IP}:${SERVER_PORT} (Pinned to core ${CORE_ID})"
echo "CONFIG_PATH=${CONFIG_PATH}"

exec ${OS_TUNING} "${BUILD_DIR}/rust/udp-echo.elf" \
  --local "${SERVER_IP}:${SERVER_PORT}"