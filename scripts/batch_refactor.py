#!/usr/bin/env python3
"""
Batch refactor all services to use cuba-service::ServiceBootstrapper
"""
import re
from pathlib import Path

# Service port mappings (extracted from main.rs files)
SERVICE_PORTS = {
    'pm-service': 50061,
    'cb-service': 50065,
    'fd-service': 50064,
    'wc-service': 50072,
    'ex-service': 50063,
    'ta-service': 50062,
    'kb-service': 50074,
    'om-service': 50080,
    'pp-service': 50058,
    'qi-service': 50060,
    'sf-service': 50059,
    'ct-service': 50076,
    'iv-service': 50069,
    'po-service': 50057,
    'sa-service': 50077,
    'se-service': 50082,
    'sp-service': 50062,
    'pl-service': 50066,
    'ps-service': 50068,
    'an-service': 50079,
    'pe-service': 50075,
    'rr-service': 50078,
    'so-service': 50055,
}

def refactor_main_rs(file_path: Path, port: int):
    """Refactor a single main.rs file"""
    content = file_path.read_text()
    
    # Check if already refactored
    if 'cuba_service::ServiceBootstrapper' in content:
        print(f"✓ {file_path.parent.parent.parent.name} already refactored")
        return False
    
    # Remove imports
    content = re.sub(r'use dotenvy::dotenv;\s*\n?', '', content)
    content = re.sub(r'use cuba_database::\{[^}]+\};\s*\n?', '', content)
    
    # Replace initialization patterns
    patterns = [
        # Pattern 1: Multi-line expanded format
        (r'cuba_telemetry::init_telemetry\(\);\s*dotenv\(\)\.ok\(\);\s*'
         r'(?:let )?addr\s*=\s*"[^"]+"\s*\.parse\(\)\?;\s*'
         r'(?:info!\([^)]+\);\s*)?'
         r'(?:let )?db_config\s*=\s*DatabaseConfig::default\(\);\s*'
         r'(?:let )?pool\s*=\s*init_pool\([^)]+\)\.await\?;',
         f'// Bootstrap Service\n    let context = cuba_service::ServiceBootstrapper::run({port}).await?;\n    let pool = context.db_pool;\n    let addr = context.addr;'),
        
        # Pattern 2: Compact single-line format
        (r'cuba_telemetry::init_telemetry\(\);?\s*dotenv\(\)\.ok\(\);?[^;]+;[^;]+init_pool[^;]+;',
         f'let context = cuba_service::ServiceBootstrapper::run({port}).await?; let pool = context.db_pool; let addr = context.addr;'),
    ]
    
    for pattern, replacement in patterns:
        new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
        if new_content != content:
            content = new_content
            break
    
    file_path.write_text(content)
    print(f"✓ Refactored {file_path.parent.parent.parent.name}")
    return True

def main():
    project_root = Path('/Users/x/x')
    refactored_count = 0
    
    for service_name, port in SERVICE_PORTS.items():
        # Find the service directory
        for module_dir in project_root.glob('apps/*'):
            service_dir = module_dir / service_name
            main_rs = service_dir / 'src' / 'main.rs'
            
            if main_rs.exists():
                if refactor_main_rs(main_rs, port):
                    refactored_count += 1
                break
    
    print(f"\n✅ Refactored {refactored_count} services!")

if __name__ == '__main__':
    main()
