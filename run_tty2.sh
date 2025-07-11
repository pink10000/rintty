#!/bin/bash

set -e  

TTY_DEVICE="/dev/tty2"
TTY_NAME="tty2"

echo "Looking for existing processes on $TTY_NAME..."

# Stop systemd getty service for tty2 (this is cleaner than killing processes)
echo "Stopping systemd getty service for $TTY_NAME..."
sudo systemctl stop "getty@$TTY_NAME.service" 2>/dev/null || echo "Getty service not running or already stopped"

# Find and kill any existing rintty process for tty2
RINTTY_PID=$(ps aux | grep "rintty.*$TTY_DEVICE" | grep -v grep | awk '{print $2}')
if [ -n "$RINTTY_PID" ]; then
    echo "Found rintty process with PID: $RINTTY_PID"
    echo "Killing existing rintty on $TTY_NAME..."
    sudo kill "$RINTTY_PID"
    sleep 1
    if ps -p "$RINTTY_PID" > /dev/null 2>&1; then
        echo "Process still running, force killing..."
        sudo kill -9 "$RINTTY_PID"
    fi
    echo "rintty killed successfully"
else
    echo "No existing rintty process found on $TTY_NAME"
fi

echo "Clearing terminal and starting rintty on $TTY_DEVICE..."

# `\033[2J` clears the screen, 
# `\033[H` moves the cursor to the top-left corner.
sudo sh -c "echo -e '\033[2J\033[H' > $TTY_DEVICE" 2>/dev/null || true 

if [ ! -f "target/release/rintty" ] || [ "src/main.rs" -nt "target/release/rintty" ]; then
    echo "Building rintty..."
    cargo build --release
fi

# Run rintty on tty2
sudo ./target/release/rintty "$TTY_DEVICE" 