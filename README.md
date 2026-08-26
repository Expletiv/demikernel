# Demikernel: Anleitung zur Reproduktion

## Ordnerstruktur

* `test-scripts`: Enthält Shell-Skripte zum Ausführen der Testanwendungen.
* `build`: Enthält die kompilierten Testanwendungen mit den verschiedenen libOSes (catnap, catnip, catpowder).
* `config`: Enthält die Konfigurationsdateien für Demikernel mit einer Konfigurationsdatei für node81 und node82 für jedes libOS.
* `examples`: Enthält den Code der Testanwendungen (identisch zum Demikernel Repository, mit Ausnahme der udp-ping-pong-Anwendung).
* `results`: Enthält die rohen Messergebnisse (Konsolen-Outputs) der durchgeführten Latenz-Benchmarks (5 Runs pro Test) für die verschiedenen libOS-Varianten und DPDK.
* `build.sh`: Zum Kompilieren der Testanwendungen für ein gewähltes libOS.
* `sync.sh`: Zum Kopieren von `test-scripts`, `build` und `config` auf node81 und node82.

## 1. Serverumgebung vorbereiten

### 1.1 ConnectX-5 in den Ethernet-Modus umstellen

Falls die Netzwerkkarten auf node81 und node82 nicht im Ethernet-Modus laufen, müssen sie zunächst umgestellt werden.
Dafür wird die PCIe-Adresse der Karte benötigt. Der Modus kann mit diesen Befehlen geändert werden:

```bash
sudo apt install mstflint
sudo mstconfig -d 0000:65:00.0 set LINK_TYPE_P1=2
```

Anschließend muss das Infiniband-Subnetz gestoppt und die Firmware der Netzwerkkarte zurückgesetzt werden:

```bash
sudo pkill -9 opensm
sudo mstfwreset -d 0000:65:00.0 reset
```

Das System muss nicht neu gestartet werden und die NICs sind sofort im Ethernet-Modus.
Der Modus kann mit diesem Befehl überprüft werden:
```bash
sudo mstconfig -d 0000:65:00.0 query | grep LINK_TYPE
```

### 1.2 Netzwerkschnittstellen konfigurieren

Die NIC-Schnittstellen müssen aktiviert werden und eine statische IP-Adresse zugewiesen bekommen. 
Zum Beispiel `10.0.0.81` für node81 und `10.0.0.82` für node82.

```bash
sudo ip link set enp101s0np0 up
sudo ip addr add 10.0.0.81/24 dev enp101s0np0
```

---

## 2. Demikernel Build-Prozess

### 2.1 Systemabhängigkeiten und Rust installieren

Für die Systemabhängigkeiten und die Rust-Toolchain sind diese Installationsbefehle vorgesehen:

```bash
sudo bash scripts/install-dev-packages.sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2.2 DPDK kompilieren und installieren

Der Start des Standard-Skripts für den DPDK-Build erfolgt über:

```bash
sudo bash scripts/build-install-dpdk.sh
```

**Falls der Build fehlschlägt:**
Manuell mit diesen Befehlen neu anstoßen (entfernt ein paar nicht benötigte Treiber aus dem Build):

```bash
rm -rf build
meson --prefix=$HOME \
  -Ddefault_library=static \
  -Denable_drivers=bus/pci,bus/auxiliary,common/mlx5,net/mlx5,net/e1000,net/ixgbe,net/i40e,net/ice,net/ring,net/virtio,net/tap build
ninja -C build
ninja -C build install
```

### 2.3 Hugepages einrichten

DPDK benötigt für die Speicherverwaltung Hugepages. Dafür müssen 2 GB reserviert und als Dateisystem eingehängt werden:

```bash
# 2GB Hugepages reservieren
echo 1024 | sudo tee /sys/devices/system/node/node*/hugepages/hugepages-2048kB/nr_hugepages

# Mount-Punkt erstellen und Dateisystem einhängen
sudo mkdir -p /mnt/huge
sudo mount -t hugetlbfs -o pagesize=2M nodev /mnt/huge
```

### 2.4 Konfigurationsdateien anlegen

Im nächsten Schritt müssen die passenden Config-Dateien für die jeweilige Umgebung erstellt werden.
In ``config`` befinden sich bereits passende Konfigurationen für node81 und node82.

---

## 3. Testanwendungen kompilieren

Der Build für ein bestimmtes libOS kann direkt über das Skript `build.sh` gestartet werden.
Die kompilierten Testanwendungen werden in `build` kopiert.

```bash
bash build.sh catnip
```

### Fehlerbehebung: Probleme mit dem `vob` Crate

Sollte der Kompilierungsvorgang beim `vob`-Crate hängenbleiben, löst ein kurzes Downgrade auf eine kompatible Version das Problem. Anschließend kann der Build neu gestartet werden:

```bash
cargo update -p vob@4.0.0 --precise 3.0.6
bash build.sh
```

---

## 4. Testanwendungen ausführen

Zum Ausführen der Testanwendungen gibt es verschiedene Shell-Skripte in `test-scripts`.
Das Skript `common.sh` wird in jedem Skript eingebunden und setzt Defaultwerte für Umgebungsvariablen, die in den anderen Skripts verwendet werden.
Aufgrund der Nutzung des Real-Time Schedulers und für das Erhöhen der Prozess-Priorität müssen alle Skripte mit Root-Rechten ausgeführt werden.
Um ein anderes libOS zu testen, muss die Umgebungsvariable `LIBOS` gesetzt werden.
Die Skripts erwarten, dass der Server auf `10.0.0.82:12345` läuft.

### 4.1 TCP-Echo Anwendung ausführen

Zuerst muss der Server gestartet werden:

```bash
sudo LIBOS=catnip /demikernel/test-scripts/tcp-echo-server.sh
```

Im Client-Skript können die Anzahl der Requests und ihre Größe angepasst werden.
Per Default werden 1 Million Requests mit 64 Byte gesendet.

```bash
sudo LIBOS=catnip /demikernel/test-scripts/tcp-echo-client.sh
```

### 4.2 UDP-Echo Anwendung ausführen

Für UDP-Echo wird die leicht veränderte Testanwendung ```udp-ping-pong``` aus dem Demikernel Repository verwendet.

```bash
sudo LIBOS=catnip /demikernel/test-scripts/udp-echo-server.sh
```

Start des Clients:

```bash
sudo LIBOS=catnip /demikernel/test-scripts/udp-ping-pong-client.sh
```

## 5. Baseline Benchmark von DPDK

Für den Baseline-Benchmark wurde das Tool `testpmd` verwendet, das direkt im DPDK-SDK enthalten ist.
`testpmd` ist ein reiner L2-Packet-Forwarder, der lediglich die MAC-Adressen der Pakete austauscht und sonst keine weitere Verarbeitung ausführt.
Für die Paketgenerierung und die Messung wurde [Pktgen-DPDK](https://github.com/pktgen/Pktgen-DPDK) verwendet. 

### 5.1 Testpmd starten

Auf dem Server muss zuerst `testpmd` gestartet werden:

```bash
sudo /demikernel/compare/dpdk-testpmd -l 2,3 -n 4 -a 0000:65:00.0 -- -i --forward-mode=mac
```

* `-l 2,3`
    * Core 2 steuert die interaktive `testpmd`-Shell.
    * Core 3 wird für die Polling-Schleife (DPDK) genutzt.

Danach `start` in der testpmd-Shell ausführen.

### 5.2 Pktgen-DPDK starten

Zum Starten von Pktgen-DPDK kann dieser Befehl verwendet werden:

```bash
sudo LD_LIBRARY_PATH=/demikernel/lib/x86_64-linux-gnu:/demikernel/lib \
    /demikernel/compare/pktgen \
    -l 3-5 -n 4 \
    -d /demikernel/lib/x86_64-linux-gnu -d /demikernel/lib \
    -a 0000:65:00.0 \
    -- -P -m "[4:5].0"
```

In der Konsole müssen dann nacheinander folgende Befehle ausgeführt werden:

```
# Ziel MAC-Adresse setzen
Pktgen:> set 0 dst mac 0C:42:A1:54:77:92

# Protokoll auf UDP umstellen
Pktgen:/> set 0 proto udp

# Paketgröße auf 64 Byte setzen
Pktgen:> set 0 size 64

# Limit auf exakt 1 Million Pakete setzen
Pktgen:> set 0 count 1000000

# Tuning für Unloaded RTT
Pktgen:> set 0 burst 1
Pktgen:> set 0 rate 1

# Latenz-Messung aktivieren & Seite wechseln
Pktgen:> enable 0 latency
Pktgen:> page latency

Pktgen:> start 0
```

## 6. Messergebnisse (Rohdaten)

Die Konsolen-Outputs der fünf durchgeführten Benchmarkläufe befinden sich im Ordner `results`.
Dort sind die Ergebnisse jeweils nach dem genutzten libOS (`catnap`, `catnip`, `catpowder`) und dem verwendeten Protokoll (`tcp.md`, `udp.md`) strukturiert.
Die in den Logs ausgegebenen Latenzen (z. B. das Feld `p50` für den Median) sind in Nanosekunden angegeben.
Zusätzlich befinden sich die Ergebnisse für den reinen DPDK-Benchmark in `results/raw-dpdk`.