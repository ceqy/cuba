import os
import re

ROOT_DIR = "protos"

def get_new_package_name(filepath):
    rel_path = os.path.relpath(filepath, ROOT_DIR)
    parts = rel_path.split(os.sep)
    
    if len(parts) == 3 and parts[2].endswith('.proto'):
        # protos/fi/gl/gl.proto -> fi.gl.v1
        return f"{parts[0]}.{parts[1]}.v1"
    elif len(parts) == 2 and parts[1].endswith('.proto'):
        # protos/common/common.proto -> common.v1
        # protos/iam/iam.proto -> iam.v1
        return f"{parts[0]}.v1"
    elif len(parts) == 2 and parts[0] == "common" and parts[1] == "common.proto":
         return "common.v1"
    
    return None

def standardize_file(filepath):
    pkg_name = get_new_package_name(filepath)
    if not pkg_name:
        print(f"Skipping {filepath}: Could not determine package name")
        return

    print(f"Processing {filepath} -> {pkg_name}")
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    lines = content.splitlines()
    new_lines = []
    
    for line in lines:
        stripped = line.strip()
        
        # Remove options
        if stripped.startswith("option go_package"): continue
        if stripped.startswith("option java_package"): continue
        if stripped.startswith("option java_multiple_files"): continue
        if stripped.startswith("option java_outer_classname"): continue
        if stripped.startswith("option csharp_namespace"): continue
        if stripped.startswith("option objc_class_prefix"): continue
        if stripped.startswith("option php_namespace"): continue
        if stripped.startswith("option ruby_package"): continue
        if stripped.startswith("option swift_prefix"): continue
        
        # Update package declaration
        if stripped.startswith("package "):
            new_lines.append(f"package {pkg_name};")
            continue
            
        # Global Replacements
        # enterprise.common -> common.v1
        line = line.replace("enterprise.common.", "common.v1.")
        # enterprise.events -> events.v1
        line = line.replace("enterprise.events.", "events.v1.")
        
        # Specific fix for existing files that might reference old common package without trailing dot
        line = line.replace("enterprise.common ", "common.v1 ")
        
        new_lines.append(line)
        
    # Validation: Ensure package line exists if it was missing (unlikely but safe to check)
    # Re-read new_lines to check for package
    has_pkg = any(l.strip().startswith("package ") for l in new_lines)
    if not has_pkg:
        # Insert after syntax
        final_lines = []
        inserted = False
        for line in new_lines:
            final_lines.append(line)
            if line.strip().startswith("syntax ="):
                final_lines.append(f"\npackage {pkg_name};")
                inserted = True
        if not inserted:
            final_lines.insert(0, f"package {pkg_name};")
        new_lines = final_lines

    with open(filepath, 'w') as f:
        f.write("\n".join(new_lines) + "\n")

def main():
    for root, dirs, files in os.walk(ROOT_DIR):
        for file in files:
            if file.endswith(".proto"):
                standardize_file(os.path.join(root, file))

if __name__ == "__main__":
    main()
