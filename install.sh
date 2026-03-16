#!/bin/bash
# DiskMapper — Kurulum & Çalıştırma Betiği
# Ubuntu / Debian için

set -e

echo "════════════════════════════════════"
echo "  DiskMapper — Kurulum Başlıyor"
echo "════════════════════════════════════"

# Python kontrolü
if ! command -v python3 &>/dev/null; then
    echo "❌  Python3 bulunamadı. Kurulum:"
    echo "    sudo apt install python3 python3-pip"
    exit 1
fi

# pip kurulumu
echo "📦  PyQt6 yükleniyor..."
pip3 install --break-system-packages PyQt6 2>/dev/null || pip3 install PyQt6

echo ""
echo "✅  Kurulum tamamlandı!"
echo ""
echo "▶️   Çalıştırmak için:"
echo "    python3 diskmapper.py"
echo ""
echo "    Veya doğrudan dizin taramak için:"
echo "    python3 diskmapper.py /home"
echo ""
