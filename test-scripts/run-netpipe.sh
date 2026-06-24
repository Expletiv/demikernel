#!/bin/bash
set -euo pipefail

LIBOS="${LIBOS:-catnap}"

# Startwerte (für den steilen Anstieg) + Lineare 32-KB-Schritte bis 256 KB
SIZES=(
  64       # 64 B
  1024     # 1 KB
  4096     # 4 KB
  16384    # 16 KB
  32768    # 32 KB (Ab hier linearer 32-KB-Takt)
  65536    # 64 KB
  98304    # 96 KB
  131072   # 128 KB
  163840   # 160 KB
  196608   # 192 KB
  229376   # 224 KB
  262144   # 256 KB
)

echo "Starte exakten NetPIPE-Sweep für ${LIBOS}..."

for sz in "${SIZES[@]}"; do
    echo -e "\n======================================================="
    echo "Nächste Paketgröße: $((sz / 1024)) KB (${sz} Bytes)."

    # Warteschleife auf Benutzereingabe
    while true; do
        read -r -s -n 1 -p "Drücke 'c' um diesen Run zu starten (oder 'q' zum Abbrechen): " key
        if [[ "$key" == "c" || "$key" == "C" ]]; then
            echo -e "\n--> Starte Run..."
            break
        elif [[ "$key" == "q" || "$key" == "Q" ]]; then
            echo -e "\n--> Abbruch durch Benutzer."
            exit 0
        fi
    done

    # Ausführung des bestehenden Demikernel-Clients
    BUF_SIZE=$sz REQUESTS=500000 CLIENTS=1 RUN_MODE=sequential /demikernel/test-scripts/tcp-echo-client.sh > "netpipe_sz_${sz}.txt"

    sleep 1
done