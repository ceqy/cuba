#!/bin/bash
set -e

# ====================================================================================
# Database Security Initialization (RBAC)
# ====================================================================================
# Creates dedicated users for each domain/service with least privilege access.
# PASSWORD MANAGEMENT: In production, use Hashicorp Vault or cloud secret managers.
# Here we use default dev passwords for local setup.
# ====================================================================================

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL

    -- 1. Create Service Users (Idempotent)
    -- Syntax: CREATE USER x WITH PASSWORD 'y';
    -- In Postgres, we usually wrap this in a DO block to avoid error if exists, 
    -- or just catch the error. For simplicity in init script:

    DO \$\$
    BEGIN
      IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'user_gl_service') THEN
        CREATE USER user_gl_service WITH PASSWORD 'pass_gl_123';
      END IF;
      IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'user_auth_service') THEN
        CREATE USER user_auth_service WITH PASSWORD 'pass_auth_123';
      END IF;
      -- Add more users for other services as needed...
    END
    \$\$;

    -- 2. Grant Privileges
    -- Principle of Least Privilege: Service user only owns its own database.

    -- Finance GL
    GRANT ALL PRIVILEGES ON DATABASE cuba_finance_gl TO user_gl_service;
    -- In Postgres 15+, need to also grant on SCHEMA public
    \c cuba_finance_gl
    GRANT ALL ON SCHEMA public TO user_gl_service;

    -- Auth
    \c cuba_auth
    GRANT ALL ON SCHEMA public TO user_auth_service;

    -- ... Repeat for others

EOSQL

echo "✅ Database Roles & Privileges Configured!"
