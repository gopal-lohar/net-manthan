#!/bin/bash
# install-dev.sh

cargo build

# Install to ~/.local/bin (should be in PATH)
mkdir -p ~/.local/bin
cp target/release/net-manthan ~/.local/bin/
cp target/release/ui ~/.local/bin/net-manthan-ui

# Desktop file
mkdir -p ~/.local/share/applications
cat > ~/.local/share/applications/net_manthan.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Net Manthan Development
Comment=Launch My GPU-Accelerated Download Manager
Exec=$HOME/.local/bin/net-manthan-ui
Type=Application
Terminal=false
Categories=Network;FileTransfer;
EOF

echo "Installed to ~/.local/bin/"
echo "Run: net-manthan-ui"