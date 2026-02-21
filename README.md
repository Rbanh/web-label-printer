# Web Label Printer

A modern, cross-platform desktop application for designing and printing labels for Phomemo Q30/D30 series Bluetooth thermal printers. Built with Tauri, Svelte 5, and Fabric.js.

![Web Label Printer Screenshot](AppIcon.png)

## Features

### Design Canvas
- **Object-Based Editing**: Full manipulation support - move, scale, rotate, and align text, icons, shapes, and rulers
- **Integrated Icon Library**: Search and insert from over 200,000 icons via the Iconify API
- **Professional Typography**:
  - Searchable Google Fonts picker with live previews
  - Built-in font library with ~300 popular fonts (works offline)
  - Paginated browsing for high performance
- **Precision Tools**:
  - Visual grid with configurable snapping
  - Smart alignment tools (Left, Center, Right, Top, Middle, Bottom)
  - Dynamic ruler objects for physical measurements
- **Templates**: Save, load, and delete label designs using LocalStorage

### Connectivity & Printing
- **Multi-Platform Bluetooth**: Works on Linux, Windows, macOS (via native Tauri backend)
- **Web Bluetooth Fallback**: Also works in modern browsers (Chrome/Edge) without installation
- **Dual-Mode Printing**: Automatically detects environment and switches between Web Bluetooth and Native Rust bridge
- **Phomemo Protocol Support**: Full support for the proprietary raster protocol used by Phomemo printers

### Native Application
- **Cross-Platform**: Linux, Windows, macOS support via Tauri
- **Flatpak Support**: Easy installation on Linux distributions
- **Hardware Workarounds**: Built-in fixes for NVIDIA/Wayland compatibility issues

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform from the [Releases](https://github.com/Rbanh/web-label-printer/releases) page:

| Platform | Download |
|----------|----------|
| Linux (AppImage) | `web-label-printer_*_amd64.AppImage` |
| Linux (DEB) | `web-label-printer_*_amd64.deb` |
| Linux (Flatpak) | `com.phomemo.weblabelprinter.flatpak` |
| Windows | `web-label-printer_*_x64-setup.exe` |
| macOS (Intel) | `web-label-printer_*_x64.dmg` |
| macOS (Apple Silicon) | `web-label-printer_*_aarch64.dmg` |

### Install Flatpak (Linux)

```bash
# Download the .flatpak file from releases
flatpak install --user com.phomemo.weblabelprinter.flatpak

# Run
flatpak run com.phomemo.weblabelprinter
```

### Build from Source

#### Prerequisites
- Node.js 22+
- Rust 1.77+
- Platform-specific dependencies:
  - **Linux**: `webkit2gtk-4.1`, `gtk3`, `openssl`, `libsoup3`
  - **Windows**: Microsoft Visual Studio Build Tools
  - **macOS**: Xcode Command Line Tools

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/Rbanh/web-label-printer.git
cd web-label-printer

# Install dependencies
npm install

# Development mode
npm run dev:linux   # Linux (with hardware workarounds)
npm run dev         # Windows/macOS

# Build for production
npm run tauri build
```

#### Build Flatpak (Linux)

```bash
npm run flatpak
```

## Usage

### Connecting to Printer

1. Ensure your Phomemo Q30/D30 printer is powered on
2. Click "Select Printer" in the app
3. Choose your printer from the list
4. The printer will be remembered for future sessions

### Designing Labels

1. Use the toolbar to add text, icons, shapes, or rulers
2. Select objects by clicking on them
3. Drag to move, use corner handles to resize
4. Use alignment tools for precise positioning
5. Enable grid for snap-to-grid functionality

### Printing

1. Design your label
2. Click "Print Label" (or press `P`)
3. The app will connect to your printer and send the label

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `P` | Print Label |
| `Delete` | Delete selected object |
| `Ctrl/Cmd + S` | Save template |
| `Ctrl/Cmd + Z` | Undo |

## Technical Details

### Architecture

```
┌─────────────────────────────────────────────┐
│           Svelte 5 Frontend                 │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐   │
│  │ Fabric.js│ │ Iconify │ │ Google Fonts│   │
│  │ Canvas   │ │ API     │ │ API         │   │
│  └─────────┘ └─────────┘ └─────────────┘   │
└─────────────────┬───────────────────────────┘
                  │ Tauri IPC
┌─────────────────▼───────────────────────────┐
│           Rust Backend (Tauri)              │
│  ┌─────────────┐ ┌─────────────────────┐   │
│  │ btleplug    │ │ Native Bluetooth    │   │
│  │ (BlueZ/     │ │ Protocol Handler    │   │
│  │ CoreBluetooth)│                    │   │
│  └─────────────┘ └─────────────────────┘   │
└─────────────────────────────────────────────┘
```

### Bluetooth Protocol

The Phomemo printers use a proprietary raster protocol:
- Service UUID: `0000ff00-0000-1000-8000-00805f9b34fb`
- Write Characteristic: `0000ff02-0000-1000-8000-00805f9b34fb`
- Protocol: ESC/POS with custom raster commands

### Project Structure

```
web-label-printer/
├── src/                    # Svelte frontend
│   ├── App.svelte         # Main application component
│   └── main.ts            # Entry point
├── src-tauri/             # Tauri/Rust backend
│   ├── src/
│   │   ├── lib.rs         # Bluetooth & printing logic
│   │   └── main.rs        # Application entry
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── flatpak/               # Flatpak packaging
│   ├── com.phomemo.weblabelprinter.yaml
│   └── build.sh
├── .github/workflows/     # CI/CD pipelines
│   └── release.yml        # Multi-platform builds
└── package.json           # Node dependencies
```

## Troubleshooting

### Linux: Blank window or crashes on NVIDIA

The app includes workarounds for NVIDIA driver issues. If you still encounter problems:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 GDK_BACKEND=x11 web-label-printer
```

### Linux: Bluetooth connection issues

1. Ensure BlueZ is running: `systemctl status bluetooth`
2. Make sure printer is not paired in system Bluetooth settings
3. Check printer is discoverable (LED should be flashing)

### Printer not found

1. Ensure printer is powered on and not connected to another device
2. Try disconnecting from system Bluetooth settings first
3. Restart the app and try again

### Print quality issues

The app uses ESC/POS raster protocol optimized for Phomemo printers. If labels appear distorted:
1. Check your label width setting matches physical label size
2. Ensure sufficient label height for your content

## Development

### Key Technologies

- **Frontend**: Svelte 5, TypeScript, Vite/Rolldown
- **Canvas**: Fabric.js for object-oriented design
- **Backend**: Tauri 2.x, Rust
- **Bluetooth**: btleplug (cross-platform BLE)
- **Packaging**: Flatpak, NSIS, DMG

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Acknowledgments

- **Original Project**: [dmsc/web-label-printer](https://github.com/dmsc/web-label-printer) - The foundation this fork builds upon
- This fork was developed with AI assistance, adding native Bluetooth support, Flatpak packaging, and cross-platform CI/CD

## License

MIT License - see [LICENSE](LICENSE) for details.

## Related Projects

- [Phomemo Protocol Documentation](https://github.com/vivier/phomemo)
- [Tauri](https://tauri.app/)
- [Fabric.js](http://fabricjs.com/)
