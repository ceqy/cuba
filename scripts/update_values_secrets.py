#!/usr/bin/env python3
"""
Update all service values files to use new Secret-based configuration
Replaces hardcoded DATABASE_URL with database.name configuration
"""
import os
import yaml
from pathlib import Path

# Service to database name mapping
SERVICE_DB_MAP = {
    # FI module
    'ap-service': 'cuba_fi_ap',
    'gl-service': 'cuba_fi_gl',
    'ar-service': 'cuba_fi_ar',
    'co-service': 'cuba_fi_co',
    'tr-service': 'cuba_fi_tr',
    'coa-service': 'cuba_fi_coa',
    
    # IAM module
    'auth-service': 'cuba_iam',
    'oauth-service': 'cuba_iam_oauth',
    'rbac-service': 'cuba_iam_rbac',
    
    # SC module
    'bt-service': 'cuba_sc_bt',
    'df-service': 'cuba_sc_df',
    'im-service': 'cuba_sc_im',
    'tp-service': 'cuba_sc_tp',
    'vs-service': 'cuba_sc_vs',
    'wm-service': 'cuba_sc_wm',
    
    # AM module
    'ah-service': 'cuba_am_ah',
    'eh-service': 'cuba_am_eh',
    'gs-service': 'cuba_am_gs',
    'pm-service': 'cuba_am_pm',
    
    # CS module
    'cb-service': 'cuba_cs_cb',
    'fd-service': 'cuba_cs_fd',
    'wc-service': 'cuba_cs_wc',
    
    # HR module
    'ex-service': 'cuba_hr_ex',
    'ta-service': 'cuba_hr_ta',
    
    # MF module
    'kb-service': 'cuba_mf_kb',
    'om-service': 'cuba_mf_om',
    'pp-service': 'cuba_mf_pp',
    'qi-service': 'cuba_mf_qi',
    'sf-service': 'cuba_mf_sf',
    
    # PM module
    'ct-service': 'cuba_pm_ct',
    'iv-service': 'cuba_pm_iv',
    'po-service': 'cuba_pm_po',
    'sa-service': 'cuba_pm_sa',
    'se-service': 'cuba_pm_se',
    'sp-service': 'cuba_pm_sp',
    
    # RD module
    'pl-service': 'cuba_rd_pl',
    'ps-service': 'cuba_rd_ps',
    
    # SD module
    'an-service': 'cuba_sd_an',
    'pe-service': 'cuba_sd_pe',
    'rr-service': 'cuba_sd_rr',
    'so-service': 'cuba_sd_so',
}

IAM_SERVICES = {'auth-service', 'oauth-service'}

def update_values_file(file_path: Path, service_name: str):
    """Update a single values file"""
    print(f"Processing {service_name}...")
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Parse YAML
    try:
        data = yaml.safe_load(content) or {}
    except:
        print(f"  Warning: Could not parse {file_path}, skipping")
        return False
    
    # Add database configuration
    db_name = SERVICE_DB_MAP.get(service_name)
    if db_name:
        data['database'] = {
            'name': db_name,
            'secretName': 'cuba-postgres-credentials'
        }
    
    # Add JWT config for IAM services
    if service_name in IAM_SERVICES:
        data['jwt'] = {
            'enabled': True,
            'secretName': 'cuba-jwt-secret',
            'secretKey': 'secret-key'
        }
    
    # Remove old DATABASE_URL from env
    if 'env' in data:
        data['env'].pop('DATABASE_URL', None)
    
    # Remove old secrets/envFrom sections
    data.pop('secrets', None)
    data.pop('envFrom', None)
    
    # Write back
    with open(file_path, 'w') as f:
        yaml.dump(data, f, default_flow_style=False, sort_keys=False, allow_unicode=True)
    
    print(f"  ✓ Updated {service_name}")
    return True

def main():
    values_dir = Path('/Users/x/x/deploy/k8s/values')
    updated_count = 0
    
    for values_file in values_dir.glob('*-service.yaml'):
        service_name = values_file.stem
        if update_values_file(values_file, service_name):
            updated_count += 1
    
    print(f"\n✅ Updated {updated_count} service values files")
    print("Database credentials are now managed via K8s Secrets!")

if __name__ == '__main__':
    main()
