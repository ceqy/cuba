import json
import os
import copy

INPUT_FILE = "docs/openapi/gl.openapi3.json"
OUTPUT_DIR = "docs/openapi/splits"

# Mapping from path prefix to Domain metadata
DOMAINS = {
    "finance": {
        "name": "财务 (Finance)",
        "description": "财务管理服务 - 总账、应收应付、成本控制、资金管理"
    },
    "procurement": {
        "name": "采购 (Procurement)",
        "description": "采购管理服务 - 采购订单、合同、发票、供应商门户"
    },
    "sales": {
        "name": "销售 (Sales)",
        "description": "销售管理服务 - 销售订单、定价、收入确认、销售分析"
    },
    "supplychain": {
        "name": "供应链 (Supply Chain)",
        "description": "供应链管理服务 - 库存、仓库、运输、需求预测、批次追溯"
    },
    "asset": {
        "name": "资产管理 (Asset)",
        "description": "资产管理服务 - 设备维护、健康监控、EHS事件、地理位置"
    },
    "manufacturing": {
        "name": "制造 (Manufacturing)",
        "description": "制造管理服务 - 生产计划、车间执行、质量检验、看板、外协"
    },
    "service": {
        "name": "客户服务 (Service)",
        "description": "客户服务管理 - 现场调度、保修索赔、合同计费"
    },
    "rd": {
        "name": "研发 (R&D)",
        "description": "研发管理服务 - PLM集成、项目成本控制"
    },
    "hr": {
        "name": "人力资源 (HR)",
        "description": "人力资源管理 - 人才招聘、员工体验"
    },
    "auth": {
        "name": "身份认证 (IAM)",
        "description": "身份与访问管理 - 用户认证、角色权限、双因素认证"
    },
}

# Additional tags to inject
EXTRA_TAGS = {
    "auth": [
        {"name": "Security & 2FA", "description": "安全与双因素认证 - 启用/禁用2FA、验证OTP"},
        {"name": "Identity Management", "description": "身份管理 - 用户登录、注册、密码重置等"},
        {"name": "Role & Permission Management", "description": "角色与权限管理 - 创建、查询和管理用户角色及权限"},
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
