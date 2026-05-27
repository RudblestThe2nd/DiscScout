#!/usr/bin/env bash
set -e

NAME="diskscout"
VERSION="0.1.0"
ARCH="amd64"
OUT_DIR="$(cd "$(dirname "$0")" && pwd)/dist"
PKG="${NAME}_${VERSION}_${ARCH}"

echo "==> Building release binary..."
cargo build --release --manifest-path "$(dirname "$0")/../Cargo.toml"

BINARY="$(dirname "$0")/../target/release/${NAME}"

echo "==> Creating package structure..."
mkdir -p "${OUT_DIR}/${PKG}/DEBIAN"
mkdir -p "${OUT_DIR}/${PKG}/usr/local/bin"
mkdir -p "${OUT_DIR}/${PKG}/usr/share/applications"
mkdir -p "${OUT_DIR}/${PKG}/usr/share/doc/${NAME}"

# Binary
cp "${BINARY}" "${OUT_DIR}/${PKG}/usr/local/bin/${NAME}"
chmod 755 "${OUT_DIR}/${PKG}/usr/local/bin/${NAME}"

# control file
cat > "${OUT_DIR}/${PKG}/DEBIAN/control" <<EOF
Package: ${NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Berkay Parcal <berkayparcal@outlook.com>
Description: WizTree-style disk space analyzer for Linux
 DiskScout scans your filesystem in parallel and visualizes
 disk usage as an interactive squarified treemap.
EOF

# Desktop entry
cat > "${OUT_DIR}/${PKG}/usr/share/applications/${NAME}.desktop" <<EOF
[Desktop Entry]
Name=DiskScout
Comment=Disk space analyzer
Exec=/usr/local/bin/${NAME}
Icon=${NAME}
Terminal=false
Type=Application
Categories=Utility;System;
EOF

# Copyright
cp "$(dirname "$0")/../LICENSE" "${OUT_DIR}/${PKG}/usr/share/doc/${NAME}/copyright"

echo "==> Building .deb..."
dpkg-deb --build --root-owner-group "${OUT_DIR}/${PKG}"

echo ""
echo "Done! Package: ${OUT_DIR}/${PKG}.deb"
