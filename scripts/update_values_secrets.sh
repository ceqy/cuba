#!/bin/bash
# Update all service values files to use K8s Secrets
# Removes hardcoded DATABASE_URL and adds database.name configuration

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VALUES_DIR="$SCRIPT_DIR/../deploy/k8s/values"
UPDATED_COUNT=0

echo "🔐 Updating service values files to use K8s Secrets..."
echo ""

# Service to database name mapping
declare -A DB_MAP=(
    # FI
    ["ap-service"]="cuba_fi_ap"
    ["gl-service"]="cuba_fi_gl"
    ["ar-service"]="cuba_fi_ar"
    ["co-service"]="cuba_fi_co"
    ["tr-service"]="cuba_fi_tr"
    ["coa-service"]="cuba_fi_coa"
    # IAM
    ["auth-service"]="cuba_iam"
    ["oauth-service"]="cuba_iam_oauth"
    ["rbac-service"]="cuba_iam_rbac"
    # SC
    ["bt-service"]="cuba_sc_bt"
    ["df-service"]="cuba_sc_df"
    ["im-service"]="cuba_sc_im"
    ["tp-service"]="cuba_sc_tp"
    ["vs-service"]="cuba_sc_vs"
    ["wm-service"]="cuba_sc_wm"
    # AM
    ["ah-service"]="cuba_am_ah"
    ["eh-service"]="cuba_am_eh"
    ["gs-service"]="cuba_am_gs"
    ["pm-service"]="cuba_am_pm"
    # CS
    ["cb-service"]="cuba_cs_cb"
    ["fd-service"]="cuba_cs_fd"
    ["wc-service"]="cuba_cs_wc"
    # HR
    ["ex-service"]="cuba_hr_ex"
    ["ta-service"]="cuba_hr_ta"
    # MF
    ["kb-service"]="cuba_mf_kb"
    ["om-service"]="cuba_mf_om"
    ["pp-service"]="cuba_mf_pp"
    ["qi-service"]="cuba_mf_qi"
    ["sf-service"]="cuba_mf_sf"
    # PM
    ["ct-service"]="cuba_pm_ct"
    ["iv-service"]="cuba_pm_iv"
    ["po-service"]="cuba_pm_po"
    ["sa-service"]="cuba_pm_sa"
    ["se-service"]="cuba_pm_se"
    ["sp-service"]="cuba_pm_sp"
    # RD
    ["pl-service"]="cuba_rd_pl"
    ["ps-service"]="cuba_rd_ps"
    # SD
    ["an-service"]="cuba_sd_an"
    ["pe-service"]="cuba_sd_pe"
    ["rr-service"]="cuba_sd_rr"
    ["so-service"]="cuba_sd_so"
)

for values_file in "$VALUES_DIR"/*-service.yaml; do
    if [ ! -f "$values_file" ]; then
        continue
    fi
    
    service_name=$(basename "$values_file" .yaml)
    db_name="${DB_MAP[$service_name]}"
    
    if [ -z "$db_name" ]; then
        echo "⚠️  Skipping $service_name (no DB mapping)"
        continue
    fi
    
    echo "Processing $service_name..."
    
    # Create backup
    cp "$values_file" "${values_file}.bak"
    
    # Remove DATABASE_URL from env section
    sed -i '' '/^  DATABASE_URL:/d' "$values_file"
    
    # Remove old secrets section
    sed -i '' '/^secrets:/,/^[a-z]/{ /^secrets:/d; /^  -/d; /^    /d; }' "$values_file"
    
    # Remove old envFrom section  
    sed -i '' '/^envFrom:/,/^[a-z]/{ /^envFrom:/d; /^  -/d; /^    /d; }' "$values_file"
    
    # Add database configuration after env section
    if ! grep -q "^database:" "$values_file"; then
        # Find line with "env:" and add database config after it
        awk -v db="$db_name" '
        /^env:/ {
            print
            in_env = 1
            next
        }
        in_env && /^[a-z]/ {
            print "database:"
            print "  name: " db
            print "  secretName: cuba-postgres-credentials"
            in_env = 0
        }
        { print }
        END {
            if (in_env) {
                print "database:"
                print "  name: " db
                print "  secretName: cuba-postgres-credentials"
            }
        }
        ' "${values_file}.bak" > "$values_file"
    fi
    
    # Add JWT config for IAM services
    if [[ "$service_name" == "auth-service" || "$service_name" == "oauth-service" ]]; then
        if ! grep -q "^jwt:" "$values_file"; then
            cat >> "$values_file" << EOF
jwt:
  enabled: true
  secretName: cuba-jwt-secret
  secretKey: secret-key
EOF
        fi
    fi
    
    echo "  ✓ Updated $service_name"
    ((UPDATED_COUNT++))
done

echo ""
echo "✅ Updated $UPDATED_COUNT service values files"
echo "Database credentials now managed via K8s Secrets!"
echo ""
echo "To review changes: git diff deploy/k8s/values/"
