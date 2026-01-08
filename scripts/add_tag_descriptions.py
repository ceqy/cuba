#!/usr/bin/env python3
"""
为生成的 OpenAPI 3.0 文档添加 Tag 描述
"""

import json
import sys

# Auth Service Tag 描述
AUTH_TAG_DESCRIPTIONS = {
    "Identity Management": "身份管理 - 用户注册、登录、登出及个人信息管理",
    "Security & 2FA": "安全与双因素认证 - 启用、禁用和验证双因素认证",
    "Role & Permission Management": "角色与权限管理 - RBAC 权限体系的完整管理",
    "Tenant Management": "租户管理 - 多租户架构的租户和成员管理"
}

# Finance Service Tag 描述
FINANCE_TAG_DESCRIPTIONS = {
    "Journal Entry Core": "核心凭证操作 - 创建、查询、过账、撤销等基础凭证功能",
    "Workflow & Approval": "工作流与审批 - 凭证审批流程和审批历史查询",
    "Parked Documents": "暂存凭证 - 暂存凭证的创建、过账和管理",
    "Clearing & Reversal": "清账与结转 - 未清项目清账、重分类和余额结转",
    "Reporting & Balances": "报表与余额 - 科目余额、明细和并行分类账数据查询",
    "Attachments & Templates": "附件与模板 - 凭证附件管理和模板功能",
    "Batch Operations": "批量操作 - 批量创建、冲销和批量输入会话管理",
    "Advanced Features": "高级功能 - 税务详情、凭证链、拆分和循环凭证等"
}


def add_tag_descriptions(file_path, tag_descriptions):
    """为 OpenAPI 文档添加 Tag 描述"""
    with open(file_path, 'r', encoding='utf-8') as f:
        spec = json.load(f)
    
    # 收集所有使用的 tags
    used_tags = set()
    for path_item in spec.get('paths', {}).values():
        for operation in path_item.values():
            if isinstance(operation, dict) and 'tags' in operation:
                used_tags.update(operation['tags'])
    
    # 创建 tags 数组
    tags = []
    
    # 保留原有的服务级别 tag
    if 'tags' in spec and spec['tags']:
        tags.extend(spec['tags'])
    
    # 添加分组 tags
    for tag_name in sorted(used_tags):
        # 跳过服务名称的 tag
        if tag_name in ['AuthService', 'GlJournalEntryService']:
            continue
        
        description = tag_descriptions.get(tag_name, tag_name)
        tags.append({
            "name": tag_name,
            "description": description
        })
    
    spec['tags'] = tags
    
    # 写回文件
    with open(file_path, 'w', encoding='utf-8') as f:
        json.dump(spec, f, ensure_ascii=False, indent=2)
    
    print(f"✅ 已为 {file_path} 添加 {len(tags)} 个标签描述")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("用法: python3 add_tag_descriptions.py <auth|finance>")
        sys.exit(1)
    
    service_type = sys.argv[1]
    
    if service_type == "auth":
        add_tag_descriptions(
            "docs/auth/auth_service.openapi3.json",
            AUTH_TAG_DESCRIPTIONS
        )
    elif service_type == "finance":
        add_tag_descriptions(
            "docs/finance/gl_journal_entry.openapi3.json",
            FINANCE_TAG_DESCRIPTIONS
        )
    else:
        print("❌ 未知服务类型，请使用 'auth' 或 'finance'")
        sys.exit(1)
