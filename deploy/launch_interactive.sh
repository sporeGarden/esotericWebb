#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# launch_interactive.sh — Start esotericWebb with native GUI
#
# Deploys the webb_live_interactive graph: NUCLEUS primals with petalTongue
# in `live` mode (IPC server + egui window) for a playable interactive CRPG.
#
# Usage:
#   ./deploy/launch_interactive.sh [--family ID] [--plasmidbin PATH]
#
# Prerequisites:
#   - plasmidBin depot with all primal binaries
#   - X11/Wayland session (petalTongue needs a display)
#   - BEARDOG_FAMILY_SEED env (or will be generated)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEBB_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ECO_ROOT="$(cd "$WEBB_ROOT/../.." && pwd)"

FAMILY_ID="${FAMILY_ID:-esotericwebb-interactive}"
PLASMIDBIN="${ECOPRIMALS_PLASMID_BIN:-$ECO_ROOT/infra/plasmidBin}"
GRAPH="$WEBB_ROOT/graphs/webb_live_interactive.toml"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --family) FAMILY_ID="$2"; shift 2 ;;
        --plasmidbin) PLASMIDBIN="$2"; shift 2 ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

export FAMILY_ID
export ECOPRIMALS_PLASMID_BIN="$PLASMIDBIN"

if [ -z "${BEARDOG_FAMILY_SEED:-}" ]; then
    export BEARDOG_FAMILY_SEED
    BEARDOG_FAMILY_SEED="$(head -c 32 /dev/urandom | xxd -p)"
    echo "Generated BEARDOG_FAMILY_SEED for session"
fi

SOCKET_DIR="${XDG_RUNTIME_DIR:-/tmp}/biomeos"
mkdir -p "$SOCKET_DIR"
export BIOMEOS_SOCKET_DIR="$SOCKET_DIR"

echo "=========================================="
echo "  esotericWebb — Live Interactive"
echo "=========================================="
echo "  Family:     $FAMILY_ID"
echo "  plasmidBin: $PLASMIDBIN"
echo "  Graph:      $GRAPH"
echo "  Sockets:    $SOCKET_DIR"
echo "=========================================="

if [ ! -f "$GRAPH" ]; then
    echo "ERROR: Graph not found: $GRAPH"
    exit 1
fi

if [ ! -d "$PLASMIDBIN" ]; then
    echo "ERROR: plasmidBin depot not found: $PLASMIDBIN"
    exit 1
fi

NUCLEUS_LAUNCHER="$ECO_ROOT/springs/primalSpring/nucleus_launcher.sh"
START_PRIMAL="$ECO_ROOT/springs/primalSpring/start_primal.sh"

if [ -x "$NUCLEUS_LAUNCHER" ]; then
    echo ""
    echo "Starting NUCLEUS via launcher..."
    "$NUCLEUS_LAUNCHER" --composition full start

    echo ""
    echo "NUCLEUS status:"
    "$NUCLEUS_LAUNCHER" status

    echo ""
    echo "Launching petalTongue in live mode..."
    PETALTONGUE_BIN="$PLASMIDBIN/petaltongue"
    if [ -x "$PETALTONGUE_BIN" ]; then
        exec "$PETALTONGUE_BIN" live --socket "$SOCKET_DIR/petaltongue-${FAMILY_ID}.sock"
    elif command -v petaltongue >/dev/null 2>&1; then
        exec petaltongue live --socket "$SOCKET_DIR/petaltongue-${FAMILY_ID}.sock"
    else
        echo "ERROR: petaltongue binary not found in plasmidBin or PATH"
        exit 1
    fi
elif [ -x "$START_PRIMAL" ]; then
    echo ""
    echo "Launching primals via start_primal.sh..."

    "$START_PRIMAL" beardog server &
    sleep 1
    "$START_PRIMAL" songbird server &
    sleep 1
    "$START_PRIMAL" ludospring server &

    echo ""
    echo "Launching petalTongue in live mode..."
    PETALTONGUE_BIN="$PLASMIDBIN/petaltongue"
    if [ -x "$PETALTONGUE_BIN" ]; then
        exec "$PETALTONGUE_BIN" live --socket "$SOCKET_DIR/petaltongue-${FAMILY_ID}.sock"
    elif command -v petaltongue >/dev/null 2>&1; then
        exec petaltongue live --socket "$SOCKET_DIR/petaltongue-${FAMILY_ID}.sock"
    else
        echo "ERROR: petaltongue binary not found"
        exit 1
    fi
else
    echo "ERROR: Neither nucleus_launcher.sh nor start_primal.sh found"
    echo "  Expected at: $NUCLEUS_LAUNCHER"
    exit 1
fi
