#!/bin/bash
# set -e # Don't exit on error (e.g. if DB exists)

# Base connection info
DB_HOST="localhost"
DB_PORT="5432"
DB_USER="postgres"
DB_PASS="postgres"

# List of databases to create (1-to-1 Mapping)
DBS=(
  # FI
  "cuba_fi_gl"
  "cuba_fi_ap"
  "cuba_fi_co"
  "cuba_fi_tr"
  # SC
  "cuba_sc_vs"
  "cuba_sc_df"
  "cuba_sc_wm"
  "cuba_sc_tp"
  "cuba_sc_bt"
  "cuba_sc_im"
  # PM
  "cuba_pm_se"
  "cuba_pm_sp"
  "cuba_pm_ct"
  "cuba_pm_sa"
  "cuba_pm_po"
  "cuba_pm_iv"
  # SD
  "cuba_sd_pe"
  "cuba_sd_rr"
  "cuba_sd_so"
  "cuba_sd_an"
  # MF
  "cuba_mf_pp"
  "cuba_mf_kb"
  "cuba_mf_om"
  "cuba_mf_qi"
  "cuba_mf_sf"
  # AM
  "cuba_am_pm"
  "cuba_am_gs"
  "cuba_am_ah"
  "cuba_am_eh"
  # CS
  "cuba_cs_fd"
  "cuba_cs_wc"
  "cuba_cs_cb"
  # HR
  "cuba_hr_ex"
  "cuba_hr_ta"
  # RD
  "cuba_rd_pl"
  "cuba_rd_ps"
  # Core
  "cuba_iam"
  "cuba_admin"
  "cuba_gl" # Legacy/Alias for compat
)

echo "Initializing 40+ Databases using sqlx..."

for db in "${DBS[@]}"; do
  echo "Creating $db..."
  export DATABASE_URL="postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${db}"
  cargo sqlx database create || echo "  Failed to create $db (it might already exist)"
done

echo "Done."
