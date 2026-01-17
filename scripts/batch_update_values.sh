#!/bin/bash
# Batch update remaining service values files

cd /Users/x/x/deploy/k8s/values

# Function to get database name for a service
get_db_name() {
    case "$1" in
        # FI
        ar-service) echo "cuba_fi_ar" ;;
        co-service) echo "cuba_fi_co" ;;
        tr-service) echo "cuba_fi_tr" ;;
        coa-service) echo "cuba_fi_coa" ;;
        # IAM
        oauth-service) echo "cuba_iam_oauth" ;;
        rbac-service) echo "cuba_iam_rbac" ;;
        # SC
        bt-service) echo "cuba_sc_bt" ;;
        df-service) echo "cuba_sc_df" ;;
        im-service) echo "cuba_sc_im" ;;
        tp-service) echo "cuba_sc_tp" ;;
        vs-service) echo "cuba_sc_vs" ;;
        wm-service) echo "cuba_sc_wm" ;;
        # AM
        ah-service) echo "cuba_am_ah" ;;
        eh-service) echo "cuba_am_eh" ;;
        gs-service) echo "cuba_am_gs" ;;
        pm-service) echo "cuba_am_pm" ;;
        # CS
        cb-service) echo "cuba_cs_cb" ;;
        fd-service) echo "cuba_cs_fd" ;;
        wc-service) echo "cuba_cs_wc" ;;
        # HR
        ex-service) echo "cuba_hr_ex" ;;
        ta-service) echo "cuba_hr_ta" ;;
        # MF
        kb-service) echo "cuba_mf_kb" ;;
        om-service) echo "cuba_mf_om" ;;
        pp-service) echo "cuba_mf_pp" ;;
        qi-service) echo "cuba_mf_qi" ;;
        sf-service) echo "cuba_mf_sf" ;;
        # PM
        ct-service) echo "cuba_pm_ct" ;;
        iv-service) echo "cuba_pm_iv" ;;
        po-service) echo "cuba_pm_po" ;;
        sa-service) echo "cuba_pm_sa" ;;
        se-service) echo "cuba_pm_se" ;;
        sp-service) echo "cuba_pm_sp" ;;
        # RD
        pl-service) echo "cuba_rd_pl" ;;
        ps-service) echo "cuba_rd_ps" ;;
        # SD
        an-service) echo "cuba_sd_an" ;;
        pe-service) echo "cuba_sd_pe" ;;
        rr-service) echo "cuba_sd_rr" ;;
        so-service) echo "cuba_sd_so" ;;
        *) echo "" ;;
    esac
}

# Update a single service values file
update_service() {
    local service=$1
    local db_name=$2
    local file="${service}.yaml"
    
    if [ ! -f "$file" ]; then
        return
    fi
    
    echo "Updating $service..."
    
    # Create new content
    cat > "${file}.new" << EOF
# ${service} Helm Values
replicaCount: 2
image:
  repository: cuba-erp/${service}
  tag: "latest"
fullnameOverride: "${service}"
service:
  grpcPort: $(grep 'grpcPort:' "$file" | awk '{print $2}')
  metricsPort: 9090

# Database configuration (credentials from Secret)
database:
  name: ${db_name}
  secretName: cuba-postgres-credentials

EOF

    # Add JWT for oauth-service
    if [ "$service" = "oauth-service" ]; then
        cat >> "${file}.new" << EOF
# JWT configuration (credentials from Secret)
jwt:
  enabled: true
  secretName: cuba-jwt-secret
  secretKey: secret-key

EOF
    fi

    # Copy env section (excluding DATABASE_URL)
    echo "env:" >> "${file}.new"
    grep -A 100 "^env:" "$file" | grep "^  " | grep -v "DATABASE_URL" >> "${file}.new" || echo "  RUST_LOG: \"info\"" >> "${file}.new"
    
    # Add resources
    cat >> "${file}.new" << EOF

resources:
  limits: {cpu: 500m, memory: 512Mi}
  requests: {cpu: 100m, memory: 128Mi}
EOF

    # Replace file
    mv "${file}.new" "$file"
    echo "  ✓ Updated $service"
}

# Process all services
count=0
for service in ar co tr coa oauth rbac bt df im tp vs wm ah eh gs pm cb fd wc ex ta kb om pp qi sf ct iv po sa se sp pl ps an pe rr so; do
    db_name=$(get_db_name "${service}-service")
    if [ -n "$db_name" ]; then
        update_service "${service}-service" "$db_name"
        ((count++))
    fi
done

echo ""
echo "✅ Updated $count service values files"
echo "All services now use K8s Secrets!"
