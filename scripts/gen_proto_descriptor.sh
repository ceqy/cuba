#!/bin/bash
# Generate proto descriptor for Envoy grpc_json_transcoder

set -e

PROTO_DIR="protos"
OUTPUT_FILE="protos/foundation/iam/iam.pb"

echo "Generating proto descriptor..."

protoc \
  -I${PROTO_DIR} \
  -I${PROTO_DIR}/third_party \
  --include_imports \
  --include_source_info \
  --descriptor_set_out=${OUTPUT_FILE} \
  ${PROTO_DIR}/foundation/iam/iam.proto

echo "Proto descriptor generated at: ${OUTPUT_FILE}"
ls -la ${OUTPUT_FILE}
