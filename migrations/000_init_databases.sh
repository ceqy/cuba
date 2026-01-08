#!/bin/bash
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE cuba_finance_gl;
    CREATE DATABASE cuba_finance_ar_ap;
    CREATE DATABASE cuba_finance_treasury;
    CREATE DATABASE cuba_finance_controlling;
EOSQL
