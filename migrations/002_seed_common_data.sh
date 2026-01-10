#!/bin/bash
set -e

# ====================================================================================
# Common Seed Data Initialization
# ====================================================================================
# Populates shared reference data like Currencies, Countries, and Timezones.
# This ensures all microservices share the same foundational data standards.
# ====================================================================================

# We assume this runs against a common 'foundation' db or replicated to specific ones.
# For this example, we populate 'cuba_finance_gl' as it requires currencies.

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "cuba_finance_gl" <<-EOSQL

    CREATE TABLE IF NOT EXISTS ref_currencies (
        code CHAR(3) PRIMARY KEY,
        name VARCHAR(50),
        symbol VARCHAR(5)
    );

    INSERT INTO ref_currencies (code, name, symbol) VALUES
        ('USD', 'US Dollar', '$'),
        ('EUR', 'Euro', '€'),
        ('CNY', 'Chinese Yuan', '¥'),
        ('JPY', 'Japanese Yen', '¥'),
        ('GBP', 'British Pound', '£')
    ON CONFLICT (code) DO NOTHING;

    -- Add more reference data...

EOSQL

echo "✅ Common Seed Data Loaded!"
