#!/bin/bash
set -e

# Create the cuba_iam database if it doesn't exist
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE cuba_iam;
EOSQL
