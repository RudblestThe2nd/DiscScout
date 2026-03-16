# DiskMapper

A WizTree-style disk space analyzer for Linux, built with PyQt6.

---

## Features

- **Squarified Treemap** — hierarchical visualization scaled by file/folder size
- **Sortable File List** — name, size, percentage, file count, and full path columns
- **Fast Scanning** — multi-threaded scanning using `os.scandir`, non-blocking UI
- **Click-to-Navigate** — drill into directories by clicking the treemap or double-clicking the list
- **Breadcrumb Navigation** — navigate back up the directory tree with a single click
- **Color-coded Sizes** — percentage column highlighted in red, yellow, or green
- **Dark Theme** — clean dark interface with monospace typography

---

## Requirements

- Python 3.8 or higher
- PyQt6 >= 6.4.0

---

## Installation

```bash
pip3 install PyQt6
```

---

## Usage

```bash
# Launch and select a directory from the UI
python3 diskmapper.py

# Scan a specific directory on startup
python3 diskmapper.py /home
```

---

## Keyboard Shortcuts

| Action | Shortcut |
|---|---|
| Select directory | Ctrl+O |
| Go up one level | Backspace |
| Drill into directory | Single click on treemap / Double click on list |

---

## Project Structure

```
DiskMapper/
├── diskmapper.py      — main application (single file)
├── requirements.txt   — dependencies
├── install.sh         — setup script for Ubuntu/Debian
└── diskmapper.desktop — Linux application launcher
```

---

## Running as Root

To analyze the full disk:

```bash
sudo python3 diskmapper.py /
```

---

## License

MIT
