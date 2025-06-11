#!/bin/sh
# generate_files.sh - Generate test files for download testing

FILES_DIR="/usr/share/nginx/html/files"

echo "Generating test files..."

# Create files directory if it doesn't exist
mkdir -p "$FILES_DIR"

# Generate extra_small.txt (1KB)
if [ ! -f "$FILES_DIR/extra_small.txt" ]; then
    echo "Creating extra_small.txt (1KB)..."
    head -c 1024 /dev/zero | tr '\0' 'A' > "$FILES_DIR/extra_small.txt"
fi

# Generate small.txt (10KB)
if [ ! -f "$FILES_DIR/small.txt" ]; then
    echo "Creating small.txt (10KB)..."
    head -c 10240 /dev/zero | tr '\0' 'B' > "$FILES_DIR/small.txt"
fi

# Generate medium.bin (1MB)
if [ ! -f "$FILES_DIR/medium.bin" ]; then
    echo "Creating medium.bin (1MB)..."
    head -c 1048576 /dev/zero | tr '\0' 'C' > "$FILES_DIR/medium.bin"
fi

# Generate large.bin (10MB)
if [ ! -f "$FILES_DIR/large.bin" ]; then
    echo "Creating large.bin (10MB)..."
    head -c 10485760 /dev/zero | tr '\0' 'D' > "$FILES_DIR/large.bin"
fi

# Generate extra_large.bin (100MB) for stress testing
if [ ! -f "$FILES_DIR/extra_large.bin" ]; then
    echo "Creating extra_large.bin (100MB)..."
    head -c 104857600 /dev/zero | tr '\0' 'E' > "$FILES_DIR/extra_large.bin"
fi

# Create a mixed content file for testing
if [ ! -f "$FILES_DIR/mixed.bin" ]; then
    echo "Creating mixed.bin (5MB with pattern)..."
    for i in $(seq 1 5120); do
        printf "Line %04d: This is test data for download manager testing.\n" $i
    done > "$FILES_DIR/mixed.bin"
fi

# Set proper permissions
chmod 644 "$FILES_DIR"/*

echo "Test files generated successfully:"
ls -lh "$FILES_DIR"