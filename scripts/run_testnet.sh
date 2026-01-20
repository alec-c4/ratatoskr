#!/bin/bash

# Kill background processes on exit
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

echo "🦀 Building Project..."

# Build Server (cargo is fine here)
cargo build --package ratatoskr-server

# Build Desktop using Tauri CLI to ensure assets are bundled
echo "🖥️  Building Desktop App..."
cd ratatoskr-desktop
# We use --debug to speed up build and keep devtools
npm run tauri build -- --debug
cd ..

# Paths
SERVER_BIN="./target/debug/ratatoskr-server"
# Tauri build output location depends on OS. For macOS:
DESKTOP_BIN="./ratatoskr-desktop/src-tauri/target/debug/bundle/macos/ratatoskr-desktop.app/Contents/MacOS/ratatoskr-desktop"

# Check if binary exists, otherwise fallback to standard target (Linux/Windows)
if [ ! -f "$DESKTOP_BIN" ]; then
    DESKTOP_BIN="./target/debug/ratatoskr-desktop"
fi

# Data Dirs
ALICE_DIR="/tmp/ratatoskr_alice"
BOB_DIR="/tmp/ratatoskr_bob"

rm -rf $ALICE_DIR $BOB_DIR
mkdir -p $ALICE_DIR $BOB_DIR

echo "🚀 Starting Relay Server..."
$SERVER_BIN &
SERVER_PID=$!
sleep 2

echo "👤 Starting Alice (Client A)..."
export RATATOSKR_CONFIG_DIR=$ALICE_DIR
$DESKTOP_BIN &

echo "👤 Starting Bob (Client B)..."
export RATATOSKR_CONFIG_DIR=$BOB_DIR
$DESKTOP_BIN &

echo "✅ Testnet Running!"
echo "   Alice Data: $ALICE_DIR"
echo "   Bob Data:   $BOB_DIR"
echo "   Press Ctrl+C to stop everything."

wait
