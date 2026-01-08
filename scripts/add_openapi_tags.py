#!/usr/bin/env python3
"""
为 Proto 文件自动添加 OpenAPI 标签分组
"""

import re
import sys

# Auth Service 方法分类
AUTH_CATEGORIES = {
    "Identity Management": [
        "Register", "Login", "Logout", "RefreshToken", "GetUserInfo",
        "UpdateProfile", "ChangePassword", "ResetPassword", "VerifyEmail"
    ],
    "Security & 2FA": [
        "Enable2FA", "Disable2FA", "Verify2FA", "GenerateBackupCodes",
        "VerifyBackupCode"
    ],
    "Role & Permission Management": [
        "CreateRole", "GetRole", "UpdateRole", "DeleteRole", "ListRoles",
        "AssignRole", "RevokeRole", "GetUserRoles",
        "CreatePermission", "GetPermission", "UpdatePermission", "DeletePermission",
        "ListPermissions", "AssignPermission", "RevokePermission"
    ],
    "Tenant Management": [
        "CreateTenant", "GetTenant", "UpdateTenant", "DeleteTenant", "ListTenants",
        "AddTenantMember", "RemoveTenantMember", "ListTenantMembers",
        "UpdateTenantMemberRole"
    ]
}

# Finance Service 方法分类
FINANCE_CATEGORIES = {
    "Journal Entry Core": [
        "CreateJournalEntry", "GetJournalEntry", "UpdateJournalEntry",
        "DeleteJournalEntry", "PostJournalEntry", "CancelJournalEntry",
        "ReverseJournalEntry", "ResetJournalEntry", "ListJournalEntries"
    ],
    "Workflow & Approval": [
        "ApproveJournalEntry", "RejectJournalEntry", "GetApprovalHistory",
        "SubmitForApproval"
    ],
    "Parked Documents": [
        "CreateParkedJournalEntry", "PostParkedJournalEntry",
        "GetParkedJournalEntry", "UpdateParkedJournalEntry",
        "DeleteParkedJournalEntry", "ListParkedJournalEntries"
    ],
    "Clearing & Reversal": [
        "ClearOpenItems", "ResetClearing", "GetClearingHistory",
        "ReclassifyAccounts"
    ],
    "Reporting & Balances": [
        "GetAccountBalances", "GetAccountLineItems", "GetJournalEntryHistory",
        "GetParallelLedgerData", "CarryForwardBalances"
    ],
    "Attachments & Templates": [
        "UploadAttachment", "DownloadAttachment", "DeleteAttachment",
        "ListAttachments", "SaveAsTemplate", "CreateFromTemplate"
    ],
    "Batch Operations": [
        "BatchCreateJournalEntries", "BatchReverseJournalEntries",
        "CreateBatchInputSession", "ProcessBatchInputSession",
        "GetBatchInputSessionStatus"
    ],
    "Advanced Features": [
        "GetJournalEntryTaxDetails", "GetDocumentChain",
        "GetDocumentSplitDetails", "CreateRecurringEntry",
        "GetPaymentInformation", "UpdatePaymentStatus"
    ]
}


def get_category(method_name, categories):
    """根据方法名获取分类"""
    for category, methods in categories.items():
        if method_name in methods:
            return category
    return None


def add_tags_to_proto(file_path, categories, service_name):
    """为 Proto 文件添加标签"""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 匹配 rpc 方法定义
    rpc_pattern = r'(  // [^\n]+\n  rpc (\w+)\([^)]+\) returns \([^)]+\) \{\n    option \(google\.api\.http\) = \{[^}]+\};)\n  \}'
    
    def replace_rpc(match):
        full_match = match.group(1)
        method_name = match.group(2)
        category = get_category(method_name, categories)
        
        if category:
            # 提取中文注释作为 summary
            comment_match = re.search(r'// (.+)', full_match)
            summary = comment_match.group(1) if comment_match else method_name
            
            tag_option = f'''
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {{
      tags: "{category}"
      summary: "{summary}"
    }};'''
            return full_match + tag_option + '\n  }'
        
        return match.group(0)
    
    # 替换所有 rpc 方法
    modified_content = re.sub(rpc_pattern, replace_rpc, content)
    
    # 写回文件
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(modified_content)
    
    print(f"✅ 已为 {file_path} 添加标签")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("用法: python3 add_openapi_tags.py <auth|finance>")
        sys.exit(1)
    
    service_type = sys.argv[1]
    
    if service_type == "auth":
        add_tags_to_proto(
            "protos/auth/auth_service.proto",
            AUTH_CATEGORIES,
            "AuthService"
        )
    elif service_type == "finance":
        add_tags_to_proto(
            "protos/finance/gl/gl_journal_entry.proto",
            FINANCE_CATEGORIES,
            "GlJournalEntryService"
        )
    else:
        print("❌ 未知服务类型，请使用 'auth' 或 'finance'")
        sys.exit(1)
