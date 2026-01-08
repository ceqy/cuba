#!/bin/bash
# 生成财务服务的 Proto Descriptor 文件
# 此文件用于 Envoy gRPC-JSON Transcoder

set -e

echo "正在生成 finance/gl_journal_entry_service.pb..."

protoc -I./protos -I./protos/third_party \
  --include_imports --include_source_info \
  --descriptor_set_out=protos/finance/gl_journal_entry_service.pb \
  protos/finance/gl_journal_entry_service.proto

if [ -f "protos/finance/gl_journal_entry_service.pb" ]; then
    echo "✅ 生成成功: protos/finance/gl_journal_entry_service.pb"
    ls -lh protos/finance/gl_journal_entry_service.pb
else
    echo "❌ 生成失败"
    exit 1
fi
