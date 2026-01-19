#!/bin/bash
# Auto-deploy fixes to remote server

SERVER="x@10.0.0.101"
PASSWORD="x"
PROJECT_DIR="~/cuba"

echo "Starting deployment..."

# Check if sshpass is installed
if ! command -v sshpass &> /dev/null; then
    echo "Warning: sshpass not found, will prompt for password"
    RSYNC_CMD="rsync"
    SSH_CMD="ssh"
else
    echo "Using sshpass for auto-authentication"
    export SSHPASS=$PASSWORD
    RSYNC_CMD="sshpass -e rsync"
    SSH_CMD="sshpass -e ssh"
fi

echo "Step 1: Syncing code to server..."
$RSYNC_CMD -avz --exclude 'target' --exclude '.git' ./ $SERVER:$PROJECT_DIR/

if [ $? -ne 0 ]; then
    echo "Error: Sync failed!"
    exit 1
fi

echo "Step 2: Building on server (excluding coa-service)..."
$SSH_CMD $SERVER "cd $PROJECT_DIR && source ~/.cargo/env && cargo build --release --workspace --exclude coa-service"

if [ $? -ne 0 ]; then
    echo "Error: Server build failed"
    exit 1
fi

echo "Deployment and build completed successfully!"
