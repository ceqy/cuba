#!/bin/bash
set -e

SERVICES=(
  "am/pm-service"
  "am/ah-service"
  "am/eh-service"
  "am/gs-service"
  "cs/fd-service"
  "cs/cb-service"
  "cs/wc-service"
  "rd/ps-service"
  "rd/pl-service"
  "iam/iam-service"
)

echo "=== Deploying Remaining Services ==="

for entry in "${SERVICES[@]}"; do
  IFS='/' read -r domain service <<< "$entry"
  VALUES_FILE="deploy/k8s/values/${service}.yaml"
  
  if [[ "$service" == "pm-service" && "$domain" == "am" ]]; then
     if [ -f "deploy/k8s/values/am-pm-service.yaml" ]; then
         VALUES_FILE="deploy/k8s/values/am-pm-service.yaml"
     fi
  fi
  
  if [ ! -f "$VALUES_FILE" ]; then
      echo "Warning: Values file $VALUES_FILE not found. Skipping $service."
      continue
  fi

  NAMESPACE="cuba-${domain}"
  if [[ "$domain" == "iam" ]]; then
      NAMESPACE="cuba-system"
  fi
  
  echo "Deploying $service to namespace $NAMESPACE..."
  
  helm upgrade --install "$service" deploy/k8s/charts/microservice \
    -f "$VALUES_FILE" \
    --namespace "$NAMESPACE" \
    --wait --timeout 2m
    
done

echo "=== All services deployed ==="
