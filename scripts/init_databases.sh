#!/bin/bash
# set -e # Don't exit on error (e.g. if DB exists)

# Base connection info
DB_HOST="localhost"
DB_PORT="5432"
DB_USER="postgres"
DB_PASS="postgres"

# List of databases to create
DBS=(
  "cuba_iam"
  "cuba_admin"
  "cuba_gl"
  "cuba_ap"
  "cuba_ar"
  "cuba_aa"
  "cuba_co"
  "cuba_mm"
  "cuba_wm"
  "cuba_pp"
  "cuba_sd"
  "cuba_cs"
  "cuba_hr"
  "cuba_pm"
  "cuba_qm"
  "cuba_rd"
  "cuba_ps"
  "cuba_tr"
)

echo "Initializing Databases using sqlx..."

for db in "${DBS[@]}"; do
  echo "Creating $db..."
  export DATABASE_URL="postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${db}"
  cargo sqlx database create || echo "  Failed to create $db (it might already exist)"
done

echo "Done."
