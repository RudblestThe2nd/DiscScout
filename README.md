# DiskMapper-rs

A WizTree-style disk space analyzer for Linux, written in Rust.

## Features

- Parallel filesystem scanning using jwalk and rayon (up to 12x faster than single-threaded)
- Squarified treemap with recursive visualization
- WizTree-style header strips with folder names and sizes
- Resizable file list panel with sortable columns
- Hover tooltips showing folder contents
- Click to navigate into directories, Up button to go back
- Dark theme with 3D bevel blocks

## Performance

Scanned 2 million files in 1.4 seconds on a standard laptop.

## Requirements

- Linux
- Rust 1.70 or higher

## Build

```bash
cargo build --release
```

## Usage

```bash
# Launch UI
./target/release/diskmapper-rs

# Scan a specific directory on startup
./target/release/diskmapper-rs /home
```

## Keyboard Shortcuts

| Action | How |
|---|---|
| Scan directory | Click Scan or type path |
| Go up one level | Click Up button |
| Navigate into folder | Click on treemap block |

## Architecture

| Layer | Technology | Purpose |
|---|---|---|
| Scanner | jwalk + rayon | Parallel filesystem traversal |
| Tree | Arena (Vec<Node>) | Memory-efficient node storage |
| Layout | Squarify algorithm | Treemap rectangle computation |
| UI | egui + eframe | GPU-accelerated rendering |

## License

MIT
