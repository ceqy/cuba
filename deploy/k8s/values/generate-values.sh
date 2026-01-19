#!/bin/bash
# Generate all Helm values files for Cuba ERP services
# Usage: ./generate-values.sh

cd "$(dirname "$0")"

# Service definitions: name:port:database:namespace
SERVICES=(
  # FI Module (Finance)
  "gl-service:50060:cuba_fi_gl:cuba-fi"
  "coa-service:50065:cuba_fi_coa:cuba-fi"
  "ap-service:50061:cuba_fi_ap:cuba-fi"
  "ar-service:50062:cuba_fi_ar:cuba-fi"
  "co-service:50063:cuba_fi_co:cuba-fi"
  "tr-service:50064:cuba_fi_tr:cuba-fi"
  # SD Module (Sales)
  "so-service:50060:cuba_sd_so:cuba-sd"
  "pe-service:50061:cuba_sd_pe:cuba-sd"
  "rr-service:50062:cuba_sd_rr:cuba-sd"
  "an-service:50063:cuba_sd_an:cuba-sd"
  # PM Module (Procurement)
  "po-service:50070:cuba_pm_po:cuba-pm"
  "iv-service:50071:cuba_pm_iv:cuba-pm"
  "ct-service:50072:cuba_pm_ct:cuba-pm"
  "sa-service:50073:cuba_pm_sa:cuba-pm"
  "se-service:50082:cuba_pm_se:cuba-pm"
  "sp-service:50083:cuba_pm_sp:cuba-pm"
  # MF Module (Manufacturing)
  "pp-service:50074:cuba_mf_pp:cuba-mf"
  "sf-service:50075:cuba_mf_sf:cuba-mf"
  "qi-service:50076:cuba_mf_qi:cuba-mf"
  "kb-service:50077:cuba_mf_kb:cuba-mf"
  "om-service:50078:cuba_mf_om:cuba-mf"
  # SC Module (Supply Chain)
  "im-service:50079:cuba_sc_im:cuba-sc"
  "wm-service:50080:cuba_sc_wm:cuba-sc"
  "bt-service:50040:cuba_sc_bt:cuba-sc"
  "df-service:50081:cuba_sc_df:cuba-sc"
  "tp-service:50084:cuba_sc_tp:cuba-sc"
  "vs-service:50087:cuba_sc_vs:cuba-sc"
  # HR Module
  "ta-service:50041:cuba_hr_ta:cuba-hr"
  "ex-service:50042:cuba_hr_ex:cuba-hr"
  # AM Module (Asset)
  "pm-service:50043:cuba_am_pm:cuba-am"
  "ah-service:50083:cuba_am_ah:cuba-am"
  "eh-service:50085:cuba_am_eh:cuba-am"
  "gs-service:50086:cuba_am_gs:cuba-am"
  # CS Module (Customer Service)
  "fd-service:50044:cuba_cs_fd:cuba-cs"
  "cb-service:50045:cuba_cs_cb:cuba-cs"
  "wc-service:50046:cuba_cs_wc:cuba-cs"
  # RD Module
  "ps-service:50047:cuba_rd_ps:cuba-rd"
  "pl-service:50048:cuba_rd_pl:cuba-rd"
  # IAM
  "auth-service:50051:cuba_iam_auth:cuba-iam"
  "rbac-service:50052:cuba_iam_rbac:cuba-iam"
  "oauth-service:50053:cuba_iam_oauth:cuba-iam"
)

for service in "${SERVICES[@]}"; do
  IFS=':' read -r name port db ns <<< "$service"
  cat > "${name}.yaml" << EOF
# ${name} Helm Values
replicaCount: 2
image:
  repository: cuba-erp/${name}
  tag: "latest"
fullnameOverride: "${name}"
service:
  grpcPort: ${port}
  metricsPort: 9090

database:
  name: ${db}
  secretName: cuba-postgres-credentials

env:
  RUST_LOG: "info"

resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi
EOF
  echo "Generated ${name}.yaml"
done

echo ""
echo "=== Generated $(ls -1 *.yaml | wc -l) Helm values files ==="
