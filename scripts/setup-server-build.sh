#!/bin/bash
# Cuba项目服务器端环境配置脚本
# 在K3s服务器上执行此脚本

set -e

echo "🚀 开始配置Cuba项目服务器端构建环境..."
echo ""

# 检查是否为root
if [ "$EUID" -eq 0 ]; then 
   echo "❌ 请不要使用root用户运行此脚本"
   exit 1
fi

# 步骤1: 安装系统依赖
echo "📦 步骤1: 安装系统依赖..."
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    git \
    curl \
    htop

echo "✓ 系统依赖安装完成"
echo ""

# 步骤2: 安装Rust
echo "🦀 步骤2: 安装Rust工具链..."
if command -v rustc &> /dev/null; then
    echo "Rust已安装: $(rustc --version)"
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    echo "✓ Rust安装完成: $(rustc --version)"
fi
echo ""

# 步骤3: 配置Cargo加速
echo "⚡ 步骤3: 配置Cargo镜像加速..."
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
echo "✓ Cargo配置完成"
echo ""

# 步骤4: 设置环境变量
echo "🔧 步骤4: 配置环境变量..."
cat >> ~/.bashrc << 'EOF'

# Cuba项目构建配置
export CARGO_BUILD_JOBS=8
export RUSTFLAGS="-C target-cpu=native"
export CARGO_INCREMENTAL=1
export PATH="$HOME/.cargo/bin:$PATH"
EOF

source ~/.bashrc
echo "✓ 环境变量配置完成"
echo ""

# 步骤5: 克隆代码（如果不存在）
echo "📥 步骤5: 准备代码目录..."
if [ -d "$HOME/cuba" ]; then
    echo "⚠️  目录 ~/cuba 已存在，跳过克隆"
else
    echo "请选择代码获取方式:"
    echo "1) 从本地Mac同步（推荐）"
    echo "2) 从Git仓库克隆"
    read -p "选择 (1/2): " choice
    
    if [ "$choice" = "1" ]; then
        echo "请在本地Mac执行以下命令同步代码:"
        echo "rsync -avz --exclude 'target' --exclude '.git' /Users/x/x/ x@10.0.0.101:~/cuba/"
        echo ""
        read -p "同步完成后按回车继续..."
    else
        read -p "请输入Git仓库地址: " repo_url
        git clone "$repo_url" ~/cuba
    fi
fi
echo ""

# 步骤6: 验证环境
echo "✅ 步骤6: 验证环境..."
echo "Rust版本: $(rustc --version)"
echo "Cargo版本: $(cargo --version)"
echo "Protoc版本: $(protoc --version)"
echo "CPU核心数: $(nproc)"
echo "可用内存: $(free -h | grep Mem | awk '{print $7}')"
echo ""

# 步骤7: 创建构建脚本
echo "📝 步骤7: 创建构建脚本..."
mkdir -p ~/cuba/scripts

# 创建并行构建脚本
cat > ~/cuba/scripts/build-all-server.sh << 'BUILDSCRIPT'
#!/bin/bash
# 服务器端并行构建所有服务

cd ~/cuba

echo "🔨 开始编译所有服务..."
echo "使用 $(nproc) 个CPU核心并行构建"
echo ""

time cargo build --release --workspace

echo ""
echo "✅ 构建完成！"
echo "构建产物:"
ls -lh target/release/*-service | wc -l
du -sh target/release/
BUILDSCRIPT

chmod +x ~/cuba/scripts/build-all-server.sh

# 创建快速更新脚本
cat > ~/cuba/scripts/quick-build.sh << 'QUICKSCRIPT'
#!/bin/bash
# 快速构建单个服务

if [ -z "$1" ]; then
    echo "用法: ./quick-build.sh <service-name>"
    echo "例如: ./quick-build.sh ap-service"
    exit 1
fi

cd ~/cuba
echo "🔨 构建 $1..."
time cargo build --release -p $1

if [ $? -eq 0 ]; then
    echo "✅ $1 构建成功！"
    ls -lh target/release/$1
else
    echo "❌ 构建失败"
    exit 1
fi
QUICKSCRIPT

chmod +x ~/cuba/scripts/quick-build.sh

echo "✓ 构建脚本创建完成"
echo ""

# 完成
echo "========================================="
echo "🎉 服务器环境配置完成！"
echo "========================================="
echo ""
echo "📋 下一步操作:"
echo ""
echo "1. 如果选择了rsync同步，在本地Mac执行:"
echo "   rsync -avz --exclude 'target' --exclude '.git' /Users/x/x/ x@10.0.0.101:~/cuba/"
echo ""
echo "2. 首次完整构建（需要15-30分钟）:"
echo "   cd ~/cuba"
echo "   ./scripts/build-all-server.sh"
echo ""
echo "3. 快速构建单个服务:"
echo "   ./scripts/quick-build.sh ap-service"
echo ""
echo "4. 查看构建产物:"
echo "   ls -lh ~/cuba/target/release/*-service"
echo ""
echo "========================================="
