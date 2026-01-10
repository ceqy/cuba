#!/bin/bash
set -e

# ====================================================================================
# Enterprise Microservices Database Initialization
# ====================================================================================
# This script initializes the databases for all 40+ microservices across 9 product lines.
# It is intended to be run once (e.g., via docker-entrypoint-initdb.d).
# ====================================================================================

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL

    -- 0. Foundation & Auth
    CREATE DATABASE cuba_auth;
    CREATE DATABASE cuba_tenant;
    
    -- 1. Finance Domain
    CREATE DATABASE cuba_finance_gl;           -- General Ledger
    CREATE DATABASE cuba_finance_ar_ap;        -- Accounts Receivable/Payable
    CREATE DATABASE cuba_finance_co;           -- Controlling (Cost)
    CREATE DATABASE cuba_finance_treasury;     -- Treasury & Risk
    
    -- 2. Procurement Domain
    CREATE DATABASE cuba_procurement_order;    -- Purchase Orders
    CREATE DATABASE cuba_procurement_invoice;  -- Invoice Processing
    CREATE DATABASE cuba_procurement_contract; -- Contract Management
    CREATE DATABASE cuba_procurement_supplier; -- Supplier Portal
    CREATE DATABASE cuba_procurement_sourcing; -- Sourcing
    
    -- 3. Manufacturing Domain
    CREATE DATABASE cuba_manufacturing_planning;   -- Production Planning
    CREATE DATABASE cuba_manufacturing_shop_floor; -- Shop Floor Execution
    CREATE DATABASE cuba_manufacturing_kanban;     -- Kanban
    CREATE DATABASE cuba_manufacturing_quality;    -- Quality Inspection
    CREATE DATABASE cuba_manufacturing_outsourced; -- Outsourced Mfg
    
    -- 4. Supply Chain Domain
    CREATE DATABASE cuba_sc_inventory;      -- Inventory Management
    CREATE DATABASE cuba_sc_warehouse;      -- Extended Warehouse (EWM)
    CREATE DATABASE cuba_sc_transportation; -- Transportation (TM)
    CREATE DATABASE cuba_sc_traceability;   -- Batch Traceability
    CREATE DATABASE cuba_sc_forecasting;    -- Demand Forecasting
    
    -- 5. Sales Domain
    CREATE DATABASE cuba_sales_fulfillment; -- Order Fulfillment
    CREATE DATABASE cuba_sales_pricing;     -- Pricing Engine
    CREATE DATABASE cuba_sales_rev_rec;     -- Revenue Recognition
    CREATE DATABASE cuba_sales_analytics;   -- Sales Analytics
    
    -- 6. Asset Management Domain
    CREATE DATABASE cuba_asset_maintenance;     -- Asset Maintenance
    CREATE DATABASE cuba_asset_health;          -- Intelligent Asset Health
    CREATE DATABASE cuba_asset_geo;             -- Geo Spatial Service
    CREATE DATABASE cuba_asset_ehs;             -- EHS Incident Mgmt
    
    -- 7. Service Domain
    CREATE DATABASE cuba_service_dispatch; -- Field Service Dispatch
    CREATE DATABASE cuba_service_contract; -- Service Contract & Billing
    CREATE DATABASE cuba_service_warranty; -- Warranty Claims
    
    -- 8. R&D / Engineering Domain
    CREATE DATABASE cuba_rd_plm;     -- PLM Integration
    CREATE DATABASE cuba_rd_project; -- Project Cost Controlling (PS)
    
    -- 9. HR / People Domain
    CREATE DATABASE cuba_hr_employee; -- Employee Experience
    CREATE DATABASE cuba_hr_talent;   -- Talent Acquisition

EOSQL

# Optional: Enable PostGIS for Geo Service
# psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "cuba_asset_geo" <<-EOSQL
#     CREATE EXTENSION IF NOT EXISTS postgis;
# EOSQL

echo "✅ All Enterprise Databases Initialized Successfully!"
