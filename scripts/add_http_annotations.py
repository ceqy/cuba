#!/usr/bin/env python3
"""
自动为 Proto 文件中的所有 RPC 添加 google.api.http 注解
用法: python3 scripts/add_http_annotations.py protos/finance/gl_journal_entry_service.proto
"""

import re
import sys
from pathlib import Path


def infer_http_method_and_path(rpc_name, request_type, package_name):
    """
    根据 RPC 名称推断 HTTP 方法和路径
    
    Args:
        rpc_name: RPC 方法名，如 "CreateJournalEntry"
        request_type: 请求类型，如 "CreateJournalEntryRequest"
        package_name: 包名，如 "finance.gl"
    
    Returns:
        tuple: (http_method, path, body)
    """
    # 从包名推断基础路径
    base_path = "/api/v1/finance"
    
    # 资源名称映射
    resource_patterns = {
        "JournalEntry": "journal-entries",
        "JournalEntries": "journal-entries",
        "OpenItem": "open-items",
        "OpenItems": "open-items",
        "Attachment": "attachments",
        "Attachments": "attachments",
        "Approval": "approvals",
        "Template": "templates",
        "Templates": "templates",
        "RecurringEntry": "recurring-entries",
        "RecurringEntries": "recurring-entries",
        "ParkedJournalEntry": "parked-journal-entries",
        "ParkedJournalEntries": "parked-journal-entries",
        "BatchInputSession": "batch-sessions",
        "DocumentChain": "document-chains",
        "AccountBalance": "account-balances",
        "AccountLineItem": "account-line-items",
    }
    
    # 创建操作 - POST
    if rpc_name.startswith("Create"):
        resource = rpc_name.replace("Create", "")
        path_segment = resource_patterns.get(resource, _to_kebab_case(resource))
        return ("post", f"{base_path}/{path_segment}", "*")
    
    # 批量创建 - POST
    if rpc_name.startswith("BatchCreate"):
        resource = rpc_name.replace("BatchCreate", "")
        path_segment = resource_patterns.get(resource, _to_kebab_case(resource))
        return ("post", f"{base_path}/{path_segment}/batch", "*")
    
    # 获取单个资源 - GET with ID
    if rpc_name.startswith("Get") and "List" not in rpc_name and "Statistics" not in rpc_name:
        resource = rpc_name.replace("Get", "")
        path_segment = resource_patterns.get(resource, _to_kebab_case(resource))
        
        # 特殊处理：需要路径参数的情况
        if "ById" in resource or "ByAccount" in resource or "History" in resource:
            return ("get", f"{base_path}/{path_segment}", None)
        
        # 默认使用 ID 参数
        id_param = _infer_id_param(resource)
        return ("get", f"{base_path}/{path_segment}/{{{id_param}}}", None)
    
    # 列表查询 - GET
    if rpc_name.startswith("List"):
        resource = rpc_name.replace("List", "")
        path_segment = resource_patterns.get(resource, _to_kebab_case(resource))
        return ("get", f"{base_path}/{path_segment}", None)
    
    # 更新操作 - PUT
    if rpc_name.startswith("Update"):
        resource = rpc_name.replace("Update", "")
        path_segment = resource_patterns.get(resource, _to_kebab_case(resource))
        id_param = _infer_id_param(resource)
        return ("put", f"{base_path}/{path_segment}/{{{id_param}}}", "*")
    
    # 删除操作 - DELETE
    if rpc_name.startswith("Delete"):
        resource = rpc_name.replace("Delete", "")
        path_segment = resource_patterns.get(resource, _to_kebab_case(resource))
        id_param = _infer_id_param(resource)
        return ("delete", f"{base_path}/{path_segment}/{{{id_param}}}", None)
    
    # 动作型操作 - POST with action
    action_verbs = [
        "Post", "Reverse", "Cancel", "Reset", "Clear", "Validate", "Simulate",
        "Approve", "Reject", "Submit", "Execute", "Process", "Upload", "Download",
        "Export", "Generate", "Park", "Reconcile", "Revaluate", "Reclassify",
        "CarryForward", "Configure", "Adjust", "Recalculate", "Save"
    ]
    
    for verb in action_verbs:
        if rpc_name.startswith(verb):
            resource = rpc_name.replace(verb, "")
            action = _to_kebab_case(verb)
            
            # 如果包含资源名，使用资源路径
            if resource:
                path_segment = resource_patterns.get(resource, _to_kebab_case(resource))
                # 某些操作需要 ID，某些不需要
                if verb in ["Post", "Reverse", "Cancel", "Reset", "Approve", "Reject"]:
                    id_param = _infer_id_param(resource)
                    return ("post", f"{base_path}/{path_segment}/{{{id_param}}}/{action}", "*")
                else:
                    return ("post", f"{base_path}/{path_segment}/{action}", "*")
            else:
                # 没有资源名，直接使用动作
                return ("post", f"{base_path}/{action}", "*")
    
    # 批量操作 - POST
    if rpc_name.startswith("Batch"):
        action = _to_kebab_case(rpc_name.replace("Batch", ""))
        return ("post", f"{base_path}/batch/{action}", "*")
    
    # 默认：POST
    return ("post", f"{base_path}/{_to_kebab_case(rpc_name)}", "*")


def _to_kebab_case(text):
    """将 PascalCase 转换为 kebab-case"""
    # 插入连字符
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1-\2', text)
    return re.sub('([a-z0-9])([A-Z])', r'\1-\2', s1).lower()


def _infer_id_param(resource_name):
    """推断资源的 ID 参数名"""
    # 移除复数形式
    singular = resource_name.rstrip('s').rstrip('ie') + 'y' if resource_name.endswith('ies') else resource_name.rstrip('s')
    return f"{_to_kebab_case(singular)}_id"


def add_http_annotations(proto_file_path):
    """
    为 proto 文件添加 HTTP 注解
    
    Args:
        proto_file_path: proto 文件路径
    """
    proto_path = Path(proto_file_path)
    
    if not proto_path.exists():
        print(f"❌ 文件不存在: {proto_file_path}")
        return False
    
    # 读取文件
    with open(proto_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 检查是否已经有注解
    if 'google/api/annotations.proto' in content:
        print("⚠️  文件已包含 google/api/annotations.proto 导入")
    else:
        # 在 import 区域添加导入
        import_pattern = r'(import\s+"google/protobuf/timestamp\.proto";)'
        replacement = r'\1\nimport "google/api/annotations.proto";'
        content = re.sub(import_pattern, replacement, content)
        print("✅ 已添加 google/api/annotations.proto 导入")
    
    # 提取包名
    package_match = re.search(r'package\s+([\w.]+);', content)
    package_name = package_match.group(1) if package_match else ""
    
    # 查找所有 RPC 定义
    rpc_pattern = r'(\s*//[^\n]*\n)*\s*rpc\s+(\w+)\s*\((\w+)\)\s*returns\s*\((\w+)\)\s*;'
    
    def replace_rpc(match):
        comments = match.group(1) or ""
        rpc_name = match.group(2)
        request_type = match.group(3)
        response_type = match.group(4)
        
        # 推断 HTTP 方法和路径
        http_method, path, body = infer_http_method_and_path(rpc_name, request_type, package_name)
        
        # 构建注解
        indent = "  "
        annotation_lines = [
            f"{comments}{indent}rpc {rpc_name}({request_type}) returns ({response_type}) {{",
            f"{indent}  option (google.api.http) = {{"
        ]
        
        annotation_lines.append(f'{indent}    {http_method}: "{path}"')
        
        if body:
            annotation_lines.append(f'{indent}    body: "{body}"')
        
        annotation_lines.append(f"{indent}  }};")
        annotation_lines.append(f"{indent}}}")
        
        return "\n".join(annotation_lines)
    
    # 替换所有 RPC
    new_content, count = re.subn(rpc_pattern, replace_rpc, content)
    
    # 保存到新文件
    output_path = proto_path.parent / f"{proto_path.stem}_annotated.proto"
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(new_content)
    
    print(f"✅ 已处理 {count} 个 RPC 方法")
    print(f"✅ 输出文件: {output_path}")
    print(f"\n建议: 检查生成的文件并根据需要调整，然后替换原文件：")
    print(f"  mv {output_path} {proto_path}")
    
    return True


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("用法: python3 scripts/add_http_annotations.py <proto_file_path>")
        print("示例: python3 scripts/add_http_annotations.py protos/finance/gl_journal_entry_service.proto")
        sys.exit(1)
    
    proto_file = sys.argv[1]
    success = add_http_annotations(proto_file)
    sys.exit(0 if success else 1)
