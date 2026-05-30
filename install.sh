#!/usr/bin/env bash

set -e

APP_NAME="jlog"
OS_TYPE=$(uname -s)

echo "🔍 Detecting Operating System..."
if [ "$OS_TYPE" = "Darwin" ]; then
    echo "Operating System: macOS"
    if [ -d "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="/opt/homebrew/bin"
        mkdir -p "$INSTALL_DIR"
    fi
elif [ "$OS_TYPE" = "Linux" ]; then
    echo "🐧 Operating System: Linux"
    INSTALL_DIR="/usr/local/bin"
else
    echo "Unsupported OS: $OS_TYPE"
    exit 1
fi

echo "Building $APP_NAME in release mode using Cargo..."
cargo build --release

echo "Installing $APP_NAME to $INSTALL_DIR..."

if [ -w "$INSTALL_DIR" ]; then
    cp target/release/$APP_NAME "$INSTALL_DIR/"
else
    echo "Write access denied for $INSTALL_DIR."
    sudo cp target/release/$APP_NAME "$INSTALL_DIR/"
fi

echo "Success! $APP_NAME has been installed to $INSTALL_DIR"
echo "Restart your terminal or run '$APP_NAME --help' to verify."
