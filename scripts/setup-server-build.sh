#!/bin/bash
# Cuba Project Server Environment Setup Script
# Run this script on the K3s server

set -e

echo "Starting Cuba project server build environment setup..."
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
   echo "Error: Please do not run this script as root"
   exit 1
fi

# Step 1: Install system dependencies
echo "Step 1: Installing system dependencies..."
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    git \
    curl \
    htop

echo "System dependencies installed"
echo ""

# Step 2: Install Rust
echo "Step 2: Installing Rust toolchain..."
if command -v rustc &> /dev/null; then
    echo "Rust already installed: $(rustc --version)"
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "Rust installed: $(rustc --version)"
fi
echo ""

# Step 3: Configure Cargo acceleration
echo "Step 3: Configuring Cargo mirror..."
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"

[build]
jobs = 8

[net]
git-fetch-with-cli = true

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
EOF
echo "Cargo configuration completed"
echo ""

# Step 4: Set environment variables
echo "Step 4: Configuring environment variables..."
cat >> ~/.bashrc << 'EOF'

# Cuba project build configuration
export CARGO_BUILD_JOBS=8
export RUSTFLAGS="-C target-cpu=native"
export CARGO_INCREMENTAL=1
export PATH="$HOME/.cargo/bin:$PATH"
EOF

source ~/.bashrc
echo "Environment variables configured"
echo ""

# Step 5: Prepare code directory
echo "Step 5: Preparing code directory..."
if [ -d "$HOME/cuba" ]; then
    echo "Warning: Directory ~/cuba already exists, skipping clone"
else
    echo "Choose code sync method:"
    echo "1) Sync from local Mac (recommended)"
    echo "2) Clone from Git repository"
    read -p "Select (1/2): " choice

    if [ "$choice" = "1" ]; then
        echo "Run the following command on your local Mac to sync code:"
        echo "rsync -avz --exclude 'target' --exclude '.git' /Users/x/x/ x@10.0.0.101:~/cuba/"
        echo ""
        read -p "Press Enter after sync is complete..."
    else
        read -p "Enter Git repository URL: " repo_url
        git clone "$repo_url" ~/cuba
    fi
fi
echo ""

# Step 6: Verify environment
echo "Step 6: Verifying environment..."
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Protoc version: $(protoc --version)"
echo "CPU cores: $(nproc)"
echo "Available memory: $(free -h | grep Mem | awk '{print $7}')"
echo ""

# Step 7: Create build scripts
echo "Step 7: Creating build scripts..."
mkdir -p ~/cuba/scripts

# Create parallel build script
cat > ~/cuba/scripts/build-all-server.sh << 'BUILDSCRIPT'
#!/bin/bash
# Server-side parallel build for all services

source ~/.cargo/env
cd ~/cuba

echo "Starting compilation of all services..."
echo "Using $(nproc) CPU cores for parallel build"
echo ""

time cargo build --release --workspace

echo ""
echo "Build completed!"
echo "Build artifacts:"
ls -lh target/release/*-service | wc -l
du -sh target/release/
BUILDSCRIPT

chmod +x ~/cuba/scripts/build-all-server.sh

# Create quick build script
cat > ~/cuba/scripts/quick-build.sh << 'QUICKSCRIPT'
#!/bin/bash
# Quick build for a single service

source ~/.cargo/env
if [ -z "$1" ]; then
    echo "Usage: ./quick-build.sh <service-name>"
    echo "Example: ./quick-build.sh ap-service"
    exit 1
fi

cd ~/cuba
echo "Building $1..."
time cargo build --release -p $1

if [ $? -eq 0 ]; then
    echo "$1 built successfully!"
    ls -lh target/release/$1
else
    echo "Build failed"
    exit 1
fi
QUICKSCRIPT

chmod +x ~/cuba/scripts/quick-build.sh

echo "Build scripts created"
echo ""

# Done
echo "========================================="
echo "Server environment setup completed!"
echo "========================================="
echo ""
echo "Next steps:"
echo ""
echo "1. If you chose rsync sync, run on local Mac:"
echo "   rsync -avz --exclude 'target' --exclude '.git' /Users/x/x/ x@10.0.0.101:~/cuba/"
echo ""
echo "2. Initial full build (takes 15-30 minutes):"
echo "   cd ~/cuba"
echo "   ./scripts/build-all-server.sh"
echo ""
echo "3. Quick build for a single service:"
echo "   ./scripts/quick-build.sh ap-service"
echo ""
echo "4. View build artifacts:"
echo "   ls -lh ~/cuba/target/release/*-service"
echo ""
echo "========================================="
