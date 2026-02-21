# Development Status Report: Web Label Printer

This document summarizes the architectural improvements, features implemented, and outstanding technical challenges for the Web Label Printer project.

## 1. Project Overview
The goal is to create a robust, multiplatform application for designing and printing labels specifically for Phomemo Q30/D30 series Bluetooth thermal printers. The project has transitioned from a pure web application to a Svelte-based native application powered by Tauri.

## 2. Completed Features

### 🎨 Designer Interface (Frontend)
*   **Object-Based Canvas:** Migrated to Fabric.js to allow full manipulation (move, scale, rotate, align) of text, icons, shapes, and rulers.
*   **Integrated Icon Library:** Dynamic search and insertion of over 200,000 icons via the Iconify API.
*   **Professional Typography:** 
    *   Searchable Google Fonts picker with live previews.
    *   **Built-in Font Library:** A curated fallback of ~300 popular fonts embedded directly in the app to bypass CORS/Network issues.
    *   **Paginated Browsing:** High-performance browsing of the massive font library.
*   **Precision Tools:** 
    *   Visual grid with configurable snapping.
    *   Smart alignment tools (Left, Center, Right, Top, Middle, Bottom).
    *   Dynamic ruler objects for physical measurements.
*   **Templates:** System for saving, loading, and deleting label designs using Browser LocalStorage.
*   **UI Stability:** Stabilized layout using fixed-height toolbars and a centered workspace to prevent layout shifts during editing.

### 🔌 Connectivity & Printing
*   **Web Bluetooth (Working):** Fully functional printing from modern browsers (Chrome/Edge) using the standard Web Bluetooth API.
*   **Dual-Mode Printing:** The app automatically detects its environment and switches between standard Web Bluetooth and the Native Rust Bridge.

### 🖥️ Native Application (Tauri)
*   **Multiplatform Core:** Initialized Tauri (Rust) backend to support Linux, Windows, macOS, Android, and iOS.
*   **Linux Workarounds:** 
    *   Implemented graphics hardware acceleration bypasses (`WEBKIT_DISABLE_DMABUF_RENDERER`) for stability on Fedora/NVIDIA systems.
    *   Forced X11 backend for Wayland compatibility.
*   **Custom Build System:** Configured specific port handling (5173) and strict-port enforcement to ensure frontend/backend synchronization.

## 3. Technical Challenges & Workarounds

### Bluetooth on Linux (The "BlueZ" Wall)
The primary challenge has been the restricted nature of Bluetooth in native Linux webviews. Unlike Chrome, the WebKitGTK engine does not expose the Bluetooth hardware to the frontend.
*   **Strategy 1 (btleplug):** Attempted a high-level Rust implementation. Faced "Busy" errors and D-Bus lockouts.
*   **Strategy 2 (Shell-Interactive):** Implemented a bridge that emulates a human using `bluetoothctl`. This successfully bypassed many OS-level blocks.
*   **Strategy 3 (System-Assisted):** Leveraged OS-level "Trust" and "Connect" commands before attempting GATT communication.

### Hardware Limitations
*   **Intel AX200 Firmware:** During testing, we identified kernel-level firmware crashes on the host's Intel Bluetooth adapter ("Invalid exception type"). This required hardware resets (reboots).
*   **Protocol Specifics:** Phomemo printers use a proprietary raster protocol (`1F 11 02 04`) that is sensitive to packet size (MTU) and timing.

## 4. Outstanding Issues

### 🔴 Native Printing (Tauri)
While the **Web App** is successfully printing, the **Native Tauri App** currently establishes a connection but fails to trigger the print head. 
*   **Symptoms:** Data appears to stream successfully in logs, but the printer remains silent.
*   **Suspected Cause:** Subtle timing issues in the `bluetoothctl` pipe or a mismatch in how the OS handles the BLE write requests compared to the Chrome browser.

### 🔴 Bluetooth Reliability
*   The "No more profiles to connect to" error in BlueZ remains a frequent blocker when the device state becomes stale.
*   The connection handshake between the Linux kernel and the budget Phomemo hardware is still sensitive to previous "dirty" disconnections.

## 5. Next Steps
1.  **Refine Native Bridge:** Investigate a raw L2CAP socket approach in Rust to bypass `bluetoothctl` entirely.
2.  **Protocol Sniffing:** Compare the raw byte output of the working browser version vs. the Tauri version to find the exact delta in the command stream.
3.  **Standalone Debugger:** Finalize the `debug_bt` tool to provide a 1-click "Reset & Test" utility for users experiencing hardware hangs.
