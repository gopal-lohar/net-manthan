# Net Manthan 📥

## In Development
The project is currently in active development.

## Overview

Net Manthan is a high-performance, cross-platform download manager engineered in Rust to Download FIles. it offers advanced multi-threaded downloading capabilities with seamless browser integration using extension and Native messaging.

## 🚀 Features

- **Concurrent Downloads**: Utilizes multiple concurrent threads per download
- **Cross-Platform Support**: Windows, macOS, and Linux
- **Browser Integration**: Chrome and Firefox extensions

## 🛠 Technical Architecture

The root of the repository has two directories
- `extensions` contains the browser extension for blocking the download from the browser and sending it to native messaging host
- `net-manthan-app` is a rust workspace and contains the entire application, it starts with receiving the download request from the extension using `native-mesage-host`.

### Workspace Packages

- `download-engine`: Core downloading logic and shared types, it handles the spawning, managing and cancelling of download thread (technically Green Threads as they are Tokio Tasks) and it takes a `crossbeam_channel::Sender;` as an argument over which it sends the aggregated progress of all threads
- `download-manager`: Daemon for managing download tasks, it handles the IPC over TPC and Starts the downloads using `download-engine`
- `native-message-host`: Communication bridge between browser and the `download-manager`
- `utils`: IPC implementations and utilities
- `cli`: Not complete cli app

### Key Technologies

- Rust 🦀
- Crossbeam
- Tokio
- SQLite
- Dioxus
- Browser Extension APIs

## 🔧 Installation

### Prerequisites

- Rust (latest stable version)
- Browser (Chrome/Firefox)
- Dioxus Tools (For building the UI)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/yourusername/net-manthan.git

# Navigate to project directory
cd net-manthan/net-manthan-app

# Build the workspace
cargo build --release

# this will generate a .dev directory in the net-manthan-app directory, in this .dev directory you will find com.net.manthan.json, you need to place this to the appropriate location for the native messaging host config for your operating system

# Then load the extension into yuor browser from the extensions directory in the root

# Run the Download Manager
cargo run --bin download-manager --release

# Run the ui in another terminal
cargo run --bin ui --release
```

### Usage
1. If you didn't install the browser extension you can directly download files by putting the link in the ui
2. If you did install the extension in browser it will intercept and cancel the download and send it to the download-manager

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
