import os

# Define partitions to stay under 1MB limit
partitions = [
    {"name": "cuba-docs-1", "files": ["auth.json", "finance.json"]},
    {"name": "cuba-docs-2", "files": ["asset.json", "procurement.json"]},
    {"name": "cuba-docs-3", "files": ["sales.json", "service.json"]},
    {"name": "cuba-docs-4", "files": ["manufacturing.json", "supplychain.json"]},
    {"name": "cuba-docs-5", "files": ["hr.json", "rd.json"]}
]

base_dir = "docs/openapi/splits"

cmd = "kubectl create configmap cuba-docs-1 -n cuba-system "
for p in partitions:
    name = p["name"]
    print(f"Creating ConfigMap {name}...")
    # we use dry-run=client -o yaml to generate manifest?
    # Or just execute kubectl create.
    # User might want to apply.
    
    # Let's generate a yaml file "deploy/k8s/infra/swagger_docs.yaml"
    pass

import yaml

manifests = []

for p in partitions:
    cm = {
        "apiVersion": "v1",
        "kind": "ConfigMap",
        "metadata": {
            "name": p["name"],
            "namespace": "cuba-system"
        },
        "data": {}
    }
    
    for fname in p["files"]:
        path = os.path.join(base_dir, fname)
        if os.path.exists(path):
            with open(path, 'r') as f:
                cm["data"][fname] = f.read()
        else:
            print(f"Warning: {path} not found")
            
    manifests.append(cm)

with open("deploy/k8s/infra/swagger_docs.yaml", "w") as f:
    yaml.dump_all(manifests, f)

print("Generated deploy/k8s/infra/swagger_docs.yaml")
