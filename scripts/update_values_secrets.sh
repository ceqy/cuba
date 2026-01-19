#!/bin/bash
# 更新所有服务的 values 文件以使用 K8s Secrets
# 移除硬编码的 DATABASE_URL 并添加 database.name 配置

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VALUES_DIR="$SCRIPT_DIR/../deploy/k8s/values"
UPDATED_COUNT=0

echo "🔐 正在更新服务 values 文件以使用 K8s Secrets..."
echo ""

# 服务到数据库名称的映射
declare -A DB_MAP=(
    # FI
    ["ap-service"]="cuba_fi_ap"
    ["gl-service"]="cuba_fi_gl"
    ["ar-service"]="cuba_fi_ar"
    ["co-service"]="cuba_fi_co"
    ["tr-service"]="cuba_fi_tr"
    ["coa-service"]="cuba_fi_coa"
    ["uj-service"]="cuba_fi_uj"
    # IAM
    ["auth-service"]="cuba_iam"
    ["oauth-service"]="cuba_iam"
    ["rbac-service"]="cuba_iam"
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
    
    # 为 IAM 服务添加 JWT 配置
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

    echo "  ✓ 已更新 $service_name"
    ((UPDATED_COUNT++))
done

echo ""
echo "✅ 已更新 $UPDATED_COUNT 个服务 values 文件"
echo "数据库凭据现已通过 K8s Secrets 管理!"
echo ""
echo "查看更改: git diff deploy/k8s/values/"
