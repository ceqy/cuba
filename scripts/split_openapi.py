import json
import os
import copy

INPUT_FILE = "docs/openapi/gl.openapi3.json"
OUTPUT_DIR = "docs/openapi/splits"

# Mapping from path prefix to Domain metadata
DOMAINS = {
    "finance": {
        "name": "Finance (FI)",
        "description": "Financial Management - General Ledger, Accounts Receivable/Payable, Cost Control, Treasury"
    },
    "procurement": {
        "name": "Procurement (PM)",
        "description": "Procurement Management - Purchase Orders, Contracts, Invoices, Supplier Portal"
    },
    "sales": {
        "name": "Sales (SD)",
        "description": "Sales Management - Sales Orders, Pricing, Revenue Recognition, Sales Analytics"
    },
    "supplychain": {
        "name": "Supply Chain (SC)",
        "description": "Supply Chain Management - Inventory, Warehouse, Transportation, Demand Forecasting, Batch Traceability"
    },
    "asset": {
        "name": "Asset Management (AM)",
        "description": "Asset Management - Equipment Maintenance, Health Monitoring, EHS Incidents, Geolocation"
    },
    "manufacturing": {
        "name": "Manufacturing (MF)",
        "description": "Manufacturing Management - Production Planning, Shop Floor Execution, Quality Inspection, Kanban, Outsourcing"
    },
    "service": {
        "name": "Customer Service (CS)",
        "description": "Customer Service Management - Field Dispatch, Warranty Claims, Contract Billing"
    },
    "rd": {
        "name": "R&D (RD)",
        "description": "R&D Management - PLM Integration, Project Cost Control"
    },
    "hr": {
        "name": "Human Resources (HR)",
        "description": "Human Resources Management - Talent Acquisition, Employee Experience"
    },
    "auth": {
        "name": "Identity & Access (IAM)",
        "description": "Identity & Access Management - User Authentication, Role Permissions, Two-Factor Authentication"
    },
}

# Additional tags to inject
EXTRA_TAGS = {
    "auth": [
        {"name": "Security & 2FA", "description": "Security & Two-Factor Authentication - Enable/Disable 2FA, Verify OTP"},
        {"name": "Identity Management", "description": "Identity Management - User Login, Registration, Password Reset"},
        {"name": "Role & Permission Management", "description": "Role & Permission Management - Create, Query, and Manage User Roles and Permissions"},
    ]
}

def get_used_tags(paths):
    used = set()
    for path_item in paths.values():
        for method in ["get", "post", "put", "patch", "delete"]:
            if method in path_item:
                for tag in path_item[method].get("tags", []):
                    used.add(tag)
    return used

def main():
    if not os.path.exists(OUTPUT_DIR):
        os.makedirs(OUTPUT_DIR)

    with open(INPUT_FILE, 'r') as f:
        spec = json.load(f)

    all_tags = {t["name"]: t for t in spec.get("tags", [])}

    domain_specs = {}

    for key, meta in DOMAINS.items():
        s = {
            "openapi": spec.get("openapi", "3.0.3"),
            "info": {
                "title": f"CUBA ERP - {meta['name']}",
                "description": meta["description"],
                "version": "1.0.0"
            },
            "servers": spec.get("servers", []),
            "paths": {},
            "tags": [],
            "components": spec.get("components", {})
        }
        domain_specs[key] = s

    # Assign paths to domains
    for path, path_item in spec.get("paths", {}).items():
        parts = path.split("/")
        if len(parts) > 3:
            prefix = parts[3]
            if prefix in ["users", "roles", "permissions", "policies", "admin", "oauth2", "api-keys"]:
                prefix = "auth"
            if prefix in domain_specs:
                domain_specs[prefix]["paths"][path] = path_item

    # Filter tags and inject extras
    for key, s in domain_specs.items():
        used_tags = get_used_tags(s["paths"])

        for tag_name in used_tags:
            if tag_name in all_tags:
                s["tags"].append(all_tags[tag_name])
            else:
                s["tags"].append({"name": tag_name})

        if key in EXTRA_TAGS:
            for extra in EXTRA_TAGS[key]:
                if extra["name"] in used_tags:
                    existing = next((t for t in s["tags"] if t["name"] == extra["name"]), None)
                    if existing and not existing.get("description"):
                        existing["description"] = extra["description"]
                    elif not existing:
                        s["tags"].append(extra)

    # Write output
    for key, s in domain_specs.items():
        if not s["paths"]:
            print(f"Skipping empty domain: {key}")
            continue

        outfile = os.path.join(OUTPUT_DIR, f"{key}.json")
        with open(outfile, 'w') as f:
            json.dump(s, f, indent=2, ensure_ascii=False)
        print(f"Generated {outfile} ({len(s['tags'])} tags, {len(s['paths'])} paths)")

if __name__ == "__main__":
    main()
