#!/bin/bash
set -euo pipefail

LIBOS="${LIBOS:-catnap}"
REMOTE_DIR="${REMOTE_DIR:-/demikernel}"
HOSTS=("$@")

if [ "${#HOSTS[@]}" -eq 0 ]; then
  HOSTS=(node81 node82)
fi

if [ ! -d "build/${LIBOS}" ]; then
  echo "Missing build/${LIBOS}. Run ./build.sh ${LIBOS} on the build machine first." >&2
  exit 1
fi

for host in "${HOSTS[@]}"; do
  echo "Syncing ${LIBOS} runtime files to ${host}:${REMOTE_DIR}/"
  ssh "${host}" "mkdir -p '${REMOTE_DIR}/build' '${REMOTE_DIR}/config' '${REMOTE_DIR}/test-scripts'"
  rsync -avz "build/${LIBOS}" "${host}:${REMOTE_DIR}/build/"
  rsync -avz "config/${LIBOS}" "${host}:${REMOTE_DIR}/config/"
  rsync -avz test-scripts/ "${host}:${REMOTE_DIR}/test-scripts/"
done
