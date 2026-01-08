#!/bin/bash
# 生成包含所有服务的合并 Proto Descriptor

set -e

echo "正在生成合并的 Proto Descriptor..."

protoc -I./protos -I./protos/third_party \
  --include_imports --include_source_info \
  --descriptor_set_out=protos/combined_services.pb \
  protos/auth/auth_service.proto \
  protos/finance/gl_journal_entry_service.proto

if [ -f "protos/combined_services.pb" ]; then
    echo "✅ 合并成功: protos/combined_services.pb"
    ls -lh protos/combined_services.pb
else
    echo "❌ 合并失败"
    exit 1
fi
