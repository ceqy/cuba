#!/usr/bin/env python3
"""
自动为 Proto 文件中的所有 RPC 添加 google.api.http 注解 (批量版)
用法: python3 scripts/add_http_annotations.py protos/**/*_service.proto
      python3 scripts/add_http_annotations.py --all  (处理所有服务)
"""

import re
import sys
import glob
from pathlib import Path

# Domain 映射到 API 路径前缀
DOMAIN_PATH_MAP = {
    "auth": "/api/v1/auth",
    "finance": "/api/v1/finance",
    "procurement": "/api/v1/procurement",
    "manufacturing": "/api/v1/manufacturing",
    "supplychain": "/api/v1/supplychain",
    "sales": "/api/v1/sales",
    "asset": "/api/v1/asset",
    "service": "/api/v1/service",
    "rd": "/api/v1/rd",
    "hr": "/api/v1/hr",
}

def infer_base_path(package_name, file_path):
    """根据包名或文件路径推断 API 基础路径"""
    # 尝试从文件路径提取 domain
    path_str = str(file_path)
    for domain in DOMAIN_PATH_MAP:
        if f"/protos/{domain}/" in path_str or f"\\protos\\{domain}\\" in path_str:
            return DOMAIN_PATH_MAP[domain]
    
    # 从包名推断
    for domain in DOMAIN_PATH_MAP:
        if domain in package_name.lower():
            return DOMAIN_PATH_MAP[domain]
    
    return "/api/v1"  # 默认


def infer_http_method_and_path(rpc_name, request_type, base_path):
    """根据 RPC 名称推断 HTTP 方法和路径"""
    
    # 创建操作 - POST
    if rpc_name.startswith("Create"):
        resource = rpc_name.replace("Create", "")
        path_segment = _to_kebab_case(resource)
        return ("post", f"{base_path}/{path_segment}", "*")
    
    # 批量创建 - POST
    if rpc_name.startswith("BatchCreate"):
        resource = rpc_name.replace("BatchCreate", "")
        path_segment = _to_kebab_case(resource)
        return ("post", f"{base_path}/{path_segment}/batch", "*")
    
    # 获取单个资源 - GET (无路径参数，使用查询参数)
    if rpc_name.startswith("Get") and "List" not in rpc_name:
        resource = rpc_name.replace("Get", "")
        path_segment = _to_kebab_case(resource)
        # 使用查询参数而非路径参数，避免字段名不匹配
        return ("get", f"{base_path}/{path_segment}", None)
    
    # 列表查询 - GET
    if rpc_name.startswith("List") or rpc_name.startswith("Search") or rpc_name.startswith("Stream"):
        resource = rpc_name.replace("List", "").replace("Search", "").replace("Stream", "")
        path_segment = _to_kebab_case(resource) if resource else "items"
        return ("get", f"{base_path}/{path_segment}", None)
    
    # 更新操作 - PUT (使用 body 传递 ID)
    if rpc_name.startswith("Update"):
        resource = rpc_name.replace("Update", "")
        path_segment = _to_kebab_case(resource)
        return ("put", f"{base_path}/{path_segment}", "*")
    
    # 删除操作 - DELETE (使用查询参数传递 ID)
    if rpc_name.startswith("Delete"):
        resource = rpc_name.replace("Delete", "")
        path_segment = _to_kebab_case(resource)
        return ("delete", f"{base_path}/{path_segment}", None)
    
    # 动作型操作 - POST with action
    action_verbs = [
        "Post", "Reverse", "Cancel", "Reset", "Clear", "Validate", "Simulate",
        "Approve", "Reject", "Submit", "Execute", "Process", "Upload", "Download",
        "Export", "Generate", "Park", "Reconcile", "Revaluate", "Reclassify",
        "CarryForward", "Configure", "Adjust", "Recalculate", "Save", "Enable",
        "Disable", "Verify", "Check", "Enter", "Import", "Trigger", "Start",
        "Stop", "Complete", "Confirm", "Assign", "Revoke", "Add", "Remove",
        "Calculate", "Optimize", "Dispatch", "Register", "Release"
    ]
    
    for verb in action_verbs:
        if rpc_name.startswith(verb):
            resource = rpc_name.replace(verb, "")
            action = _to_kebab_case(verb)
            
            if resource:
                path_segment = _to_kebab_case(resource)
                return ("post", f"{base_path}/{path_segment}/{action}", "*")
            else:
                return ("post", f"{base_path}/{action}", "*")
    
    # 默认：POST
    return ("post", f"{base_path}/{_to_kebab_case(rpc_name)}", "*")


def _to_kebab_case(text):
    """将 PascalCase 转换为 kebab-case"""
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1-\2', text)
    return re.sub('([a-z0-9])([A-Z])', r'\1-\2', s1).lower()


def _infer_id_param(resource_name):
    """推断资源的 ID 参数名 - 使用简单的 'id' 避免字段不匹配"""
    # 使用通用的 'id' 以确保与大多数 Request message 的 string id = 1 字段兼容
    return "id"


def _to_snake_case(text):
    """将 PascalCase 转换为 snake_case"""
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', text)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


def add_http_annotations(proto_file_path, in_place=True):
    """为 proto 文件添加 HTTP 注解"""
    proto_path = Path(proto_file_path)
    
    if not proto_path.exists():
        print(f"❌ 文件不存在: {proto_file_path}")
        return False
    
    with open(proto_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 跳过已有注解的文件
    if 'google.api.http' in content:
        print(f"⏭️  跳过 (已有注解): {proto_path.name}")
        return True
    
    # 添加 import
    if 'google/api/annotations.proto' not in content:
        # 在最后一个 import 后添加
        import_pattern = r'(import\s+"[^"]+";)(\s*\n)(?=\s*(?:option|package|message|service|enum|//))'
        
        def add_import(match):
            return match.group(1) + '\nimport "google/api/annotations.proto";' + match.group(2)
        
        content = re.sub(import_pattern, add_import, content, count=1)
    
    # 提取包名
    package_match = re.search(r'package\s+([\w.]+);', content)
    package_name = package_match.group(1) if package_match else ""
    
    # 推断基础路径
    base_path = infer_base_path(package_name, proto_path)
    
    # 查找并替换 RPC 定义 (未注解的)
    # 匹配: rpc MethodName(Request) returns (Response);
    rpc_pattern = r'(\s*)(//[^\n]*\n\s*)?rpc\s+(\w+)\s*\(([^)]+)\)\s*returns\s*\(([^)]+)\)\s*;'
    
    count = 0
    def replace_rpc(match):
        nonlocal count
        indent = match.group(1)
        comment = match.group(2) or ""
        rpc_name = match.group(3)
        request_type = match.group(4).strip()
        response_type = match.group(5).strip()
        
        http_method, path, body = infer_http_method_and_path(rpc_name, request_type, base_path)
        
        # Build annotation block
        lines = [
            f"{indent}{comment}rpc {rpc_name}({request_type}) returns ({response_type}) {{",
            f"{indent}  option (google.api.http) = {{",
            f'{indent}    {http_method}: "{path}"'
        ]
        
        if body:
            lines.append(f'{indent}    body: "{body}"')
        
        lines.append(f"{indent}  }};")
        lines.append(f"{indent}}}")
        
        count += 1
        return "\n".join(lines)
    
    new_content = re.sub(rpc_pattern, replace_rpc, content)
    
    if count == 0:
        print(f"⏭️  跳过 (无需处理): {proto_path.name}")
        return True
    
    # 写入文件
    output_path = proto_path if in_place else proto_path.parent / f"{proto_path.stem}_annotated.proto"
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
    
    print(f"✅ {proto_path.name}: 处理了 {count} 个 RPC")
    return True


def process_all_protos():
    """处理所有 proto 文件"""
    proto_files = glob.glob("protos/**/*_service.proto", recursive=True)
    
    if not proto_files:
        print("❌ 未找到任何 *_service.proto 文件")
        return False
    
    print(f"🔍 找到 {len(proto_files)} 个服务 proto 文件")
    print("=" * 50)
    
    success_count = 0
    for proto_file in sorted(proto_files):
        if add_http_annotations(proto_file, in_place=True):
            success_count += 1
    
    print("=" * 50)
    print(f"✅ 完成: {success_count}/{len(proto_files)} 个文件处理成功")
    return True


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("用法:")
        print("  python3 scripts/add_http_annotations.py --all")
        print("  python3 scripts/add_http_annotations.py <proto_file>")
        sys.exit(1)
    
    if sys.argv[1] == "--all":
        success = process_all_protos()
    else:
        success = add_http_annotations(sys.argv[1])
    
    sys.exit(0 if success else 1)
