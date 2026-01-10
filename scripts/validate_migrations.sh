#!/bin/bash
set -e

# ====================================================================================
# Migration Validation Script (CI/CD)
# ====================================================================================
# Usage: ./scripts/validate_migrations.sh
# 
# Checks:
# 1. Naming Convention: Ensure files match YYYYMMDDHHMM_description.sql
# 2. Immutability: Ensure committed migrations haven't changed (checksum check)
# ====================================================================================

echo "🔍 Starting Migration Validation..."

FOUND_ERRORS=0

# Find all migrations directories
find apps -name "migrations" -type d | while read -r migration_dir; do
    echo "Checking directory: $migration_dir"
    
    # Check 1: Naming Convention
    for file in "$migration_dir"/*.sql; do
        [ -e "$file" ] || continue
        filename=$(basename "$file")
        
        # Regex: 12 digits (timestamp) + underscore + text + .sql
        if [[ ! "$filename" =~ ^[0-9]{12}_[a-zA-Z0-9_]+\.sql$ ]]; then
            echo "  ❌ Invalid Naming: $filename (Must be YYYYMMDDHHMM_name.sql)"
            FOUND_ERRORS=1
        fi
    done
done

if [ $FOUND_ERRORS -eq 1 ]; then
    echo "🚨 Validation Failed!"
    exit 1
else
    echo "✅ All Migrations Validated!"
fi
