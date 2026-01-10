#!/usr/bin/env python3
import json
import sys
import os
from pathlib import Path

# =============================================================================
# Configuration: Service Metadata (filename -> metadata)
# =============================================================================
SERVICE_METADATA = {
    # Asset Management
    "asset_maintenance_service": {
        "title": "Asset Maintenance Service API",
        "description": "资产维护管理服务 - 设备维护计划、工单管理、预防性维护",
        "contact": {"name": "Asset Team", "email": "asset@enterprise.com"}
    },
    "ehs_incident_service": {
        "title": "EHS Incident Service API",
        "description": "环境健康安全事件管理服务 - 事件报告、调查分析、CAPA管理",
        "contact": {"name": "EHS Team", "email": "ehs@enterprise.com"}
    },
    "intelligent_asset_health_service": {
        "title": "Intelligent Asset Health Service API",
        "description": "智能资产健康监控服务 - 预测性维护、健康评分、故障预警",
        "contact": {"name": "Asset Intelligence Team", "email": "asset-ai@enterprise.com"}
    },
    "geo_service": {
        "title": "Geolocation Service API",
        "description": "地理位置服务 - 地理编码、路径规划、距离计算",
        "contact": {"name": "Platform Team", "email": "platform@enterprise.com"}
    },
    
    # Finance
    "controlling_allocation_service": {
        "title": "Cost Allocation Service API",
        "description": "成本分配服务 - 成本中心分配、循环规则、分配执行",
        "contact": {"name": "Finance CO Team", "email": "finance-co@enterprise.com"}
    },
    "treasury_services": {
        "title": "Treasury Service API",
        "description": "资金管理服务 - 现金管理、银行对账、支付处理",
        "contact": {"name": "Treasury Team", "email": "treasury@enterprise.com"}
    },
    
    # HR
    "employee_experience_service": {
        "title": "Employee Experience Service API",
        "description": "员工体验服务 - 员工调查、认可奖励、敬业度管理",
        "contact": {"name": "HR Team", "email": "hr@enterprise.com"}
    },
    "talent_acquisition_service": {
        "title": "Talent Acquisition Service API",
        "description": "人才招聘服务 - 职位发布、候选人管理、面试安排",
        "contact": {"name": "Talent Team", "email": "talent@enterprise.com"}
    },
    
    # Manufacturing
    "kanban_service": {
        "title": "Kanban Service API",
        "description": "看板管理服务 - 看板触发、补货管理、库存拉动",
        "contact": {"name": "Manufacturing Team", "email": "manufacturing@enterprise.com"}
    },
    "outsourced_manufacturing_service": {
        "title": "Outsourced Manufacturing Service API",
        "description": "委外加工服务 - 外协订单、物料发放、成品接收",
        "contact": {"name": "Manufacturing Team", "email": "manufacturing@enterprise.com"}
    },
    "production_planning_service": {
        "title": "Production Planning Service API",
        "description": "生产计划服务 - MRP运算、工单生成、能力平衡",
        "contact": {"name": "Planning Team", "email": "planning@enterprise.com"}
    },
    "quality_inspection_service": {
        "title": "Quality Inspection Service API",
        "description": "质量检验服务 - 检验批次、质检结果、使用决策",
        "contact": {"name": "Quality Team", "email": "quality@enterprise.com"}
    },
    "shop_floor_execution_service": {
        "title": "Shop Floor Execution Service API",
        "description": "车间执行服务 - 工序确认、物料消耗、产出报工",
        "contact": {"name": "MES Team", "email": "mes@enterprise.com"}
    },
    
    # Procurement
    "contract_management_service": {
        "title": "Contract Management Service API",
        "description": "合同管理服务 - 合同创建、里程碑跟踪、履约管理",
        "contact": {"name": "Procurement Team", "email": "procurement@enterprise.com"}
    },
    "invoice_processing_service": {
        "title": "Invoice Processing Service API",
        "description": "发票处理服务 - 发票校验、三单匹配、自动审批",
        "contact": {"name": "AP Team", "email": "ap@enterprise.com"}
    },
    "order_service": {
        "title": "Purchase Order Service API",
        "description": "采购订单服务 - 订单创建、审批发布、跟踪管理",
        "contact": {"name": "Procurement Team", "email": "procurement@enterprise.com"}
    },
    "sourcing_event_service": {
        "title": "Sourcing Event Service API",
        "description": "寻源活动服务 - RFQ管理、供应商报价、商务评标",
        "contact": {"name": "Sourcing Team", "email": "sourcing@enterprise.com"}
    },
    "spend_analytics_service": {
        "title": "Spend Analytics Service API",
        "description": "支出分析服务 - 采购分析、供应商绩效、成本洞察",
        "contact": {"name": "Procurement Analytics Team", "email": "procurement-analytics@enterprise.com"}
    },
    "supplier_portal_service": {
        "title": "Supplier Portal Service API",
        "description": "供应商门户服务 - PO确认、ASN发送、协同管理",
        "contact": {"name": "Supplier Relations Team", "email": "supplier@enterprise.com"}
    },
    
    # R&D
    "plm_integration_service": {
        "title": "PLM Integration Service API",
        "description": "PLM集成服务 - BOM同步、物料主数据、ECN变更",
        "contact": {"name": "R&D Team", "email": "rd@enterprise.com"}
    },
    "project_cost_controlling_service": {
        "title": "Project Cost Controlling Service API",
        "description": "项目成本控制服务 - 预算管理、实际成本、差异分析",
        "contact": {"name": "Project Team", "email": "project@enterprise.com"}
    },
    
    # Sales
    "analytics_service": {
        "title": "Sales Analytics Service API",
        "description": "销售分析服务 - 销售漏斗、业绩预测、客户洞察",
        "contact": {"name": "Sales Analytics Team", "email": "sales-analytics@enterprise.com"}
    },
    "pricing_engine_service": {
        "title": "Pricing Engine Service API",
        "description": "定价引擎服务 - 价格计算、折扣规则、条件逻辑",
        "contact": {"name": "Pricing Team", "email": "pricing@enterprise.com"}
    },
    "revenue_recognition_service": {
        "title": "Revenue Recognition Service API",
        "description": "收入确认服务 - 收入分摊、递延收入、IFRS15合规",
        "contact": {"name": "Revenue Team", "email": "revenue@enterprise.com"}
    },
    "sales_order_fulfillment_service": {
        "title": "Sales Order Fulfillment Service API",
        "description": "订单履行服务 - 订单处理、发货管理、ATP检查",
        "contact": {"name": "Order Fulfillment Team", "email": "fulfillment@enterprise.com"}
    },
    
    # Service
    "contract_billing_service": {
        "title": "Contract Billing Service API",
        "description": "合同计费服务 - 计费计划、里程碑计费、周期性开票",
        "contact": {"name": "Service Billing Team", "email": "service-billing@enterprise.com"}
    },
    "field_service_dispatch_service": {
        "title": "Field Service Dispatch Service API",
        "description": "现场服务调度 - 工单分派、技术员调度、路径优化",
        "contact": {"name": "Field Service Team", "email": "field-service@enterprise.com"}
    },
    "warranty_claims_service": {
        "title": "Warranty Claims Service API",
        "description": "质保索赔服务 - 索赔提交、审核处理、供应商回收",
        "contact": {"name": "Warranty Team", "email": "warranty@enterprise.com"}
    },
    
    # Supply Chain
    "batch_traceability_service": {
        "title": "Batch Traceability Service API",
        "description": "批次追溯服务 - 产品追溯、批次查询、供应链可视化",
        "contact": {"name": "Supply Chain Team", "email": "supplychain@enterprise.com"}
    },
    "demand_forecasting_service": {
        "title": "Demand Forecasting Service API",
        "description": "需求预测服务 - 统计预测、机器学习、需求计划",
        "contact": {"name": "Planning Team", "email": "planning@enterprise.com"}
    },
    "inventory_management_service": {
        "title": "Inventory Management Service API",
        "description": "库存管理服务 - 库存查询、移动过账、盘点管理",
        "contact": {"name": "Warehouse Team", "email": "warehouse@enterprise.com"}
    },
    "transportation_planning_service": {
        "title": "Transportation Planning Service API",
        "description": "运输计划服务 - 货运计划、承运商管理、运输执行",
        "contact": {"name": "Logistics Team", "email": "logistics@enterprise.com"}
    },
    "visibility_service": {
        "title": "Supply Chain Visibility Service API",
        "description": "供应链可视化服务 - 实时跟踪、流程监控、异常预警",
        "contact": {"name": "Visibility Team", "email": "visibility@enterprise.com"}
    },
    "warehouse_operations_service": {
        "title": "Warehouse Operations Service API",
        "description": "仓库作业服务 - 上架下架、拣货包装、Wave管理",
        "contact": {"name": "Warehouse Ops Team", "email": "warehouse-ops@enterprise.com"}
    },
}

# =============================================================================
# Configuration: Tag Descriptions (English -> Chinese)
# =============================================================================
TAG_DESCRIPTIONS = {
    # Finance - GL
    "Journal Entry Core": "总账凭证核心 (Journal Entry Core) - 增删改查",
    "Workflow & Approval": "工作流与审批 (Workflow) - 审核流程",
    "Batch Operations": "批量操作 (Batch) - 批量导入与处理",
    
    # Auth
    "Identity Management": "身份管理 (Identity) - 登录与注册",
    "Security & 2FA": "安全与双因素 (Security) - 2FA与验证",
    "Tenant Management": "租户管理 (Tenant) - 组织架构",
    
    # Common
    "Utility": "通用工具 (Utility)",
}

def infer_service_key(file_path):
    """从文件路径推断服务键名"""
    filename = Path(file_path).stem.replace('.openapi3', '')
    return filename

def enrich_openapi(file_path):
    print(f"✨ Enhancing OpenAPI JSON: {file_path}")
    
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            spec = json.load(f)
    except Exception as e:
        print(f"❌ Failed to read file: {e}")
        sys.exit(1)

    # 1. Ensure OpenAPI 3.1.0 version (Swagger UI 5.x supports it)
    if not spec.get('openapi', '').startswith('3.'):
        print(f"⚠️ Warning: Spec version is {spec.get('openapi')}, expected 3.x")
    
    # 1.5. Enrich Info Section with Service Metadata
    service_key = infer_service_key(file_path)
    if service_key in SERVICE_METADATA:
        metadata = SERVICE_METADATA[service_key]
        if 'info' not in spec:
            spec['info'] = {}
        
        spec['info']['title'] = metadata.get('title', spec.get('info', {}).get('title', 'API'))
        spec['info']['version'] = spec.get('info', {}).get('version', '0.1.0')
        spec['info']['description'] = metadata.get('description', '')
        
        if 'contact' in metadata:
            spec['info']['contact'] = metadata['contact']
    else:
        # Fallback for services without metadata
        if 'info' not in spec:
            spec['info'] = {'title': 'Enterprise API', 'version': '0.1.0'}

    # 2. Inject Security Schemes (Bearer Auth)
    if 'components' not in spec:
        spec['components'] = {}
    
    if 'securitySchemes' not in spec['components']:
        spec['components']['securitySchemes'] = {}

    # Add Bearer Token definition
    spec['components']['securitySchemes']['BearerAuth'] = {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT",
        "description": "Enter your JWT token (e.g. from Login API)"
    }

    # 3. Apply Security Globally (Enable lock icon for all endpoints)
    if 'security' not in spec:
        spec['security'] = []
    
    # Check if already exists to avoid duplicates
    has_bearer = any('BearerAuth' in s for s in spec['security'])
    if not has_bearer:
        spec['security'].append({"BearerAuth": []})

    # 4. Enhance Tags (Group Descriptions)
    # Collect existing tags from paths to ensure we don't miss any
    existing_tags = set()
    if 'paths' in spec:
        for path, methods in spec['paths'].items():
            for method, details in methods.items():
                if 'tags' in details:
                    for tag in details['tags']:
                        existing_tags.add(tag)

    # Re-build top-level tags list with descriptions
    if 'tags' not in spec:
        spec['tags'] = []
    
    # Create a map of existing definitions
    defined_tags = {t['name']: t for t in spec['tags']}

    for tag_name in existing_tags:
        # Get description from config or keep existing or default
        desc = TAG_DESCRIPTIONS.get(tag_name, defined_tags.get(tag_name, {}).get('description', tag_name))
        
        defined_tags[tag_name] = {
            "name": tag_name,
            "description": desc
        }

    # Convert back to list
    spec['tags'] = sorted(list(defined_tags.values()), key=lambda x: x['name'])

    # 5. Save enhanced JSON
    try:
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(spec, f, indent=2, ensure_ascii=False)
        print("✅ Successfully enriched OpenAPI spec")
    except Exception as e:
        print(f"❌ Failed to save file: {e}")
        sys.exit(1)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 enrich_openapi.py <path_to_openapi.json>")
        sys.exit(1)
    
    enrich_openapi(sys.argv[1])
