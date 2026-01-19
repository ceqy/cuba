#!/bin/bash
# 自动部署修复到远程服务器
# 用法: SERVER=user@host PASSWORD=xxx ./deploy_fix.sh

SERVER="${SERVER:-user@server}"
PASSWORD="${PASSWORD:-}"
PROJECT_DIR="${PROJECT_DIR:-~/cuba}"

if [ -z "$PASSWORD" ]; then
    echo "警告: PASSWORD 未设置，将提示输入密码"
    echo "用法: SERVER=user@host PASSWORD=xxx ./deploy_fix.sh"
fi

echo "开始部署..."

# 检查是否安装了 sshpass
if ! command -v sshpass &> /dev/null; then
    echo "警告: sshpass 未找到，将提示输入密码"
    RSYNC_CMD="rsync"
    SSH_CMD="ssh"
else
    echo "使用 sshpass 自动认证"
    export SSHPASS=$PASSWORD
    RSYNC_CMD="sshpass -e rsync"
    SSH_CMD="sshpass -e ssh"
fi

echo "步骤 1: 同步代码到服务器..."
$RSYNC_CMD -avz --exclude 'target' --exclude '.git' ./ $SERVER:$PROJECT_DIR/

if [ $? -ne 0 ]; then
    echo "错误: 同步失败!"
    exit 1
fi

echo "步骤 2: 在服务器上构建 (排除 coa-service)..."
$SSH_CMD $SERVER "cd $PROJECT_DIR && source ~/.cargo/env && cargo build --release --workspace --exclude coa-service"

if [ $? -ne 0 ]; then
    echo "错误: 服务器构建失败"
    exit 1
fi

echo "部署和构建完成!"
