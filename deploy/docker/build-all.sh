#!/bin/bash
# Build all Cuba ERP service Docker images

set -e

REGISTRY="${REGISTRY:-cuba-erp}"
TAG="${TAG:-latest}"

SERVICES=(
  "fi/gl-service:gl-service"
  "fi/ap-service:ap-service"
  "fi/ar-service:ar-service"
  "fi/co-service:co-service"
  "fi/tr-service:tr-service"
  "sd/so-service:so-service"
  "sd/pe-service:pe-service"
  "sd/rr-service:rr-service"
  "sd/an-service:an-service"
  "pm/po-service:po-service"
  "pm/iv-service:iv-service"
  "pm/ct-service:ct-service"
  "pm/sa-service:sa-service"
  "pm/se-service:se-service"
  "pm/sp-service:sp-service"
  "mf/pp-service:pp-service"
  "mf/sf-service:sf-service"
  "mf/qi-service:qi-service"
  "mf/kb-service:kb-service"
  "mf/om-service:om-service"
  "sc/im-service:im-service"
  "sc/wm-service:wm-service"
  "sc/bt-service:bt-service"
  "sc/df-service:df-service"
  "sc/tp-service:tp-service"
  "sc/vs-service:vs-service"
  "hr/ta-service:ta-service"
  "hr/ex-service:ex-service"
  "am/pm-service:am-pm-service"
  "am/ah-service:ah-service"
  "am/eh-service:eh-service"
  "am/gs-service:gs-service"
  "cs/fd-service:fd-service"
  "cs/cb-service:cb-service"
  "cs/wc-service:wc-service"
  "rd/ps-service:ps-service"
  "rd/pl-service:pl-service"
  "iam/iam-service:iam-service"
)

echo "=== Building Cuba ERP Docker Images ==="

for entry in "${SERVICES[@]}"; do
  IFS=':' read -r path name <<< "$entry"
  echo "Building ${name}..."
  docker build \
    --build-arg SERVICE_PATH="apps/${path}" \
    --build-arg SERVICE_NAME="${name}" \
    -t "${REGISTRY}/${name}:${TAG}" \
    -f deploy/docker/Dockerfile \
    .
done

echo ""
echo "=== All images built successfully ==="
echo "Push to registry with: docker push ${REGISTRY}/<service>:${TAG}"
