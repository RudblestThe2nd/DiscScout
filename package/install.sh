#!/usr/bin/env bash
set -e

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="diskscout"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Try to find the binary next to this script first, then build it
if [ -f "${SCRIPT_DIR}/${BINARY_NAME}" ]; then
    BINARY="${SCRIPT_DIR}/${BINARY_NAME}"
elif [ -f "${SCRIPT_DIR}/../target/release/${BINARY_NAME}" ]; then
    BINARY="${SCRIPT_DIR}/../target/release/${BINARY_NAME}"
else
    echo "Binary not found. Building from source..."
    cargo build --release --manifest-path "${SCRIPT_DIR}/../Cargo.toml"
    BINARY="${SCRIPT_DIR}/../target/release/${BINARY_NAME}"
fi

echo "Installing DiskScout to ${INSTALL_DIR}..."
sudo install -m 755 "${BINARY}" "${INSTALL_DIR}/${BINARY_NAME}"

# Desktop entry
DESKTOP_FILE="/usr/share/applications/${BINARY_NAME}.desktop"
sudo tee "${DESKTOP_FILE}" > /dev/null <<EOF
[Desktop Entry]
Name=DiskScout
Comment=Disk space analyzer
Exec=${INSTALL_DIR}/${BINARY_NAME}
Terminal=false
Type=Application
Categories=Utility;System;
EOF

echo ""
echo "DiskScout installed! Run with: diskscout"
echo "Or launch from your application menu."
