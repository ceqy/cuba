#!/usr/bin/env python3
"""
Fix database connections in Helm values files.
Updates DATABASE_URL host and generates init SQL for PostgreSQL.
"""
import os
import re
from pathlib import Path

# Use relative paths from script location
script_dir = Path(__file__).parent
project_root = script_dir.parent
values_dir = project_root / 'deploy/k8s/values'
postgres_yaml_path = project_root / 'deploy/k8s/infra/postgres.yaml'

db_names = set()
files_processed = 0

# Regex to match the DATABASE_URL and extract DB name
# matches postgres://postgres:postgres@postgres-service.cuba-fi:5432/cuba_fi_gl
url_pattern = re.compile(r'postgres://postgres:postgres@([^:]+):5432/([a-z0-9_]+)')

for filename in os.listdir(values_dir):
    if not filename.endswith('.yaml'):
        continue

    filepath = values_dir / filename
    with open(filepath, 'r') as f:
        content = f.read()

    # Extract DB name
    match = url_pattern.search(content)
    if match:
        old_host = match.group(1)
        db_name = match.group(2)
        db_names.add(db_name)

        # Replace host
        new_host = "cuba-postgres.default.svc.cluster.local"

        # Construct the new URL and replace the old one
        old_url = f"postgres://postgres:postgres@{old_host}:5432/{db_name}"
        new_url = f"postgres://postgres:postgres@{new_host}:5432/{db_name}"

        if old_host != new_host:
            new_content = content.replace(old_url, new_url)
            with open(filepath, 'w') as f:
                f.write(new_content)
            files_processed += 1
            print(f"Updated {filename}: {old_host} -> {new_host}, DB: {db_name}")

# Generate init-dbs.sql content
sql_content = ""
for db in sorted(list(db_names)):
    sql_content += f"    CREATE DATABASE {db};\n"

# Verify we got relevant DBs
if 'cuba_iam' not in db_names:
    sql_content += "    CREATE DATABASE cuba_iam;\n"

print(f"Total files processed: {files_processed}")
print("Generated SQL content len:", len(sql_content))

# Update postgres.yaml
with open(postgres_yaml_path, 'r') as f:
    pg_content = f.read()

# Replace the init-dbs.sql section
start_marker = "  init-dbs.sql: |\n"
end_marker = "---"

start_idx = pg_content.find(start_marker)
if start_idx != -1:
    end_idx = pg_content.find(end_marker, start_idx)
    if end_idx != -1:
        new_pg_content = pg_content[:start_idx + len(start_marker)] + sql_content + pg_content[end_idx:]
        with open(postgres_yaml_path, 'w') as f:
            f.write(new_pg_content)
        print("Updated postgres.yaml")
    else:
        print("Could not find end marker in postgres.yaml")
else:
    print("Could not find start marker in postgres.yaml")
