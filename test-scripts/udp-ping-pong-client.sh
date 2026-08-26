#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
source "${SCRIPT_DIR}/common.sh"

SERVER_IP="${SERVER_IP:-10.0.0.82}"
SERVER_PORT="${SERVER_PORT:-12345}"
CLIENT_IP="${CLIENT_IP:-10.0.0.81}"
CLIENT_PORT="${CLIENT_PORT:-12345}"

CORE_ID="${CORE_ID:-3}"
OS_TUNING="nice -n -20 chrt -f 99 taskset -c ${CORE_ID}"

echo "Starting ${LIBOS} UDP ping-pong client from ${CLIENT_IP}:${CLIENT_PORT} (Pinned to core ${CORE_ID})"
echo "Targeting server at ${SERVER_IP}:${SERVER_PORT}"
echo "CONFIG_PATH=${CONFIG_PATH}"

exec ${OS_TUNING} "${BUILD_DIR}/rust/udp-ping-pong.elf" \
  --client "${CLIENT_IP}:${CLIENT_PORT}" "${SERVER_IP}:${SERVER_PORT}"