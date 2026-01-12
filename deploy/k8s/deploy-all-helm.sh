#!/bin/bash
set -e

# List of services in format "domain/service"
# Copied from build-all.sh
SERVICES=(
  "fi/gl-service"
  "fi/ap-service"
  "fi/ar-service"
  "fi/co-service"
  "fi/tr-service"
  "sd/so-service"
  "sd/pe-service"
  "sd/rr-service"
  "sd/an-service"
  "pm/po-service"
  "pm/iv-service"
  "pm/ct-service"
  "pm/sa-service"
  "pm/se-service"
  "pm/sp-service"
  "mf/pp-service"
  "mf/sf-service"
  "mf/qi-service"
  "mf/kb-service"
  "mf/om-service"
  "sc/im-service"
  "sc/wm-service"
  "sc/bt-service"
  "sc/df-service"
  "sc/tp-service"
  "sc/vs-service"
  "hr/ta-service"
  "hr/ex-service"
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

echo "=== Deploying All Services via Helm ==="

for entry in "${SERVICES[@]}"; do
  # Split domain and service
  IFS='/' read -r domain service <<< "$entry"
  
  # For am/pm-service, the service name is 'am-pm-service' in Cargo.toml but 'pm-service' in directory?
  # Wait, in build-all.sh it was "am/pm-service:am-pm-service".
  # My list just says "am/pm-service".
  # I need to be careful about the Helm release name vs Values file name.
  
  # Check values file existence
  VALUES_FILE="deploy/k8s/values/${service}.yaml"
  
  # Special case for am-pm-service if values file differs
  if [[ "$service" == "pm-service" && "$domain" == "am" ]]; then
     # Check if pm-service.yaml exists or am-pm-service.yaml
     if [ -f "deploy/k8s/values/am-pm-service.yaml" ]; then
         VALUES_FILE="deploy/k8s/values/am-pm-service.yaml"
         # Release name can be am-pm-service to avoid conflict if any?
         # But namespace is cuba-am.
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
