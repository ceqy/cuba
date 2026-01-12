import re
import sys

input_file = 'deploy/k8s/infra/istio-gateway.yaml'

with open(input_file, 'r') as f:
    content = f.read()

# Split into parts to isolate VirtualService
# We assume standard structure
parts = content.split('kind: VirtualService')
if len(parts) < 2:
    print("Could not find VirtualService")
    sys.exit(1)
    
header = parts[0] + 'kind: VirtualService'
vs_content = parts[1]

# Find the spec: http: section
http_idx = vs_content.find('  http:')
vs_meta = vs_content[:http_idx]
routes_section = vs_content[http_idx:]

# Regex to capture route blocks
# We capture entire blocks from "- match:" to the end of that route definition
# Assuming standard indentation of 2 spaces for "  - match:"
full_pattern = r'(\s+- match:\n\s+- uri:\n\s+prefix: \S+\n\s+route:\n\s+- destination:\n\s+host: [\w.-]+\n\s+port:\n\s+number: \d+)'

# We also handle blocks that MIGHT ALREADY HAVE gRPC routes from previous run (idempotency check).
# If block has multiple URIs, we just regenerate it fresh using the ORIGINAL REST URI as key.

def replace_route(match):
    block = match.group(1)
    
    # Extract existing REST URI
    uri_match = re.search(r'prefix: (/api/v1/\S+)', block)
    # Extract Host
    host_match = re.search(r'host: ([\w-]+)\.cuba-([\w-]+)\.svc\.cluster\.local', block)
    
    if not uri_match or not host_match:
        # Maybe it's already modified?
        return block
        
    old_prefix = uri_match.group(1)
    service = host_match.group(1)
    namespace_suffix = host_match.group(2)
    
    # Mapping Logic
    # svc_code = first part of service name (gl-service -> gl)
    svc_code = service.split('-')[0]
    
    # Domain Mapping
    domain_map = {
        'system': 'iam',
        'fi': 'finance',
        'sd': 'sales',
        'pm': 'procurement',
        'mf': 'manufacturing',
        'sc': 'supplychain',
        'hr': 'hr',
        'am': 'asset',
        'cs': 'customer',
        'rd': 'rd'
    }
    
    # Handle Exceptions/Specifics
    if namespace_suffix == 'system' and svc_code == 'iam':
        grpc_package = 'iam'
    elif namespace_suffix in domain_map:
        grpc_package = f"{domain_map[namespace_suffix]}.{svc_code}"
    else:
        # Fallback
        grpc_package = f"cuba.{namespace_suffix}.{svc_code}"

    grpc_prefix = f"/{grpc_package}.v1"
        
    # Construct New Block
    # Preservation of indentation is key.
    # Base indent is 2 spaces for "- match:"
    
    new_block = f"""  - match:
    - uri:
        prefix: {old_prefix}
    - uri:
        prefix: {grpc_prefix}
    route:
    - destination:
        host: {service}.cuba-{namespace_suffix}.svc.cluster.local
        port:
          number: {re.search(r'number: (\d+)', block).group(1)}"""
          
    return new_block

# To allow idempotency, we first filter out existing blocks that might check for gRPC routes?
# Or clearer: We rebuild the file assuming input is the *source* (maybe clean gateway file).
# But we overwrote it.
# If I run this script on ALREADY modified file, regex `prefix: (/api/v1/\S+)` will still match the first line?
# My regex `full_pattern` expects exactly one `- uri:`.
# If I modified it to have two `- uri:`, the regex WON'T match.
# So running it twice is safe (it acts as no-op).

new_routes_section = re.sub(full_pattern, replace_route, routes_section)

final_content = header + vs_meta + new_routes_section

with open(input_file, 'w') as f:
    f.write(final_content)

print(f"Updated {input_file} with corrected gRPC routes.")
