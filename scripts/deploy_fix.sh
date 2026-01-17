#!/bin/bash
# 自动部署修复到服务器

SERVER="x@10.0.0.101"
PASSWORD="x"
PROJECT_DIR="~/cuba"

echo "🚀 开始部署修复..."

# 检查是否安装sshpass
if ! command -v sshpass &> /dev/null; then
    echo "⚠️  未检测到sshpass，将手动输入密码"
    RSYNC_CMD="rsync"
    SSH_CMD="ssh"
else
    echo "✅ 使用sshpass自动输入密码"
    export SSHPASS=$PASSWORD
    RSYNC_CMD="sshpass -e rsync"
    SSH_CMD="sshpass -e ssh"
fi

echo "📦 1. 同步代码到服务器..."
$RSYNC_CMD -avz --exclude 'target' --exclude '.git' ./ $SERVER:$PROJECT_DIR/

if [ $? -ne 0 ]; then
    echo "❌ 同步失败！"
    exit 1
fi

echo "🔨 2. 在服务器上构建 (跳过coa-service)..."
$SSH_CMD $SERVER "cd $PROJECT_DIR && source ~/.cargo/env && cargo build --release --workspace --exclude coa-service"

if [ $? -ne 0 ]; then
    echo "❌ 面向服务器的构建失败"
    exit 1
fi

echo "✅ 部署与构建成功！"
